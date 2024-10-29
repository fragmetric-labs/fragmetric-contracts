use anchor_lang::prelude::*;

use crate::errors::ErrorCode;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DepositMetadata {
    wallet_provider: String,
    contribution_accrual_rate: u8, // 100 is 1.0
    expired_at: i64,
}

impl DepositMetadata {
    pub(super) fn verify_expiration(&self, current_timestamp: i64) -> Result<()> {
        require_gte!(
            self.expired_at,
            current_timestamp,
            ErrorCode::FundDepositMetadataSignatureExpiredError,
        );

        Ok(())
    }

    /// Returns (wallet_provider, contribution_accrual_rate)
    pub(super) fn split(self) -> (String, u8) {
        (self.wallet_provider, self.contribution_accrual_rate)
    }
}
