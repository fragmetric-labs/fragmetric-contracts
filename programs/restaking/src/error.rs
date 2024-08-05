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
    #[msg("Batch withdrwal amount exceeds SOL amount in fund")]
    FundWithdrawalRequestExceedsSOLAmountsInTemp,
    // receipt_token_extensions
    #[msg("Token is not currently transferring")]
    TokenNotCurrentlyTransferring,
    #[msg("Withdrawal request not found")]
    FundWithdrawalRequestNotFound,
    #[msg("Withdrawal request not completed")]
    FundWithdrawalNotCompleted,
    #[msg("Not enough reserved Sol")]
    FundNotEnoughReservedSol,
    #[msg("Withdrawal is currently disabled")]
    FundWithdrawalDisabled,
    #[msg("Withdrawal request already started processing")]
    FundWithdrawalAlreadyInProgress,
}
