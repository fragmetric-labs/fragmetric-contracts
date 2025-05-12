use anchor_lang::prelude::*;

declare_id!("fragnAis7Bp6FTsMoa6YcH8UffhEw43Ph79qAiK3iF3");

#[constant]
pub const TARGET: &str = "mainnet";

#[constant]
pub const ADMIN_PUBKEY: Pubkey = pubkey!("fragSkuEpEmdoj9Bcyawk9rBdsChcVJLWHfj9JX1Gby");

#[constant]
pub const FUND_MANAGER_PUBKEY: Pubkey = pubkey!("79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84"); // ledger-o3

#[constant]
pub const PROGRAM_REVENUE_ADDRESS: Pubkey = pubkey!("XEhpR3UauMkARQ8ztwaU9Kbv16jEpBbXs9ftELka9wj");

#[constant]
pub const JITO_VAULT_PROGRAM_ID: Pubkey = pubkey!("Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8");

#[constant]
pub const JITO_VAULT_PROGRAM_FEE_WALLET: Pubkey =
    pubkey!("5eosrve6LktMZgVNszYzebgmmC7BjLK8NoWyRQtcmGTF");

#[constant]
pub const JITO_VAULT_CONFIG_ADDRESS: Pubkey =
    pubkey!("UwuSgAq4zByffCGCrWH87DsjfsewYjuqHfJEpzw1Jq3");

#[constant]
pub const SOLV_PROGRAM_ID: Pubkey = pubkey!("FSoLvf9dv17a4DzMGYKxqFnDGj9EiXRW5wKrwQ39UDH");

////////////////////////////////////////////
// receipt token: fragSOL
////////////////////////////////////////////

#[constant]
pub const FRAGSOL_ADDRESS_LOOKUP_TABLE_ADDRESS: Pubkey =
    pubkey!("HjNXH2HMfso5YU6U7McfhsbfoecGR5QTBAxTCSbFoYqy");

#[constant]
pub const FRAGSOL_MINT_ADDRESS: Pubkey = pubkey!("FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo");

#[constant]
pub const FRAGSOL_NORMALIZED_TOKEN_MINT_ADDRESS: Pubkey =
    pubkey!("nSoLnkrvh2aY792pgCNT6hzx84vYtkviRzxvhf3ws8e");

#[constant]
pub const FRAGSOL_WRAPPED_TOKEN_MINT_ADDRESS: Pubkey =
    pubkey!("WFRGSWjaz8tbAxsJitmbfRuFV2mSNwy7BMWcCwaA28U");

#[constant]
pub const FRAGSOL_JITO_NSOL_VAULT_ACCOUNT_ADDRESS: Pubkey =
    pubkey!("HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S");

#[constant]
pub const FRAGSOL_JITO_NSOL_VAULT_RECEIPT_TOKEN_MINT_ADDRESS: Pubkey =
    pubkey!("CkXLPfDG3cDawtUvnztq99HdGoQWhJceBZxqKYL2TUrg");

#[constant]
pub const FRAGSOL_JITO_JITOSOL_VAULT_ACCOUNT_ADDRESS: Pubkey =
    pubkey!("CyGwYsiNg1FEYNCVXmPAixCkn2ZqWQVQJVy9RBCPNF7t");

#[constant]
pub const FRAGSOL_JITO_JITOSOL_VAULT_RECEIPT_TOKEN_MINT_ADDRESS: Pubkey =
    pubkey!("FRj1uf9W7TsGfSoHc1gU6V2sKxs4wMhaXC5A8KjVYvA5");

////////////////////////////////////////////
// receipt token: fragJTO
////////////////////////////////////////////

#[constant]
pub const FRAGJTO_ADDRESS_LOOKUP_TABLE_ADDRESS: Pubkey =
    pubkey!("AQtDes99nLUnSK6BQJgj9KJ6b3eDv8bUUxGCmnEJUkY5");

#[constant]
pub const FRAGJTO_MINT_ADDRESS: Pubkey = pubkey!("FRAGJ157KSDfGvBJtCSrsTWUqFnZhrw4aC8N8LqHuoos");

#[constant]
pub const FRAGJTO_WRAPPED_TOKEN_MINT_ADDRESS: Pubkey =
    pubkey!("WFRGJnQt5pK8Dv4cDAbrSsgPcmboysrmX3RYhmRRyTR");

#[constant]
pub const FRAGJTO_JITO_JTO_VAULT_ACCOUNT_ADDRESS: Pubkey =
    pubkey!("BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd");

#[constant]
pub const FRAGJTO_JITO_JTO_VAULT_RECEIPT_TOKEN_MINT_ADDRESS: Pubkey =
    pubkey!("FRJtoBLuU72X3qgkVeBU1wXtmgQpWQmWptYsAdyyu3qT");
