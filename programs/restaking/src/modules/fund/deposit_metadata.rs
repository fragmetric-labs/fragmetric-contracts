use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::ed25519;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DepositMetadata {
    wallet_provider: String,
    contribution_accrual_rate: u8, // 100 is 1.0
    expired_at: i64,
}

impl DepositMetadata {
    pub(super) fn verify(self, instructions_sysvar: &AccountInfo, current_timestamp: i64) -> Result<(String, u8)> {

        ed25519::verify_preceding_ed25519_instruction(
            instructions_sysvar,
            self.try_to_vec()?.as_slice(),
        )?;

        require_gte!(
            self.expired_at,
            current_timestamp,
            ErrorCode::FundDepositMetadataSignatureExpiredError,
        );

        Ok((self.wallet_provider, self.contribution_accrual_rate))
    }
}
