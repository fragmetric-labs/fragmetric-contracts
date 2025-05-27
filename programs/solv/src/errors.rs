use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("invalid account data version")]
    InvalidAccountDataVersionError,

    #[msg("calculation arithmetic exception")]
    CalculationArithmeticException,

    #[msg("vault admin mismatch")]
    VaultAdminMismatchError,

    #[msg("solv protocol wallet mismatch")]
    SolvProtocolWalletMismatchError,

    #[msg("vault supported token mint mismatch")]
    VaultSupportedTokenMintMismatchError,

    #[msg("solv receipt token mint mismatch")]
    SolvReceiptTokenMintMismatchError,

    #[msg("exceeded max withdrawal requests")]
    ExceededMaxWithdrawalRequestsError,

    #[msg("exceeded max delegated reward tokens")]
    ExceededMaxDelegatedRewardTokensError,

    #[msg("non-delegable reward token mint")]
    NonDelegableRewardTokenMintError,

    #[msg("invalid srt exchange rate")]
    InvalidSRTExchangeRateError,
}
