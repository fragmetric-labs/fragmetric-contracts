use anchor_lang::prelude::*;

use crate::fund::*;

impl WithdrawalStatus {
    /// 1 fee rate = 1bps = 0.01%
    pub(super) const WITHDRAWAL_FEE_RATE_DIVISOR: u64 = 10_000;

    fn withdrawal_fee_rate_f32(&self) -> f32 {
        self.sol_withdrawal_fee_rate as f32 / (Self::WITHDRAWAL_FEE_RATE_DIVISOR / 100) as f32
    }
}

impl FundV2 {
    pub(super) fn to_info(&self, admin: Pubkey, receipt_token_mint: Pubkey) -> FundInfo {
        FundInfo {
            admin,
            lrt_mint: receipt_token_mint,
            supported_tokens: self.whitelisted_tokens.clone(),
            sol_amount_in: self.sol_amount_in,
            sol_reserved_amount: self.withdrawal_status.reserved_fund.sol_remaining,
            sol_withdrawal_fee_rate: self.withdrawal_status.withdrawal_fee_rate_f32(),
            sol_withdrawal_enabled: self.withdrawal_status.withdrawal_enabled_flag,
        }
    }
}
