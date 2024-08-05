use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // fund
    #[msg("SOL transfer failed")]
    FundSOLTransferFailed,
    #[msg("Token transfer failed")]
    FundTokenTransferFailed,
    #[msg("Already existing token")]
    FundAlreadyExistingToken,
    #[msg("Not existing token")]
    FundNotExistingToken,
    #[msg("Duplicated tokens in the list")]
    FundDuplicatedToken,
    #[msg("Exceeds the token cap")]
    FundExceedsTokenCap,
    // receipt_token_extensions
    #[msg("Token is not currently transferring")]
    TokenNotCurrentlyTransferring,
    #[msg("Receipt token lock failed")]
    FundReceiptTokenLockFailed,
    #[msg("Withdrawal request not found")]
    FundWithdrawalRequestNotFound,
    #[msg("Withdrawal request not completed")]
    FundWithdrawlNotCompleted,
    #[msg("Not enough reserved Sol")]
    FundNotEnoughReservedSol,

    // Operator
    #[msg("Batch unmet threshold")]
    OperatorUnmetThreshold,
}
