use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // fund
    #[msg("Sol transfer failed")]
    SolTransferFailed,
    #[msg("Already existing token")]
    AlreadyExistingToken,
    #[msg("Not existing token")]
    NotExistingToken,
    #[msg("Exceeds the token cap")]
    ExceedsTokenCap,
}
