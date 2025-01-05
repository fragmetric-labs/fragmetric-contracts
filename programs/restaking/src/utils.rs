use anchor_lang::prelude::*;
use anchor_lang::solana_program::{entrypoint, system_program};
use anchor_lang::{CheckOwner, ZeroCopy};

use crate::errors;

pub trait PDASeeds<const N: usize> {
    const SEED: &'static [u8];
    fn get_bump(&self) -> u8;
    fn get_seeds(&self) -> [&[u8]; N];
}

/// Zero-copy account that has header (data-version, bump).
pub trait ZeroCopyHeader: ZeroCopy {
    /// Offset of bump (8bit)
    fn get_bump_offset() -> usize;
}

/// An extension trait for [AccountLoader].
pub trait AccountLoaderExt<'info> {
    /// Sets zero-copy header when initializing the account
    /// even without enough account data size provided.
    ///
    /// This operation is almost equivalent to [load_init](AccountLoader::load_init),
    /// but skips bytemuck's pointer type casting and accesses directly
    /// to byte array using offset.
    fn initialize_zero_copy_header(&mut self, bump: u8) -> Result<()>;

    /// Reads bump directly from data without
    /// borsh deserialization or bytemuck type casting.
    fn get_bump(&self) -> Result<u8>;

    /// Realloc account to increase extra amount of data size.
    /// It will add at most 10KB([`MAX_PERMITTED_DATA_INCREASE`]).
    ///
    /// [`MAX_PERMITTED_DATA_INCREASE`]: MAX_PERMITTED_DATA_INCREASE
    fn expand_account_size_if_needed(
        &self,
        payer: &Signer<'info>,
        system_program: &Program<'info, System>,
        desired_account_size: Option<u32>,
    ) -> Result<()>;
}

impl<'info, T: ZeroCopyHeader + Owner> AccountLoaderExt<'info> for AccountLoader<'info, T> {
    fn initialize_zero_copy_header(&mut self, bump: u8) -> Result<()> {
        if !self.as_ref().is_writable {
            return Err(ErrorCode::AccountNotMutable.into());
        }

        // The discriminator should be zero, since we're initializing.
        let mut data = self.as_ref().try_borrow_mut_data()?;
        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);
        let discriminator = u64::from_le_bytes(disc_bytes);
        if discriminator != 0 {
            return Err(ErrorCode::AccountDiscriminatorAlreadySet.into());
        }

        // Safety check
        let offset = 8 + T::get_bump_offset();
        if data.len() < offset + 1 {
            return Err(ErrorCode::ConstraintSpace.into());
        }

        // Sets bump.
        data[offset] = bump;

        Ok(())
    }

    fn get_bump(&self) -> Result<u8> {
        let data = self.as_ref().try_borrow_data()?;

        // Safety check
        let offset = 8 + T::get_bump_offset();
        if data.len() < offset + 1 {
            return Err(ErrorCode::ConstraintSpace.into());
        }

        Ok(data[offset])
    }

    fn expand_account_size_if_needed(
        &self,
        payer: &Signer<'info>,
        system_program: &Program<'info, System>,
        desired_account_size: Option<u32>,
    ) -> Result<()> {
        let account_info = self.as_ref();

        let current_account_size = account_info.data_len();
        let min_account_size = 8 + std::mem::size_of::<T>();
        let target_account_size = desired_account_size
            .map(|size| std::cmp::max(size as usize, min_account_size))
            .unwrap_or(min_account_size);
        let required_realloc_size = target_account_size.saturating_sub(current_account_size);

        msg!(
            "realloc account size: current={}, target={}, required={}",
            current_account_size,
            target_account_size,
            required_realloc_size
        );

        if required_realloc_size > 0 {
            let rent = Rent::get()?;
            let current_lamports = account_info.lamports();
            let minimum_lamports = rent.minimum_balance(target_account_size);
            let required_lamports = minimum_lamports.saturating_sub(current_lamports);
            if required_lamports > 0 {
                let cpi_context = CpiContext::new(
                    system_program.to_account_info(),
                    anchor_lang::system_program::Transfer {
                        from: payer.to_account_info(),
                        to: account_info.clone(),
                    },
                );
                anchor_lang::system_program::transfer(cpi_context, required_lamports)?;
                msg!("realloc account lamports: added={}", required_lamports);
            }

            let increase = std::cmp::min(
                required_realloc_size,
                entrypoint::MAX_PERMITTED_DATA_INCREASE,
            );
            let new_account_size = current_account_size + increase;

            account_info.realloc(new_account_size, false)?;
            msg!(
                "account reallocated: current={}, target={}, required={}",
                new_account_size,
                target_account_size,
                target_account_size - new_account_size
            );
        }

        Ok(())
    }
}

/// drops sub-decimal values.
/// when both numerator and denominator are zero, returns `amount`.
pub fn get_proportional_amount(amount: u64, numerator: u64, denominator: u64) -> Result<u64> {
    if numerator == denominator {
        return Ok(amount);
    }

    (amount as u128)
        .checked_mul(numerator as u128)
        .and_then(|v| v.checked_div(denominator as u128))
        .and_then(|v| u64::try_from(v).ok())
        .ok_or_else(|| error!(errors::ErrorCode::CalculationArithmeticException))
}

#[allow(dead_code)]
pub trait AccountInfoExt<'info> {
    fn is_initialized(&self) -> bool;

    fn parse_account_boxed<T>(&'info self) -> Result<Box<Account<'info, T>>>
    where
        T: AccountSerialize + AccountDeserialize + Clone + Owner;

    fn parse_interface_account_boxed<T>(&'info self) -> Result<Box<InterfaceAccount<'info, T>>>
    where
        T: AccountSerialize + AccountDeserialize + Clone + CheckOwner;

    fn parse_optional_account_boxed<T>(&'info self) -> Result<Option<Box<Account<'info, T>>>>
    where
        T: AccountSerialize + AccountDeserialize + Clone + Owner;

    fn parse_optional_account_loader<T>(&'info self) -> Result<Option<AccountLoader<'info, T>>>
    where
        T: ZeroCopy + Owner;

    /// Treats system program account as `None`.
    fn to_option(&self) -> Option<&Self>;
}

impl<'info> AccountInfoExt<'info> for AccountInfo<'info> {
    fn is_initialized(&self) -> bool {
        self.owner != &system_program::ID || !self.data_is_empty()
    }

    #[inline(never)]
    fn parse_account_boxed<T>(&'info self) -> Result<Box<Account<'info, T>>>
    where
        T: AccountSerialize + AccountDeserialize + Clone + Owner,
    {
        Account::try_from(self).map(Box::new)
    }

    #[inline(never)]
    fn parse_interface_account_boxed<T>(&'info self) -> Result<Box<InterfaceAccount<'info, T>>>
    where
        T: AccountSerialize + AccountDeserialize + Clone + CheckOwner,
    {
        InterfaceAccount::try_from(self).map(Box::new)
    }

    #[inline(never)]
    fn parse_optional_account_boxed<T>(&'info self) -> Result<Option<Box<Account<'info, T>>>>
    where
        T: AccountSerialize + AccountDeserialize + Clone + Owner,
    {
        if !self.is_initialized() {
            return Ok(None);
        }

        Account::try_from(self).map(Box::new).map(Some)
    }

    fn parse_optional_account_loader<T>(&'info self) -> Result<Option<AccountLoader<'info, T>>>
    where
        T: ZeroCopy + Owner,
    {
        if !self.is_initialized() {
            return Ok(None);
        }

        AccountLoader::try_from(self).map(Some)
    }

    fn to_option(&self) -> Option<&Self> {
        (self.key != &system_program::ID).then_some(self)
    }
}

pub trait AsAccountInfo<'info> {
    /// SAFETY: `info: &'info AccountInfo<'info>` field of `Account` type
    /// is returned by `AsRef::<AccountInfo<'info>>::as_ref()`, but due to
    /// the trait's signature, its lifetime('info) is narrowed down to `'a`.
    ///
    /// Therefore it is absolutely safe to restore `&'a AccountInfo<'info>`
    /// back to `&'info AccountInfo<'info>`.
    ///
    /// ```rs
    /// // account.rs
    /// pub struct Account<'info, T> {
    ///     account: T,
    ///     info: &'info AccountInfo<'info>,
    /// }
    ///
    /// impl<'info, T> AsRef<AccountInfo<'info>> for Account<'info, T> {
    ///     // lifetime of return value('info) is narrowed down to the
    ///     // lifetime of `self`('1) due to the method signature.
    ///     fn as_ref(&self) -> &AccountInfo<'info> {
    ///         self.info
    ///     }
    /// }
    /// ```
    fn as_account_info(&self) -> &'info AccountInfo<'info>;
}

impl<'info, T> AsAccountInfo<'info> for AccountLoader<'info, T>
where
    T: ZeroCopy + Owner + Clone,
{
    fn as_account_info(&self) -> &'info AccountInfo<'info> {
        unsafe { std::mem::transmute::<&AccountInfo, _>(self.as_ref()) }
    }
}

impl<'info, T> AsAccountInfo<'info> for Account<'info, T>
where
    T: AccountSerialize + AccountDeserialize + Clone,
{
    fn as_account_info(&self) -> &'info AccountInfo<'info> {
        unsafe { std::mem::transmute::<&AccountInfo, _>(self.as_ref()) }
    }
}

impl<'info, T> AsAccountInfo<'info> for InterfaceAccount<'info, T>
where
    T: AccountSerialize + AccountDeserialize + Clone,
{
    fn as_account_info(&self) -> &'info AccountInfo<'info> {
        unsafe { std::mem::transmute::<&AccountInfo, _>(self.as_ref()) }
    }
}

impl<'info, T> AsAccountInfo<'info> for Program<'info, T> {
    fn as_account_info(&self) -> &'info AccountInfo<'info> {
        unsafe { std::mem::transmute::<&AccountInfo, _>(self.as_ref()) }
    }
}

impl<'info, T> AsAccountInfo<'info> for Interface<'info, T> {
    fn as_account_info(&self) -> &'info AccountInfo<'info> {
        unsafe { std::mem::transmute::<&AccountInfo, _>(self.as_ref()) }
    }
}

pub trait SystemProgramExt<'info> {
    /// Need signer seeds of every PDAs.
    fn initialize_account(
        &self,
        account_to_initialize: &AccountInfo<'info>,
        payer: &AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]],
        space: usize,
        target_lamports: Option<u64>,
        owner: &Pubkey,
    ) -> Result<()>;

    /// System program can only transfer SOL from system accounts (i.e., wallet).
    /// To avoid such consideration, this method will do that for you.
    ///
    /// This method will directly move lamports when `from` account is owned by this program.
    /// You don't need `from_seeds` in this case.
    ///
    /// Otherwise it will call system program CPI.
    fn transfer_sol(
        &self,
        from: &AccountInfo<'info>,
        from_seeds: &[&[&[u8]]],
        to: &AccountInfo<'info>,
        lamports: u64,
    ) -> Result<()>;
}

impl<'info> SystemProgramExt<'info> for Program<'info, System> {
    fn initialize_account(
        &self,
        account_to_initialize: &AccountInfo<'info>,
        payer: &AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]],
        space: usize,
        target_lamports: Option<u64>,
        owner: &Pubkey,
    ) -> Result<()> {
        let rent = Rent::get()?;
        let target_lamports = target_lamports.unwrap_or_else(|| rent.minimum_balance(space));

        let current_lamports = account_to_initialize.lamports();
        if current_lamports == 0 {
            anchor_lang::system_program::create_account(
                CpiContext::new_with_signer(
                    self.to_account_info(),
                    anchor_lang::system_program::CreateAccount {
                        from: payer.to_account_info(),
                        to: account_to_initialize.clone(),
                    },
                    signer_seeds,
                ),
                target_lamports,
                space as u64,
                owner,
            )?;
        } else {
            require_keys_neq!(
                payer.key(),
                account_to_initialize.key(),
                ErrorCode::TryingToInitPayerAsProgramAccount
            );

            let required_lamports = target_lamports.saturating_sub(current_lamports);
            if required_lamports > 0 {
                anchor_lang::system_program::transfer(
                    CpiContext::new_with_signer(
                        self.to_account_info(),
                        anchor_lang::system_program::Transfer {
                            from: payer.to_account_info(),
                            to: account_to_initialize.clone(),
                        },
                        signer_seeds,
                    ),
                    required_lamports,
                )?;
            }
            anchor_lang::system_program::allocate(
                CpiContext::new_with_signer(
                    self.to_account_info(),
                    anchor_lang::system_program::Allocate {
                        account_to_allocate: account_to_initialize.clone(),
                    },
                    signer_seeds,
                ),
                space as u64,
            )?;
            anchor_lang::system_program::assign(
                CpiContext::new_with_signer(
                    self.to_account_info(),
                    anchor_lang::system_program::Assign {
                        account_to_assign: account_to_initialize.clone(),
                    },
                    signer_seeds,
                ),
                owner,
            )?;
        }

        Ok(())
    }

    fn transfer_sol(
        &self,
        from: &AccountInfo<'info>,
        from_seeds: &[&[&[u8]]],
        to: &AccountInfo<'info>,
        lamports: u64,
    ) -> Result<()> {
        // direct transfer
        if from.owner == &crate::ID {
            from.sub_lamports(lamports)?;
            to.add_lamports(lamports)?;

            return Ok(());
        }

        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                self.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                },
                from_seeds,
            ),
            lamports,
        )
    }
}

#[allow(unused)]
#[macro_use]
macro_rules! debug_msg_heap_size {
    ($marker:expr) => {
        #[allow(unexpected_cfgs)]
        {
            #[cfg(all(not(feature = "custom-heap"), target_os = "solana"))]
            {
                let pos = unsafe { *(crate::A.start as *mut usize) };
                let heap_top = crate::A.start + crate::A.len;
                let heap_usage = heap_top.saturating_sub(pos);
                msg!(
                    "HEAP#{} = {:?}bytes ({}%)",
                    $marker,
                    heap_usage,
                    (heap_usage * 100) as f32 / crate::A.len as f32,
                );
            }
        }
    };
}

#[allow(unused_imports)]
pub(crate) use debug_msg_heap_size;

/// Test utils.
#[cfg(test)]
pub mod tests {
    use std::{
        cell::{RefCell, RefMut},
        collections::HashMap,
    };

    use solana_sdk::account::Account;

    use super::*;

    /// Mocks `solana_account_db::accounts_db::AccountDb`.
    ///
    /// This type is useful for testing methods that requires `AccountInfo` value,
    /// but not solana runtime support, such as Sysvars or System Calls.
    ///
    /// Usage:
    ///
    /// ```ignore
    /// fn method_to_test(state: &Account<State>) -> Result<()> { /* ... */ }
    ///
    /// #[test]
    /// fn test_method() {
    ///     let key = Pubkey::find_program_address(/* ... */).0;
    ///     let state = State { /* ... */ };
    ///     let mut data = vec![];
    ///     state.try_serialize(&mut data).unwrap();
    ///     let lamports = 1000000;
    ///     let owner = crate::ID;
    ///     
    ///     let mut accounts = MockAccountsDb::default();
    ///     accounts.add_or_update_accounts(
    ///         key,
    ///         lamports,
    ///         data,
    ///         owner,
    ///         false,
    ///     );
    ///     accounts.run_with_accounts(
    ///         &[AccountMeta::new_readonly(key, false)],
    ///         |account_infos| {
    ///             let state = Account::try_from(&account_infos[0])?;
    ///             method_to_test(&state)
    ///         }
    ///     )
    ///     .expect("Method failed");
    /// }
    /// ```
    #[derive(Default)]
    pub struct MockAccountsDb(HashMap<Pubkey, RefCell<Account>>);

    impl MockAccountsDb {
        pub fn add_or_update_accounts(
            &mut self,
            key: Pubkey,
            lamports: u64,
            data: Vec<u8>,
            owner: Pubkey,
            executable: bool,
        ) {
            self.0.insert(
                key,
                RefCell::new(Account {
                    lamports,
                    data,
                    owner,
                    executable,
                    rent_epoch: u64::MAX,
                }),
            );
        }

        #[allow(clippy::expect_fun_call)]
        pub fn run_with_accounts<'a, F, R>(
            &self,
            account_metas: impl IntoIterator<Item = &'a AccountMeta>,
            f: F,
        ) -> R
        where
            F: for<'info> FnOnce(&'info [AccountInfo<'info>]) -> R,
        {
            let mut guards: Vec<(&Pubkey, RefMut<Account>, &'a AccountMeta)> = account_metas
                .into_iter()
                .map(|meta| {
                    let (key, account) = self
                        .0
                        .get_key_value(&meta.pubkey)
                        .expect(&format!("Account {:?} not in DB", meta.pubkey));

                    (key, account.borrow_mut(), meta)
                })
                .collect();

            let account_infos: Vec<AccountInfo> = guards
                .iter_mut()
                .map(|(key, guard, meta)| {
                    let Account {
                        lamports,
                        data,
                        owner,
                        executable,
                        rent_epoch,
                    } = &mut **guard;
                    AccountInfo::new(
                        key,
                        meta.is_signer,
                        meta.is_writable,
                        lamports,
                        data,
                        owner,
                        *executable,
                        *rent_epoch,
                    )
                })
                .collect();

            f(account_infos.as_slice())
        }
    }
}
