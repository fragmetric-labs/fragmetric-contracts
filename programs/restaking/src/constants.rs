#![allow(unused)]
use anchor_lang::prelude::*;

// privileged for financial operations and fund configuration (ledger in mainnet)
#[cfg(feature = "mainnet")]
pub const FUND_MANAGER_PUBKEY: Pubkey = pubkey!("79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"); // ledger-o3
#[cfg(feature = "devnet")]
pub const FUND_MANAGER_PUBKEY: Pubkey = pubkey!("5UpLTLA7Wjqp7qdfjuTtPcUw3aVtbqFA5Mgm34mxPNg2"); // ledger-e1
#[cfg(not(all(feature = "mainnet", feature="devnet")))]
pub const FUND_MANAGER_PUBKEY: Pubkey = pubkey!(/*local:FUND_MANAGER*/"5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx"/**/);

// privileged for non-financial operations and scheduled tasks
#[cfg(feature = "mainnet")]
pub const ADMIN_PUBKEY: Pubkey = pubkey!("fragSkuEpEmdoj9Bcyawk9rBdsChcVJLWHfj9JX1Gby");
#[cfg(feature = "devnet")]
pub const ADMIN_PUBKEY: Pubkey = pubkey!("fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP");
#[cfg(not(all(feature = "mainnet", feature="devnet")))]
pub const ADMIN_PUBKEY: Pubkey = pubkey!(/*local:ADMIN*/"9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL"/**/);


#[cfg(any(feature = "mainnet", feature="devnet"))]
#[constant]
pub const FRAGSOL_MINT_ADDRESS: Pubkey = pubkey!("FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo");
#[cfg(not(all(feature = "mainnet", feature="devnet")))]
#[constant]
pub const FRAGSOL_MINT_ADDRESS: Pubkey = pubkey!(/*local:FRAGSOL_MINT*/"Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD"/**/);


/**
    Below address are needed to be passed to transactions which includes pricing of tokens (token deposit, withdrawal request)
    A complete list will be provided to client via address lookup table later.
**/

#[constant]
pub const MAINNET_BSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi");
#[constant]
pub const DEVNET_BSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("azFVdHtAJN8BX3sbGAYkXvtdjdrT5U6rj9rovvUFos9");
#[constant]
pub const MAINNET_MSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC");
#[constant]
pub const DEVNET_MSOL_STAKE_POOL_ADDRESS: Pubkey = MAINNET_MSOL_STAKE_POOL_ADDRESS;
#[constant]
pub const MAINNET_JITOSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb");
