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

#[constant]
pub const JITO_VAULT_PROGRAM_ID: Pubkey = pubkey!("34X2uqBhEGiWHu43RDEMwrMqXF4CpCPEZNaKdAaUS9jx");

#[constant]
pub const JITO_VAULT_CONFIG_ADDRESS: Pubkey = pubkey!("Cx2tQmB4RdCQADK8dGzt9sbXDpQ9o2pcjuhKnN42NxbK");

#[constant]
pub const JITO_VAULT_ADDRESS: Pubkey = pubkey!("8bCy6TWfxc7H2ib61ijR1LzGynZNuVspdeUNra9AS9Lg");

#[constant]
pub const JITO_VAULT_RECEIPT_TOKEN: Pubkey = pubkey!("5w2JCmAbBdSRv1y8igM3YNjvnGdfYUeYuVmtw9fU5TXZ");

#[constant]
pub const JITO_VAULT_SUPPORTED_TOKEN: Pubkey = pubkey!("J1shGRurZVzL2DqZNzfSU8s44H2B94kw5YyyckuszG1N");
