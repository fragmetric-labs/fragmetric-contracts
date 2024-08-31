use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
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
    #[msg("Exceeds the sol cap")]
    FundExceedsSolCap,
    #[msg("Exceeds the token cap")]
    FundExceedsTokenCap,
    #[msg("Exceeds max withdrawal request")]
    FundExceedsMaxWithdrawalRequestSize,
    #[msg("Batch withdrwal amount exceeds SOL amount in fund")]
    FundWithdrawalRequestExceedsSOLAmountsInTemp,
    #[msg("Operator unmet threshold")]
    OperatorUnmetThreshold,
    #[msg("Token is not currently transferring")]
    TokenNotCurrentlyTransferring,
    #[msg("Invalid token transfer args")]
    TokenInvalidTransferArgs,
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
    #[msg("Signature verification failed")]
    SigVerificationFailed,
    #[msg("Calculation failed due to overflow/underflow")]
    CalculationFailure,
    #[msg("Token pricing source not provided")]
    FundTokenPricingSourceNotFound,
    #[msg("Invalid reward type")]
    RewardInvalidRewardType,
    #[msg("Already existing reward pool")]
    RewardAlreadyExistingPool,
    #[msg("Reward pool not found")]
    RewardPoolNotFound,
    #[msg("Reward pool is already closed")]
    RewardPoolAlreadyClosed,
    #[msg("Invalid reward pool configuration")]
    RewardInvalidPoolConfiguration,
    #[msg("Invalid accounting")]
    RewardInvalidAccounting,
    #[msg("Invalid amount or contribution accrual rate")]
    RewardInvalidAllocatedAmountDelta,
    #[msg("Cannot find stale settlement block")]
    RewardStaleSettlementBlockDoesNotExist,
    #[msg("Invalid settlement block height")]
    RewardInvalidSettlementBlockHeight,
    #[msg("Invalid settlement block contribution")]
    RewardInvalidSettlementBlockContribution,
    #[msg("Sum of user settled amount cannot exceed total amount")]
    RewardInvalidTotalUserSettledAmount,
    #[msg("Sum of user settled contribution cannot exceed total contribution")]
    RewardInvalidTotalUserSettledContribution,
}
