use anchor_lang::prelude::*;

use crate::modules::fund::command::OperationCommand;

#[event]
pub struct OperatorRanFund {
    pub receipt_token_mint: Pubkey,
    pub fund_account: Pubkey,
    pub executed_command: OperationCommand,
    pub next_operation_sequence: u16,
}
