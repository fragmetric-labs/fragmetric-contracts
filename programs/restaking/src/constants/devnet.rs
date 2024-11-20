use anchor_lang::prelude::*;

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
pub const NSOL_MINT_ADDRESS: Pubkey = pubkey!("nSoLnkrvh2aY792pgCNT6hzx84vYtkviRzxvhf3ws8e");

#[constant]
pub const JITO_VAULT_PROGRAM_ID: Pubkey = pubkey!("Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8");

#[constant]
pub const JITO_VAULT_PROGRAM_FEE_WALLET: Pubkey =
    pubkey!("9eZbWiHsPRsxLSiHxzg2pkXsAuQMwAjQrda7C7e21Fw6");

#[constant]
pub const FRAGSOL_JITO_VAULT_CONFIG_ADDRESS: Pubkey =
    pubkey!("UwuSgAq4zByffCGCrWH87DsjfsewYjuqHfJEpzw1Jq3");

#[constant]
pub const FRAGSOL_JITO_VAULT_ACCOUNT_ADDRESS: Pubkey =
    pubkey!("BxhsigZDYjWTzXGgem9W3DsvJgFpEK5pM2RANP22bxBE");

#[constant]
pub const FRAGSOL_JITO_VAULT_RECEIPT_TOKEN_MINT_ADDRESS: Pubkey =
    pubkey!("7jff6VT8twUX3513HuhN7EF18DtUzBj2N1goWroZ29t");
