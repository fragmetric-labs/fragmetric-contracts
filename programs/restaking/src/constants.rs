use anchor_lang::prelude::*;

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
pub const ADMIN_PUBKEY: Pubkey = pubkey!("fragSkuEpEmdoj9Bcyawk9rBdsChcVJLWHfj9JX1Gby");
#[cfg(not(feature = "mainnet"))]
#[constant]
pub const ADMIN_PUBKEY: Pubkey = pubkey!("fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP");

#[constant]
pub const FRAGSOL_MINT_ADDRESS: Pubkey = pubkey!("FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo");

pub const BSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi");
pub const MSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC");
