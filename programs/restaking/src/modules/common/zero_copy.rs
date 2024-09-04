use anchor_lang::{prelude::*, ZeroCopy};

/// Zero-copy account that has header (data-version, bump).
pub trait ZeroCopyHeader: ZeroCopy + Owner {
    /// Offset of bump (8bit)
    fn bump_offset() -> usize;
}

/// A trait for account loader to perform operation without load.
pub trait ZeroCopyWithoutLoad {
    /// Checks if we are initializing this account and sets bump, without loading.
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

        // Sets bump.
        let offset = 8 + T::bump_offset();
        data[offset] = bump;

        Ok(())
    }

    // fn data_version(&self) -> Result<u16> {
    //     let data = self.as_ref().try_borrow_data()?;
    //     let mut disc_bytes = [0u8; 2];
    //     let offset = 8 + T::data_version_offset();
    //     disc_bytes.copy_from_slice(&data[offset..offset + 2]);
    //     Ok(u16::from_le_bytes(disc_bytes))
    // }

    fn bump(&self) -> Result<u8> {
        let data = self.as_ref().try_borrow_data()?;
        let offset = 8 + T::bump_offset();
        Ok(data[offset])
    }
}
