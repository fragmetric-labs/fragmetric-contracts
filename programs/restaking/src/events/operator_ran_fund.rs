use anchor_lang::prelude::*;

use crate::modules::fund::command::OperationCommand;
use crate::modules::fund::FundAccountInfo;

#[event]
pub struct OperatorRanFund {
    pub receipt_token_mint: Pubkey,
    pub fund_account: FundAccountInfo,
    pub executed_operation_commands: Vec<OperationCommand>,
    pub next_operation_sequence: u16,
}
