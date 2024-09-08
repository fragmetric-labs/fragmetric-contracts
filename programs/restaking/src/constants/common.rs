#![allow(dead_code, unused_imports)]
use anchor_lang::prelude::*;

/**
    Below address are needed to be passed to transactions which includes pricing of tokens (token deposit, withdrawal request)
    A complete list will be provided to client via address lookup table later.
**/

#[constant]
pub const MAINNET_BSOL_MINT_ADDRESS: Pubkey = pubkey!("bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1");
#[constant]
pub const DEVNET_BSOL_MINT_ADDRESS: Pubkey = MAINNET_BSOL_MINT_ADDRESS;
#[constant]
pub const MAINNET_BSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi");
#[constant]
pub const DEVNET_BSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("azFVdHtAJN8BX3sbGAYkXvtdjdrT5U6rj9rovvUFos9");

#[constant]
pub const MAINNET_MSOL_MINT_ADDRESS: Pubkey = pubkey!("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So");
#[constant]
pub const DEVNET_MSOL_MINT_ADDRESS: Pubkey = MAINNET_MSOL_MINT_ADDRESS;
#[constant]
pub const MAINNET_MSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC");
#[constant]
pub const DEVNET_MSOL_STAKE_POOL_ADDRESS: Pubkey = MAINNET_MSOL_STAKE_POOL_ADDRESS;

#[constant]
pub const MAINNET_JITOSOL_MINT_ADDRESS: Pubkey = pubkey!("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn");
#[constant]
pub const MAINNET_JITOSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb");
