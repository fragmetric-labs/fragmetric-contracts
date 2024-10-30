use anchor_lang::prelude::Pubkey;
use anchor_lang::{constant, declare_id, pubkey};

declare_id!("fragnAis7Bp6FTsMoa6YcH8UffhEw43Ph79qAiK3iF3");

#[constant]
pub const TARGET: &str = "mainnet";

#[constant]
pub const ADMIN_PUBKEY: Pubkey = pubkey!("fragSkuEpEmdoj9Bcyawk9rBdsChcVJLWHfj9JX1Gby");

#[constant]
pub const FUND_MANAGER_PUBKEY: Pubkey = pubkey!("79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"); // ledger-o3

#[constant]
pub const FRAGSOL_MINT_ADDRESS: Pubkey = pubkey!("FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo");
