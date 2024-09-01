use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("signature verification failed")]
    SignatureVerificationFailed,

    #[msg("token is not transferable currently")]
    TokenNotTransferable,

    #[msg("token is not transferring currently")]
    TokenNotTransferring,

    #[msg("calculation arithmetic exception")]
    CalculationArithmeticException,

    #[msg("fund: cannot apply invalid update")]
    FundInvalidUpdate,

    #[msg("fund: sol transfer failed")]
    FundSOLTransferFailed,

    #[msg("fund: token transfer failed")]
    FundTokenTransferFailed,

    #[msg("fund: already supported token")]
    FundAlreadySupportedToken,

    #[msg("fund: not supported the token")]
    FundNotSupportedToken,

    #[msg("fund: exceeded sol capacity amount")]
    FundExceededSOLCapacityAmount,

    #[msg("fund: exceeded token capacity amount")]
    FundExceededTokenCapacityAmount,

    #[msg("fund: exceeded max withdrawal request per user")]
    FundExceededMaxWithdrawalRequestSizePerUser,

    #[msg("fund: operation reserved sol is exhausted")]
    FundOperationReservedSOLExhausted,

    #[msg("fund: withdrawal request not found")]
    FundWithdrawalRequestNotFound,

    #[msg("fund: withdrawal request not completed yet")]
    FundWithdrawalNotCompletedYet,

    #[msg("fund: withdrawal reserved sol is exhausted")]
    FundWithdrawalReservedSOLExhausted,

    #[msg("fund: withdrawal is currently disabled")]
    FundWithdrawalDisabled,

    #[msg("fund: withdrawal request is already in progress")]
    FundWithdrawalRequestAlreadyInProgress,

    #[msg("fund: token pricing source is not found")]
    FundTokenPricingSourceNotFound,

    #[msg("operator: job unmet threshold")]
    OperatorJobUnmetThreshold,

    #[msg("reward: invalid token transfer args")]
    RewardInvalidTransferArgs,

    #[msg("reward: already existing holder")]
    RewardAlreadyExistingHolder,

    #[msg("reward: already existing reward")]
    RewardAlreadyExistingReward,

    #[msg("reward: already existing pool")]
    RewardAlreadyExistingPool,

    #[msg("reward: pool not found")]
    RewardPoolNotFound,

    #[msg("reward: pool is closed")]
    RewardPoolClosed,

    #[msg("reward: invalid pool configuration")]
    RewardInvalidPoolConfiguration,

    #[msg("reward: invalid reward pool access")]
    RewardInvalidPoolAccess,

    #[msg("reward: unmet account size reallocation")]
    RewardUnmetAccountRealloc,

    #[msg("reward: incorrect accounting exception")]
    RewardAccountingException,

    #[msg("reward: invalid amount or contribution accrual rate")]
    RewardInvalidAllocatedAmountDelta,

    #[msg("reward: stale settlement block not exist")]
    RewardStaleSettlementBlockNotExist,

    #[msg("reward: invalid settlement block height")]
    RewardInvalidSettlementBlockHeight,

    #[msg("reward: invalid settlement block contribution")]
    RewardInvalidSettlementBlockContribution,

    #[msg("reward: sum of user settled amount cannot exceed total amount")]
    RewardInvalidTotalUserSettledAmount,

    #[msg("reward: sum of user settled contribution cannot exceed total contribution")]
    RewardInvalidTotalUserSettledContribution,
}
