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
pub const FRAGSOL_MINT_ADDRESS: Pubkey = pubkey!(/*local:FRAGSOL_MINT*/"Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD"/**/);

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
