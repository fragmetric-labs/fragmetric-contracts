use anchor_lang::{prelude::*, solana_program, CheckOwner, ZeroCopy};

pub trait PDASeeds<const N: usize> {
    const SEED: &'static [u8];

    fn get_seeds(&self) -> [&[u8]; N];
    fn get_bump_ref(&self) -> &u8;

    fn get_signer_seeds(&self) -> Vec<&[u8]> {
        let mut signer_seeds = self.get_seeds().to_vec();
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
    /// [`MAX_PERMITTED_DATA_INCREASE`]: solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE
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

            let max_increase = solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
            let increase = std::cmp::min(required_realloc_size, max_increase);
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
pub fn get_proportional_amount(amount: u64, numerator: u64, denominator: u64) -> Option<u64> {
    if numerator == 0 && denominator == 0 {
        return Some(amount);
    }

    u64::try_from(
        (amount as u128)
            .checked_mul(numerator as u128)?
            .checked_div(denominator as u128)?,
    )
    .ok()
}

#[inline(never)]
pub fn parse_account_boxed<'info, T>(
    account: &'info AccountInfo<'info>,
) -> Result<Box<Account<'info, T>>>
where
    T: AccountSerialize + AccountDeserialize + Clone + Owner,
{
    Account::try_from(account).map(Box::new)
}

#[inline(never)]
pub fn parse_interface_account_boxed<'info, T>(
    account: &'info AccountInfo<'info>,
) -> Result<Box<InterfaceAccount<'info, T>>>
where
    T: AccountSerialize + AccountDeserialize + Clone + CheckOwner,
{
    InterfaceAccount::try_from(account).map(Box::new)
}
