use anchor_lang::prelude::*;

use crate::errors::ErrorCode;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DepositMetadata {
    pub wallet_provider: String,
    pub contribution_accrual_rate: u8, // 100 is 1.0
    pub expired_at: i64,
}

impl DepositMetadata {
    pub fn verify_expiration(&self, current_time: i64) -> Result<()> {
        require_gte!(
            self.expired_at,
            current_time,
            ErrorCode::FundDepositMetadataSignatureExpiredError,
        );

        Ok(())
    }
}
