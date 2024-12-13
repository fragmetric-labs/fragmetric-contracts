use anchor_lang::prelude::*;

use crate::modules::fund::command::OperationCommand;
use crate::modules::fund::FundAccountInfo;

#[event]
pub struct OperatorRanFund {
    pub executed_command: OperationCommand,
    pub receipt_token_mint: Pubkey,
    pub fund_account: FundAccountInfo,
}
