use anchor_lang::prelude::*;

#[constant]
pub const ADMIN_PUBKEY: Pubkey = pubkey!("91zBeWL8kHBaMtaVrHwWsck1UacDKvje82QQ3HE2k8mJ");
#[constant]
pub const FRAGSOL_MINT_ADDRESS: Pubkey = pubkey!("FRAGsJAbW4cHk2DYhtAWohV6MUMauJHCFtT1vGvRwnXN");
pub const FUND_SEED: &[u8] = b"fund";
pub const FUND_TOKEN_AUTHORITY_SEED: &[u8] = b"fund_token_authority";
pub const USER_RECEIPT_SEED: &[u8] = b"user_receipt";
pub const EXPTECED_IX_SYSVAR_INDEX: usize = 0;
