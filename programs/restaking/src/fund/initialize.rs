use anchor_lang::prelude::*;

use crate::fund::*;

impl Fund {
    pub(super) fn initialize_if_needed(&mut self, bump: u8, receipt_token_mint: Pubkey) {
        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
            self.supported_tokens = vec![];
            self.sol_amount_in = 0;
            self.withdrawal_status = Default::default();
        }
    }
}

impl WithdrawalStatus {
    pub(super) fn set_sol_withdrawal_fee_rate(&mut self, sol_withdrawal_fee_rate: u16) {
        self.sol_withdrawal_fee_rate = sol_withdrawal_fee_rate;
    }

    pub(super) fn set_withdrawal_enabled_flag(&mut self, flag: bool) {
        self.withdrawal_enabled_flag = flag;
    }

    pub(super) fn set_batch_processing_threshold(
        &mut self,
        amount: Option<u64>,
        duration: Option<i64>,
    ) {
        if let Some(amount) = amount {
            self.batch_processing_threshold_amount = amount;
        }
        if let Some(duration) = duration {
            self.batch_processing_threshold_duration = duration;
        }
    }
}

impl ReceiptTokenLockAuthority {
    pub(super) fn initialize_if_needed(&mut self, bump: u8, receipt_token_mint: Pubkey) {
        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
        }
    }
}

impl SupportedTokenAuthority {
    pub(super) fn initialize_if_needed(
        &mut self,
        bump: u8,
        receipt_token_mint: Pubkey,
        token_mint: Pubkey,
    ) {
        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
            self.token_mint = token_mint;
        }
    }
}

impl ReceiptTokenMintAuthority {
    pub(super) fn initialize_if_needed(&mut self, bump: u8, receipt_token_mint: Pubkey) {
        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
        }
    }
}

impl UserReceipt {
    pub(super) fn initialize_if_needed(
        &mut self,
        bump: u8,
        user: Pubkey,
        receipt_token_mint: Pubkey,
    ) {
        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.user = user;
            self.receipt_token_mint = receipt_token_mint;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize() {
        let receipt_token_mint = Pubkey::new_unique();
        let sol_withdrawal_fee_rate = 100;
        let withdrawal_enabled_flag = false;
        let batch_processing_threshold_amount = 10;
        let batch_processing_threshold_duration = 10;

        let mut fund = Fund {
            data_version: 1,
            bump: 0,
            receipt_token_mint: Pubkey::default(),
            supported_tokens: vec![],
            sol_amount_in: 0,
            withdrawal_status: Default::default(),
        };

        fund.initialize_if_needed(0, receipt_token_mint);

        fund.withdrawal_status
            .set_sol_withdrawal_fee_rate(sol_withdrawal_fee_rate);
        fund.withdrawal_status
            .set_withdrawal_enabled_flag(withdrawal_enabled_flag);
        fund.withdrawal_status.set_batch_processing_threshold(
            Some(batch_processing_threshold_amount),
            Some(batch_processing_threshold_duration),
        );

        assert_eq!(fund.receipt_token_mint, receipt_token_mint);
        assert_eq!(
            fund.withdrawal_status.sol_withdrawal_fee_rate,
            sol_withdrawal_fee_rate
        );
        assert_eq!(
            fund.withdrawal_status.withdrawal_enabled_flag,
            withdrawal_enabled_flag
        );
        assert_eq!(
            fund.withdrawal_status.batch_processing_threshold_amount,
            batch_processing_threshold_amount
        );
        assert_eq!(
            fund.withdrawal_status.batch_processing_threshold_duration,
            batch_processing_threshold_duration
        );
        assert_eq!(fund.withdrawal_status.pending_batch_withdrawal.batch_id, 1);
    }
}
