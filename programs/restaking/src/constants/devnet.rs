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
pub const JITO_VAULT_PROGRAM_FEE_WALLET: Pubkey =
    pubkey!("9eZbWiHsPRsxLSiHxzg2pkXsAuQMwAjQrda7C7e21Fw6");

#[constant]
pub const JITO_VAULT_CONFIG_ADDRESS: Pubkey =
    pubkey!("UwuSgAq4zByffCGCrWH87DsjfsewYjuqHfJEpzw1Jq3");

#[constant]
pub const JITO_RESTAKING_PROGRAM_ID: Pubkey =
    pubkey!("RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q");

#[constant]
pub const JITO_RESTAKING_CONFIG_ADDRESS: Pubkey =
    pubkey!("4vvKh3Ws4vGzgXRVdo8SdL4jePXDvCqKVmi21BCBGwvn");

////////////////////////////////////////////
// receipt token: fragSOL
////////////////////////////////////////////

#[constant]
pub const FRAGSOL_ADDRESS_LOOKUP_TABLE_ADDRESS: Pubkey =
    pubkey!("5i5ExdTT7j36gKyiyjhaEcqFWUESvi6maASJyxKVZLyU");

#[constant]
pub const FRAGSOL_MINT_ADDRESS: Pubkey = pubkey!("FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo");

#[constant]
pub const FRAGSOL_NORMALIZED_TOKEN_MINT_ADDRESS: Pubkey =
    pubkey!("nSoLnkrvh2aY792pgCNT6hzx84vYtkviRzxvhf3ws8e");

#[constant]
pub const FRAGSOL_JITO_NSOL_VAULT_ACCOUNT_ADDRESS: Pubkey =
    pubkey!("BxhsigZDYjWTzXGgem9W3DsvJgFpEK5pM2RANP22bxBE");

#[constant]
pub const FRAGSOL_JITO_NSOL_VAULT_RECEIPT_TOKEN_MINT_ADDRESS: Pubkey =
    pubkey!("7jff6VT8twUX3513HuhN7EF18DtUzBj2N1goWroZ29t");

#[constant]
pub const FRAGSOL_JITO_JITOSOL_VAULT_ACCOUNT_ADDRESS: Pubkey =
    pubkey!("9rNA5PpjRPGxexDSoffQ8yRhMBMvRQrffwSnDBcXJjwY");

#[constant]
pub const FRAGSOL_JITO_JITOSOL_VAULT_RECEIPT_TOKEN_MINT_ADDRESS: Pubkey =
    pubkey!("J6AS6PFJip13cStdiuvRrLz2hDZiZvxdLhmsopN7YTDM");

////////////////////////////////////////////
// receipt token: fragJTO
////////////////////////////////////////////

#[constant]
pub const FRAGJTO_ADDRESS_LOOKUP_TABLE_ADDRESS: Pubkey =
    pubkey!("6VHmiiuZAW2PVoY5N16oqs8wYVkXnfmZBcM7Vkbb76jH");

#[constant]
pub const FRAGJTO_MINT_ADDRESS: Pubkey = pubkey!("FRAGJ157KSDfGvBJtCSrsTWUqFnZhrw4aC8N8LqHuoos");

#[constant]
pub const FRAGJTO_JITO_JTO_VAULT_ACCOUNT_ADDRESS: Pubkey =
    pubkey!("7dCQpU5w6Xz3aAnpFrXByBg9LxLdz33deUCrWJAVcNaE");

#[constant]
pub const FRAGJTO_JITO_JTO_VAULT_RECEIPT_TOKEN_MINT_ADDRESS: Pubkey =
    pubkey!("6VSjoP9hyHKKNZfcDzrAKRKWKSnyKhzLgBR9dtewPN9z");
