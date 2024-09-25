use anchor_lang::{prelude::*, ZeroCopy};

/// Zero-copy account that has header (data-version, bump).
pub trait ZeroCopyHeader: ZeroCopy + Owner {
    // /// Offset of data_version (16bit)
    // fn data_version_offset() -> usize;
    /// Offset of bump (8bit)
    fn bump_offset() -> usize;
}

/// A trait for account loader to perform operation without load.
pub trait ZeroCopyWithoutLoad {
    /// Checks if we are initializing this account and then sets bump, without loading.
    /// Should only be called once, when the account is being initialized.
    fn init_without_load(&mut self, bump: u8) -> Result<()>;
    // /// Reads data version without loading.
    // fn data_version(&self) -> Result<u16>;
    /// Reads bump without loading.
    fn bump(&self) -> Result<u8>;
}

impl<'info, T: ZeroCopyHeader> ZeroCopyWithoutLoad for AccountLoader<'info, T> {
    fn init_without_load(&mut self, bump: u8) -> Result<()> {
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

        // Safety Check
        let offset = 8 + T::bump_offset();
        if data.len() < offset + 1 {
            return Err(ErrorCode::ConstraintSpace.into());
        }

        // Sets bump.
        data[offset] = bump;

        Ok(())
    }

    // fn data_version(&self) -> Result<u16> {
    //     let data = self.as_ref().try_borrow_data()?;
    //     let offset = 8 + T::data_version_offset();

    //     // Safety Check
    //     if data.len() < offset + 2 {
    //         return Err(ErrorCode::ConstraintSpace.into());
    //     }

    //     let mut disc_bytes = [0u8; 2];
    //     disc_bytes.copy_from_slice(&data[offset..offset + 2]);
    //     Ok(u16::from_le_bytes(disc_bytes))
    // }

    fn bump(&self) -> Result<u8> {
        let data = self.as_ref().try_borrow_data()?;
        let offset = 8 + T::bump_offset();

        // Safety Check
        if data.len() < offset + 1 {
            return Err(ErrorCode::ConstraintSpace.into());
        }

        Ok(data[offset])
    }
}

/// A trait for zero-copy account to realloc size.
/// It cannot reduce the size.
pub trait ZeroCopyAccountRealloc<'info> {
    fn expand_account_size_if_needed(
        &self,
        payer: &Signer<'info>,
        system_program: &Program<'info, System>,
        desired_account_size: Option<u32>,
        initialize: bool,
    ) -> Result<()>;
}

impl<'info, T: ZeroCopyHeader> ZeroCopyAccountRealloc<'info> for AccountLoader<'info, T> {
    fn expand_account_size_if_needed(
        &self,
        payer: &Signer<'info>,
        system_program: &Program<'info, System>,
        desired_account_size: Option<u32>,
        initialize: bool,
    ) -> Result<()> {
        let account_info = self.as_ref();

        let current_account_size = account_info.data_len();
        let min_account_size = 8 + std::mem::size_of::<T>();
        let target_account_size = desired_account_size
            .map(|desired_size| std::cmp::max(desired_size as usize, min_account_size))
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

            let max_increase = anchor_lang::solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
            let increase = std::cmp::min(required_realloc_size, max_increase);
            let new_account_size = current_account_size + increase;
            if new_account_size < target_account_size && initialize {
                return Err(crate::errors::ErrorCode::AccountUnmetDesiredReallocSizeError)?;
            }

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
