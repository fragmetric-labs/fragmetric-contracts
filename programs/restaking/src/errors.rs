use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("signature verification failed")]
    InvalidSignatureError,

    #[msg("token is not transferable currently")]
    TokenNotTransferableError,

    #[msg("token is not transferring currently")]
    TokenNotTransferringException,

    #[msg("calculation arithmetic exception")]
    CalculationArithmeticException,

    #[msg("decode invalid utf-8 format exception")]
    DecodeInvalidUtf8FormatException,

    #[msg("fund: cannot apply invalid update")]
    FundInvalidUpdateError,

    #[msg("fund: sol transfer failed")]
    FundSOLTransferFailedException,

    #[msg("fund: token transfer failed")]
    FundTokenTransferFailedException,

    #[msg("fund: already supported token")]
    FundAlreadySupportedTokenError,

    #[msg("fund: not supported the token")]
    FundNotSupportedTokenError,

    #[msg("fund: exceeded sol capacity amount")]
    FundExceededSOLCapacityAmountError,

    #[msg("fund: exceeded token capacity amount")]
    FundExceededTokenCapacityAmountError,

    #[msg("fund: exceeded max withdrawal request per user")]
    FundExceededMaxWithdrawalRequestError,

    #[msg("fund: operation reserved sol is exhausted")]
    FundOperationReservedSOLExhaustedException,

    #[msg("fund: withdrawal request not found")]
    FundWithdrawalRequestNotFoundError,

    #[msg("fund: withdrawal request not completed yet")]
    FundPendingWithdrawalRequestError,

    #[msg("fund: withdrawal reserved sol is exhausted")]
    FundWithdrawalReservedSOLExhaustedException,

    #[msg("fund: withdrawal is currently disabled")]
    FundWithdrawalDisabledError,

    #[msg("fund: withdrawal request is already in progress")]
    FundProcessingWithdrawalRequestError,

    #[msg("fund: token pricing source is not found")]
    FundTokenPricingSourceNotFoundException,

    #[msg("operator: job unmet threshold")]
    OperatorJobUnmetThresholdError,

    #[msg("reward: invalid token transfer args")]
    RewardInvalidTransferArgsException,

    #[msg("reward: invalid metadata name length")]
    RewardInvalidMetadataNameLengthError,

    #[msg("reward: invalid metadata description length")]
    RewardInvalidMetadataDescriptionLengthError,

    #[msg("reward: invalid reward type")]
    RewardInvalidRewardType,

    #[msg("reward: already existing holder")]
    RewardAlreadyExistingHolderError,

    #[msg("reward: already existing reward")]
    RewardAlreadyExistingRewardError,

    #[msg("reward: already existing pool")]
    RewardAlreadyExistingPoolError,

    #[msg("reward: holder not found")]
    RewardHolderNotFoundError,

    #[msg("reward: reward not found")]
    RewardNotFoundError,

    #[msg("reward: pool not found")]
    RewardPoolNotFoundError,

    #[msg("reward: user pool not found")]
    RewardUserPoolNotFoundError,

    #[msg("reward: pool is closed")]
    RewardPoolClosedError,

    #[msg("reward: invalid pool configuration")]
    RewardInvalidPoolConfigurationException,

    #[msg("reward: invalid reward pool access")]
    RewardInvalidPoolAccessException,

    #[msg("unmet account size reallocation")]
    AccountUnmetDesiredReallocSizeError,

    #[msg("reward: incorrect accounting exception")]
    RewardInvalidAccountingException,

    #[msg("reward: invalid amount or contribution accrual rate")]
    RewardInvalidAllocatedAmountDeltaException,

    #[msg("reward: exceeded max holders")]
    RewardExceededMaxHoldersException,

    #[msg("reward: exceeded max rewards")]
    RewardExceededMaxRewardsException,

    #[msg("reward: exceeded max reward pools")]
    RewardExceededMaxRewardPoolsException,

    #[msg("reward: exceeded max user reward pools")]
    RewardExceededMaxUserRewardPoolsException,

    #[msg("reward: exceeded max pubkeys per holder")]
    RewardExceededMaxHolderPubkeysException,

    #[msg("reward: exceeded max token allocated amount record")]
    RewardExceededMaxTokenAllocatedAmountRecordException,

    #[msg("reward: exceeded max reward settlements per pool")]
    RewardExceededMaxRewardSettlementException,

    #[msg("reward: stale settlement block not exist")]
    RewardStaleSettlementBlockNotExistError,

    #[msg("reward: invalid settlement block height")]
    RewardInvalidSettlementBlockHeightException,

    #[msg("reward: invalid settlement block contribution")]
    RewardInvalidSettlementBlockContributionException,

    #[msg("reward: sum of user settled amount cannot exceed total amount")]
    RewardInvalidTotalUserSettledAmountException,

    #[msg("reward: sum of user settled contribution cannot exceed total contribution")]
    RewardInvalidTotalUserSettledContributionException,

    #[msg("reward: cannot close the reward pool")]
    RewardPoolCloseConditionError,

    #[msg("fund: signature has expired")]
    FundDepositMetadataSignatureExpiredError,

    #[msg("invalid data account version")]
    InvalidDataVersionError,

    #[msg("fund: exceeded max batch withdrawals in progress")]
    FundExceededMaxBatchWithdrawalInProgressError,

    #[msg("fund: exceeded max supported tokens")]
    FundExceededMaxSupportedTokensError,

    #[msg("fund: invalid sol withdrawal fee rate")]
    FundInvalidSolWithdrawalFeeRateError,

    #[msg("fund: unexpected execution reserved account balance")]
    FundUnexpectedExecutionReservedAccountBalanceException,
}
