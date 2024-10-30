use anchor_lang::prelude::Pubkey;
use anchor_lang::{constant, declare_id, pubkey};

declare_id!("4qEHCzsLFUnw8jmhmRSmAK5VhZVoSD1iVqukAf92yHi5");

#[constant]
pub const TARGET: &str = "local";

// privileged for non-financial operations and scheduled tasks
#[constant]
pub const ADMIN_PUBKEY: Pubkey = pubkey!(/*local:ADMIN*/"9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL"/**/);

// privileged for financial operations and fund configuration
#[constant]
pub const FUND_MANAGER_PUBKEY: Pubkey = pubkey!(/*local:FUND_MANAGER*/"5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx"/**/);

#[constant]
pub const FRAGSOL_MINT_ADDRESS: Pubkey = pubkey!(/*local:FRAGSOL_MINT*/ "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD"/**/);

#[constant]
pub const NSOL_MINT_ADDRESS: Pubkey = pubkey!(/*local:NSOL_MINT*/ "4noNmx2RpxK4zdr68Fq1CYM5VhN4yjgGZEFyuB7t2pBX"/**/);

#[constant]
pub const JITO_VAULT_PROGRAM_ID: Pubkey = pubkey!("Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8");

#[constant]
pub const FRAGSOL_JITO_VAULT_CONFIG_ADDRESS: Pubkey = pubkey!("UwuSgAq4zByffCGCrWH87DsjfsewYjuqHfJEpzw1Jq3");

#[constant]
pub const FRAGSOL_JITO_VAULT_ACCOUNT_ADDRESS: Pubkey = pubkey!("Fr5Xvta98284hu9HkMUixWRRW7pSK3nS19goGTYUzHpf");

#[constant]
pub const FRAGSOL_JITO_VAULT_RECEIPT_TOKEN_MINT_ADDRESS: Pubkey = pubkey!("DXHvJfVP9dHmjP4m4GGD6ujyj8dj7BjvKgwHguqw3mD9");
