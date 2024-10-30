use anchor_lang::prelude::Pubkey;
use anchor_lang::{constant, declare_id, pubkey};

declare_id!("frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ");

#[constant]
pub const TARGET: &str = "devnet";

#[constant]
pub const ADMIN_PUBKEY: Pubkey = pubkey!("fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP");

#[constant]
pub const FUND_MANAGER_PUBKEY: Pubkey = pubkey!("5UpLTLA7Wjqp7qdfjuTtPcUw3aVtbqFA5Mgm34mxPNg2"); // ledger-e1

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
