use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::ed25519;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct DepositMetadata {
    user: Pubkey,
    wallet_provider: String,
    contribution_accrual_rate: u16, // 100 is 1.0
    expired_at: i64,
}

impl DepositMetadata {
    pub(super) fn verify(
        self,
        instructions_sysvar: &AccountInfo,
        payload_signer_key: &Pubkey,
        user_key: &Pubkey,
        current_timestamp: i64,
    ) -> Result<(String, u16)> {
        ed25519::SignatureVerificationService::verify(
            instructions_sysvar,
            self.try_to_vec()?.as_slice(),
            payload_signer_key,
        )?;

        require_gte!(
            self.expired_at,
            current_timestamp,
            ErrorCode::FundDepositMetadataSignatureExpiredError,
        );

        require_keys_eq!(*user_key, self.user);

        Ok((self.wallet_provider, self.contribution_accrual_rate))
    }
}
