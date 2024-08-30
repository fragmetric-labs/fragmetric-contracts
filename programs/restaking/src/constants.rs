use anchor_lang::prelude::*;

#[constant]
pub const ADMIN_PUBKEY: Pubkey = pubkey!("CiRGR8qLmqryQS375HW3yJPQxGyWiCCZExWrLKVeK4aw");
#[constant]
pub const FRAGSOL_MINT_ADDRESS: Pubkey = pubkey!("24z3s9NgUHNePqJXxCXqnFaBLM5T7fDz2XHETHuipYX1");
#[constant]
pub const REWARD_ACCOUNT_ADDRESS: Pubkey = pubkey!("FkbzST7uhLhWWeGXnZTyKWshXScm3CeCNArme4fB56Hn");

pub const RECEIPT_TOKEN_LOCK_ACCOUNT_SEED: &[u8] = b"receipt_token_lock_account";
pub const FUND_SUPPORTED_TOKEN_ACCOUNT_SEED: &[u8] = b"fund_supported_token_account";
pub const PAYER_ACCOUNT_SEED: &[u8] = b"payer_account";

#[cfg(feature = "mainnet")]
pub const BSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi");
#[cfg(not(feature = "mainnet"))]
pub const BSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("azFVdHtAJN8BX3sbGAYkXvtdjdrT5U6rj9rovvUFos9");

pub const MSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC");
