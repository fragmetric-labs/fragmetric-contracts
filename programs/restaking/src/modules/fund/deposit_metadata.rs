use anchor_lang::prelude::*;

use crate::errors::ErrorCode;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DepositMetadata {
    pub wallet_provider: String,
    pub contribution_accrual_rate: u8, // 100 is 1.0
    pub expired_at: i64,
}

impl DepositMetadata {
    pub fn verify_expiration(&self) -> Result<()> {
        let current_timestamp = crate::utils::timestamp_now()?;

        if current_timestamp > self.expired_at {
            err!(ErrorCode::FundDepositMetadataSignatureExpiredError)?
        }

        Ok(())
    }
}
