use anchor_lang::prelude::*;

use crate::modules::fund::commands::{OperationCommand, OperationCommandResult};

#[event]
pub struct OperatorRanFundCommand {
    pub receipt_token_mint: Pubkey,
    pub fund_account: Pubkey,
    pub next_sequence: u16,
    pub num_operated: u64,
    pub command: OperationCommand,
    pub result: Option<OperationCommandResult>,
}
