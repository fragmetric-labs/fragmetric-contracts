use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("invalid account data version")]
    InvalidAccountDataVersionError,

    #[msg("calculation arithmetic exception")]
    CalculationArithmeticException,

    #[msg("vault admin mismatch")]
    VaultAdminMismatchError,

    #[msg("solv protocol wallet already set")]
    SolvProtocolWalletAlreadySetError,

    #[msg("solv protocol wallet mismatch")]
    SolvProtocolWalletMismatchError,

    // TODO/phase3: deprecate
    #[msg("invalid srt deposit fee rate")]
    InvalidSolvProtocolDepositFeeRateError,

    // TODO/phase3: deprecate
    #[msg("invalid srt withdrawal fee rate")]
    InvalidSolvProtocolWithdrawalFeeRateError,

    // TODO/phase3: deprecate
    #[msg("invalid srt extra fee amount")]
    InvalidSolvProtocolExtraFeeAmountError,

    #[msg("vault supported token mint mismatch")]
    VaultSupportedTokenMintMismatchError,

    #[msg("solv receipt token mint mismatch")]
    SolvReceiptTokenMintMismatchError,

    #[msg("invalid srt price")]
    InvalidSRTPriceError,

    #[msg("exceeded max withdrawal requests")]
    ExceededMaxWithdrawalRequestsError,

    #[msg("exceeded max delegated reward tokens")]
    ExceededMaxDelegatedRewardTokensError,

    #[msg("non-delegable reward token mint")]
    NonDelegableRewardTokenMintError,

    #[msg("deposit in progress")]
    DepositInProgressError,

    #[msg("deposit not in progress")]
    DepositNotInProgressError,

    #[msg("withdrawal request not found")]
    WithdrawalRequestNotFoundError,

    #[msg("vault receipt token mint mismatch")]
    VaultReceiptTokenMintMismatchError,
}
