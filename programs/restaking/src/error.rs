use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // fund
    #[msg("Fund - Sol transfer failed")]
    FundSolTransferFailed,
    #[msg("Fund - Already existing token")]
    FundAlreadyExistingToken,
    #[msg("Fund - Not existing token")]
    FundNotExistingToken,
    #[msg("Fund - Exceeds the token cap")]
    FundExceedsTokenCap,
}
