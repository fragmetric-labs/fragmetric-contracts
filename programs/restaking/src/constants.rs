use anchor_lang::prelude::*;

#[cfg(feature = "mainnet")]
#[constant]
pub const PROGRAM_ID: Pubkey = pubkey!("fragnAis7Bp6FTsMoa6YcH8UffhEw43Ph79qAiK3iF3");
#[cfg(not(feature = "mainnet"))]
#[constant]
pub const PROGRAM_ID: Pubkey = pubkey!("frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ");

// privileged for financial operations and fund configuration (ledger in mainnet)
#[cfg(feature = "mainnet")]
#[constant]
pub const FUND_MANAGER_PUBKEY: Pubkey = pubkey!("79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84");
#[cfg(not(feature = "mainnet"))]
#[constant]
pub const FUND_MANAGER_PUBKEY: Pubkey = pubkey!("fragHx7xwt9tXZEHv2bNo3hGTtcHP9geWkqc2Ka6FeX");

// privileged for non-financial operations and scheduled tasks
#[cfg(feature = "mainnet")]
#[constant]
pub const ADMIN_PUBKEY: Pubkey = pubkey!("fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP");
#[cfg(not(feature = "mainnet"))]
#[constant]
pub const ADMIN_PUBKEY: Pubkey = pubkey!("fragSkuEpEmdoj9Bcyawk9rBdsChcVJLWHfj9JX1Gby");

#[constant]
pub const FRAGSOL_MINT_ADDRESS: Pubkey = pubkey!("FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo");

#[cfg(feature = "mainnet")]
pub const BSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi");
#[cfg(not(feature = "mainnet"))]
pub const BSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("azFVdHtAJN8BX3sbGAYkXvtdjdrT5U6rj9rovvUFos9");

pub const MSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC");

pub const PAYER_ACCOUNT_SEED: &[u8] = b"payer_account";