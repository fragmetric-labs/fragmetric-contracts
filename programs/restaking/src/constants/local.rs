use anchor_lang::prelude::*;

declare_id!("4qEHCzsLFUnw8jmhmRSmAK5VhZVoSD1iVqukAf92yHi5");

#[constant]
pub const TARGET: &str = "local";

// privileged for non-financial operations and scheduled tasks
#[constant]
pub const ADMIN_PUBKEY: Pubkey =
    pubkey!(/*local:ADMIN*/ "9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL" /**/);

// privileged for financial operations and fund configuration
#[constant]
pub const FUND_MANAGER_PUBKEY: Pubkey =
    pubkey!(/*local:FUND_MANAGER*/ "5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx" /**/);

#[constant]
pub const PROGRAM_REVENUE_ADDRESS: Pubkey = pubkey!("GuSruSKKCmAGuWMeMsiw3mbNhjeiRtNhnh9Eatgz33NA");

#[constant]
pub const JITO_VAULT_PROGRAM_ID: Pubkey = pubkey!("Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8");

#[constant]
pub const JITO_VAULT_CONFIG_ADDRESS: Pubkey =
    pubkey!("UwuSgAq4zByffCGCrWH87DsjfsewYjuqHfJEpzw1Jq3");

#[constant]
pub const SOLV_PROGRAM_ID: Pubkey = pubkey!("9beGuWXNoKPKCApT6xJUm5435Fz8EMGzoTTXgkcf3zAz");
