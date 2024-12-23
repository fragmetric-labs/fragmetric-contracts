use anchor_lang::prelude::*;
use anchor_lang::solana_program::{entrypoint, system_program};
use anchor_lang::{CheckId, CheckOwner, ZeroCopy};

pub trait PDASeeds<const N: usize> {
    const SEED: &'static [u8];

    fn get_seed_phrase(&self) -> [&[u8]; N];
    fn get_bump_ref(&self) -> &u8;

    fn get_seeds(&self) -> Vec<&[u8]> {
        let mut signer_seeds = self.get_seed_phrase().to_vec();
        signer_seeds.push(std::slice::from_ref(self.get_bump_ref()));
        signer_seeds
    }

    fn get_bump(&self) -> u8 {
        *self.get_bump_ref()
    }
}

/// Zero-copy account that has header (data-version, bump).
pub trait ZeroCopyHeader: ZeroCopy {
    /// Offset of bump (8bit)
    fn get_bump_offset() -> usize;
}

/// An extension trait for [AccountLoader].
pub trait AccountLoaderExt<'info> {
    /// Sets zero-copy header when initializing the account
    /// without enough account data size provided.
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
/// when both numerator and denominator are zero, returns amount.
pub fn get_proportional_amount(amount: u64, numerator: u64, denominator: u64) -> Result<u64> {
    if numerator == denominator {
        return Ok(amount);
    }

    (amount as u128)
        .checked_mul(numerator as u128)
        .and_then(|v| v.checked_div(denominator as u128))
        .and_then(|v| u64::try_from(v).ok())
        .ok_or_else(|| error!(crate::errors::ErrorCode::CalculationArithmeticException))
}

pub trait AccountInfoExt<'info> {
    fn is_initialized(&self) -> bool;

    fn parse_account_boxed<T>(&'info self) -> Result<Box<Account<'info, T>>>
    where
        T: AccountSerialize + AccountDeserialize + Clone + Owner;

    fn parse_interface_account_boxed<T>(&'info self) -> Result<Box<InterfaceAccount<'info, T>>>
    where
        T: AccountSerialize + AccountDeserialize + Clone + CheckOwner;

    fn parse_program_boxed<T>(&'info self) -> Result<Box<Program<'info, T>>>
    where
        T: Id;

    fn parse_interface_boxed<T>(&'info self) -> Result<Box<Interface<'info, T>>>
    where
        T: CheckId;

    fn parse_optional_account_boxed<T>(&'info self) -> Result<Option<Box<Account<'info, T>>>>
    where
        T: AccountSerialize + AccountDeserialize + Clone + Owner;

    fn parse_optional_account_loader<T>(&'info self) -> Result<Option<AccountLoader<'info, T>>>
    where
        T: ZeroCopy + Owner;

    fn to_option(&self) -> Option<&Self>;
}

impl<'info> AccountInfoExt<'info> for AccountInfo<'info> {
    fn is_initialized(&self) -> bool {
        self.lamports() != 0 || self.owner != &system_program::ID
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
    fn parse_program_boxed<T>(&'info self) -> Result<Box<Program<'info, T>>>
    where
        T: Id,
    {
        Program::try_from(self).map(Box::new)
    }

    #[inline(never)]
    fn parse_interface_boxed<T>(&'info self) -> Result<Box<Interface<'info, T>>>
    where
        T: CheckId,
    {
        Interface::try_from(self).map(Box::new)
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

    /// treat zeroed pubkey as None
    fn to_option(&self) -> Option<&Self> {
        if self.key().to_bytes().iter().all(|b| *b == 0) {
            None
        } else {
            Some(self)
        }
    }
}

pub trait AccountExt<'info> {
    /// SAFETY: `info: &'info AccountInfo<'info>` field of `Account` type
    /// is returned by `AsRef::<AccountInfo<'info>>::as_ref()`, but due to
    /// the trait's signature, its lifetime('info) is narrowed down to `'a`.
    ///
    /// Therefore it is absolutely safe to restore `&'a AccountInfo<'info>`
    /// back to `&'info AccountInfo<'info>`.
    ///
    /// ```rs
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

impl<'info, T> AccountExt<'info> for AccountLoader<'info, T>
where
    T: ZeroCopy + Owner + Clone,
{
    fn as_account_info(&self) -> &'info AccountInfo<'info> {
        unsafe { std::mem::transmute::<&AccountInfo, _>(self.as_ref()) }
    }
}

impl<'info, T> AccountExt<'info> for Account<'info, T>
where
    T: AccountSerialize + AccountDeserialize + Clone,
{
    fn as_account_info(&self) -> &'info AccountInfo<'info> {
        unsafe { std::mem::transmute::<&AccountInfo, _>(self.as_ref()) }
    }
}

impl<'info, T> AccountExt<'info> for InterfaceAccount<'info, T>
where
    T: AccountSerialize + AccountDeserialize + Clone,
{
    fn as_account_info(&self) -> &'info AccountInfo<'info> {
        unsafe { std::mem::transmute::<&AccountInfo, _>(self.as_ref()) }
    }
}

impl<'info, T> AccountExt<'info> for Program<'info, T> {
    fn as_account_info(&self) -> &'info AccountInfo<'info> {
        unsafe { std::mem::transmute::<&AccountInfo, _>(self.as_ref()) }
    }
}

impl<'info, T> AccountExt<'info> for Interface<'info, T> {
    fn as_account_info(&self) -> &'info AccountInfo<'info> {
        unsafe { std::mem::transmute::<&AccountInfo, _>(self.as_ref()) }
    }
}

pub trait SystemProgramExt<'info> {
    fn create_account(
        &self,
        account_to_create: &AccountInfo<'info>,
        account_to_create_seeds: &[&[u8]],
        payer: &(impl ToAccountInfo<'info> + Key),
        payer_seeds: &[&[u8]],
        space: usize,
        owner: &Pubkey,
    ) -> Result<()>;
}

impl<'info> SystemProgramExt<'info> for Program<'info, System> {
    fn create_account(
        &self,
        account_to_create: &AccountInfo<'info>,
        account_to_create_seeds: &[&[u8]],
        payer: &(impl ToAccountInfo<'info> + Key),
        payer_seeds: &[&[u8]],
        space: usize,
        owner: &Pubkey,
    ) -> Result<()> {
        let rent = Rent::get()?;
        let current_lamports = account_to_create.lamports();
        if current_lamports == 0 {
            anchor_lang::system_program::create_account(
                CpiContext::new_with_signer(
                    self.to_account_info(),
                    anchor_lang::system_program::CreateAccount {
                        from: payer.to_account_info(),
                        to: account_to_create.clone(),
                    },
                    &[payer_seeds, account_to_create_seeds],
                ),
                rent.minimum_balance(space),
                space as u64,
                owner,
            )?;
        } else {
            require_keys_neq!(
                payer.key(),
                account_to_create.key(),
                ErrorCode::TryingToInitPayerAsProgramAccount
            );

            let required_lamports = rent
                .minimum_balance(space)
                .max(1)
                .saturating_sub(current_lamports);
            if required_lamports > 0 {
                anchor_lang::system_program::transfer(
                    CpiContext::new_with_signer(
                        self.to_account_info(),
                        anchor_lang::system_program::Transfer {
                            from: payer.to_account_info(),
                            to: account_to_create.clone(),
                        },
                        &[payer_seeds],
                    ),
                    required_lamports,
                )?;
            }
            anchor_lang::system_program::allocate(
                CpiContext::new_with_signer(
                    self.to_account_info(),
                    anchor_lang::system_program::Allocate {
                        account_to_allocate: account_to_create.clone(),
                    },
                    &[account_to_create_seeds],
                ),
                space as u64,
            )?;
            anchor_lang::system_program::assign(
                CpiContext::new_with_signer(
                    self.to_account_info(),
                    anchor_lang::system_program::Assign {
                        account_to_assign: account_to_create.clone(),
                    },
                    &[account_to_create_seeds],
                ),
                owner,
            )?;
        }

        Ok(())
    }
}

#[allow(unused)]
pub fn debug_msg_heap_size(marker: &str) {
    let heap_total = entrypoint::HEAP_LENGTH as u64;
    let heap_top = heap_total + entrypoint::HEAP_START_ADDRESS;
    #[allow(unused_allocation)]
    let heap_usage: u64 = heap_top.saturating_sub(Box::new(0u8).as_ref() as *const u8 as u64);
    msg!(
        "HEAP#{} = {:?}bytes ({}%)",
        marker,
        heap_usage,
        (heap_usage * 100) as f32 / heap_total as f32
    );
}
