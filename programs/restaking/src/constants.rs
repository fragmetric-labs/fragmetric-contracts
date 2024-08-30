use anchor_lang::prelude::*;

#[constant]
pub const ADMIN_PUBKEY: Pubkey = pubkey!("91zBeWL8kHBaMtaVrHwWsck1UacDKvje82QQ3HE2k8mJ");
#[constant]
pub const FRAGSOL_MINT_ADDRESS: Pubkey = pubkey!("FRAGsJAbW4cHk2DYhtAWohV6MUMauJHCFtT1vGvRwnXN");
#[constant]
pub const REWARD_ACCOUNT_ADDRESS: Pubkey = pubkey!("Drdh99fsnAi36iA4DrKxfYq2h5QL3FymDrZ6iBcSCdPF");

pub const RECEIPT_TOKEN_LOCK_ACCOUNT_SEED: &[u8] = b"receipt_token_lock_account";
pub const FUND_SUPPORTED_TOKEN_ACCOUNT_SEED: &[u8] = b"fund_supported_token_account";
pub const PAYER_ACCOUNT_SEED: &[u8] = b"payer_account";

#[cfg(feature = "mainnet")]
pub const BSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi");
#[cfg(not(feature = "mainnet"))]
pub const BSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("azFVdHtAJN8BX3sbGAYkXvtdjdrT5U6rj9rovvUFos9");

pub const MSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC");
