use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("calculation arithmetic exception")]
    CalculationArithmeticException,

    #[msg("index out of bounds exception")]
    IndexOutOfBoundsException,

    #[msg("utf-8 decoding exception")]
    UTF8DecodingException,

    #[msg("signature verification failed")]
    InvalidSignatureError,

    #[msg("invalid account data version")]
    InvalidAccountDataVersionError,

    #[msg("token is not transferable currently")]
    TokenNotTransferableError,

    #[msg("token is not transferring currently")]
    TokenNotTransferringException,

    #[msg("reward: invalid token transfer args")]
    RewardInvalidTransferArgsException,

    #[msg("reward: invalid metadata name length")]
    RewardInvalidMetadataNameLengthError,

    #[msg("reward: invalid metadata description length")]
    RewardInvalidMetadataDescriptionLengthError,

    #[msg("reward: reward is already claimable")]
    RewardAlreadyClaimableError,

    #[msg("reward: not enough rewards to claim")]
    RewardNotEnoughRewardsToClaimError,

    #[msg("reward: already existing reward")]
    RewardAlreadyExistingRewardError,

    #[msg("reward: settlement not found")]
    RewardSettlementNotFoundError,

    #[msg("reward: reward not found")]
    RewardNotFoundError,

    #[msg("reward: pool not found")]
    RewardPoolNotFoundError,

    #[msg("reward: user pool not found")]
    RewardUserPoolNotFoundError,

    #[msg("reward: reward is not claimable")]
    RewardNotClaimableError,

    // deprecated
    #[msg("reward: invalid pool configuration")]
    RewardInvalidPoolConfigurationException,

    // deprecated
    #[msg("reward: invalid reward pool access")]
    RewardInvalidPoolAccessException,

    #[msg("reward: incorrect accounting exception")]
    RewardInvalidAccountingException,

    #[msg("reward: invalid amount or contribution accrual rate")]
    RewardInvalidAllocatedAmountDeltaException,

    // deprecated
    #[msg("reward: exceeded max holders")]
    RewardExceededMaxHoldersError,

    #[msg("reward: exceeded max rewards")]
    RewardExceededMaxRewardsError,

    #[msg("reward: exceeded max reward pools")]
    RewardExceededMaxRewardPoolsError,

    #[msg("reward: exceeded max user reward pools")]
    RewardExceededMaxUserRewardPoolsError,

    // deprecated
    #[msg("reward: exceeded max pubkeys per holder")]
    RewardExceededMaxHolderPubkeysError,

    #[msg("reward: exceeded max token allocated amount record")]
    RewardExceededMaxTokenAllocatedAmountRecordException,

    #[msg("reward: exceeded max reward settlements per pool")]
    RewardExceededMaxRewardSettlementError,

    #[msg("reward: exceeded max reward settlement block per settlement")]
    RewardExceededMaxRewardSettlementBlockError,

    #[msg("reward: invalid settlement block height")]
    RewardInvalidSettlementBlockHeightException,

    #[msg("reward: invalid settlement block contribution")]
    RewardInvalidSettlementBlockContributionException,

    #[msg("reward: sum of user settled amount cannot exceed total amount")]
    RewardInvalidTotalUserSettledAmountException,

    #[msg("reward: sum of user settled contribution cannot exceed total contribution")]
    RewardInvalidTotalUserSettledContributionException,

    // deprecated
    #[msg("reward: cannot close the reward pool")]
    RewardPoolCloseConditionError,

    #[msg("pricing: token pricing source is not found")]
    TokenPricingSourceAccountNotFoundError,

    #[msg("fund: cannot apply invalid configuration update")]
    FundInvalidConfigurationUpdateError,

    #[msg("fund: already supported token")]
    FundAlreadySupportedTokenError,

    #[msg("fund: not supported token")]
    FundNotSupportedTokenError,

    #[msg("fund: deposit is currently disabled")]
    FundDepositDisabledError,

    #[msg("fund: exceeded deposit capacity amount")]
    FundExceededDepositCapacityAmountError,

    #[msg("fund: deposit is not supported for the given asset")]
    FundDepositNotSupportedAsset,

    #[msg("fund: exceeded max withdrawal request per user")]
    FundExceededMaxWithdrawalRequestError,

    #[msg("fund: withdrawal request not found")]
    FundWithdrawalRequestNotFoundError,

    #[msg("fund: withdrawal request not belongs to the given batch")]
    FundWithdrawalRequestIncorrectBatchError,

    #[msg("fund: withdrawal is currently disabled")]
    FundWithdrawalDisabledError,

    #[msg("fund: withdrawal is not supported for the given asset")]
    FundWithdrawalNotSupportedAsset,

    #[msg("fund: withdrawal reserve is exhausted for the given asset")]
    FundWithdrawalReserveExhaustedSupportedAsset,

    #[msg("fund: withdrawal request is already in progress")]
    FundWithdrawalRequestAlreadyQueuedError,

    #[msg("fund: deposit metadata signature has expired")]
    FundDepositMetadataSignatureExpiredError,

    #[msg("fund: exceeded max supported tokens")]
    FundExceededMaxSupportedTokensError,

    #[msg("fund: invalid withdrawal fee rate")]
    FundInvalidWithdrawalFeeRateError,

    #[msg("fund: normalized token already set")]
    FundNormalizedTokenAlreadySetError,

    #[msg("fund: normalized token is not set")]
    FundNormalizedTokenNotSetError,

    #[msg("fund: restaking vault already registered")]
    FundRestakingVaultAlreadyRegisteredError,

    #[msg("fund: exceeded max restaking vaults")]
    FundExceededMaxRestakingVaultsError,

    #[msg("fund: not supported restaking vault")]
    FundRestakingNotSupportedVaultError,

    #[msg("fund: restaking vault not found")]
    FundRestakingVaultNotFoundError,

    #[msg("fund: restaking vault operator not found")]
    FundRestakingVaultOperatorNotFoundError,

    #[msg("fund: restaking vault operator already registered")]
    FundRestakingVaultOperatorAlreadyRegisteredError,

    #[msg("fund: exceeded max restaking vault delegations")]
    FundExceededMaxRestakingVaultDelegationsError,

    #[msg("fund: restaking vault compounding reward token already registered")]
    FundRestakingVaultCompoundingRewardTokenAlreadyRegisteredError,

    #[msg("fund: exceeded max restaking vault compounding reward tokens")]
    FundExceededMaxRestakingVaultCompoundingRewardTokensError,

    #[msg("fund: unauhorized operation command")]
    FundOperationUnauthorizedCommandError,

    #[msg("fund: failed to compute required accounts for the operation command")]
    FundOperationCommandAccountComputationException,

    #[msg("fund: failed to execute the operation command")]
    FundOperationCommandExecutionFailedException,

    #[msg("normalization: not supported token")]
    NormalizedTokenPoolNotSupportedTokenError,

    #[msg("normalization: already supported token")]
    NormalizedTokenPoolAlreadySupportedTokenError,

    #[msg("normalization: exceeded max supported tokens")]
    NormalizedTokenPoolExceededMaxSupportedTokensError,

    #[msg("normalization: not enough supported token in the pool")]
    NormalizedTokenPoolNotEnoughSupportedTokenException,

    #[msg("normalization: already settled withdrawal account")]
    NormalizedTokenPoolAlreadySettledWithdrawalAccountError,

    #[msg("normalization: the token is non-claimable for the given withdrawal account")]
    NormalizedTokenPoolNonClaimableTokenError,

    #[msg("staking: failed to find uninitialized withdraw ticket")]
    StakingUninitializedWithdrawTicketNotFoundException,

    #[msg("staking: account not matched")]
    StakingAccountNotMatchedException,

    #[msg("staking: spl stake pool's active stake not available")]
    StakingSPLActiveStakeNotAvailableException,

    #[msg("restaking: all withdrawal tickets are already in use")]
    RestakingVaultWithdrawalTicketsExhaustedError,

    #[msg("restaking: withdrawal ticket is not withdrawable")]
    RestakingVaultWithdrawalTicketNotWithdrawableError,

    #[msg("restaking: withdrawal ticket is already initialized")]
    RestakingVaultWithdrawalTicketAlreadyInitializedError,

    #[msg("fund: donation is currently disabled")]
    FundDonationDisabledError,

    #[msg("fund: wrapped token already set")]
    FundWrappedTokenAlreadySetError,

    #[msg("fund: wrapped token is not set")]
    FundWrappedTokenNotSetError,

    #[msg("fund: token swap strategy already registered")]
    FundTokenSwapStrategyAlreadyRegistered,

    #[msg("fund: exceeded max token swap strategies")]
    FundExceededMaxTokenSwapStrategiesError,

    #[msg("fund: token swap strategy not found")]
    FundTokenSwapStrategyNotFoundError,

    #[msg("fund: restaking vault distributing reward token already registered")]
    FundRestakingVaultDistributingRewardTokenAlreadyRegisteredError,

    #[msg("fund: exceeded max restaking vault distributing reward tokens")]
    FundExceededMaxRestakingVaultDistributingRewardTokensError,
}
