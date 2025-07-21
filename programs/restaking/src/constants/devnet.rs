use anchor_lang::prelude::*;

declare_id!("frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ");

#[constant]
pub const TARGET: &str = "devnet";

#[constant]
pub const ADMIN_PUBKEY: Pubkey = pubkey!("fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP");

#[constant]
pub const FUND_MANAGER_PUBKEY: Pubkey = pubkey!("5UpLTLA7Wjqp7qdfjuTtPcUw3aVtbqFA5Mgm34mxPNg2"); // ledger-e1

#[constant]
pub const PROGRAM_REVENUE_ADDRESS: Pubkey = pubkey!("SRCMj3B7cYjvwTtqJxUSptgJPWkL8bHLrQme6q4zHn7");

#[constant]
pub const JITO_VAULT_PROGRAM_ID: Pubkey = pubkey!("Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8");

#[constant]
pub const JITO_VAULT_CONFIG_ADDRESS: Pubkey =
    pubkey!("UwuSgAq4zByffCGCrWH87DsjfsewYjuqHfJEpzw1Jq3");

#[constant]
pub const SOLV_PROGRAM_ID: Pubkey = pubkey!("FsoLVPaSSXfsfZHYaxtQSZs6npFFUeRCyXB6dcC8ckn");
