use anchor_lang::prelude::*;

#[event]
pub struct SolvManagerCompletedWithdrawalRequests {
    pub vault: Pubkey,
    pub solv_protocol_wallet: Pubkey,
    pub solv_manager: Pubkey,

    pub vault_supported_token_mint: Pubkey,
    pub vault_receipt_token_mint: Pubkey,
    pub solv_receipt_token_mint: Pubkey,

    pub burnt_srt_amount: u64,
    /// VST received for burning SRT
    pub received_vst_amount: u64,
    /// Estimated amount minus fee
    pub withdrawn_vst_amount: u64,
    /// Extra amount if exists
    pub extra_vst_amount: u64,
    /// Withdrawal fee
    pub deducted_vst_fee_amount: u64,
    /// Total claimable amount
    pub total_claimable_vst_amount: u64,
}
