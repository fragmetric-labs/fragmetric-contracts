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
}
