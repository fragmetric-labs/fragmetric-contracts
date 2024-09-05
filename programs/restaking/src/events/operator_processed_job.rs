use anchor_lang::prelude::*;

use crate::modules::fund::FundAccountInfo;

#[event]
pub struct OperatorProcessedJob {
    pub fund_account: FundAccountInfo,
}
