use anchor_lang::prelude::*;

use crate::operator::*;
use crate::fund::FundInfo;

#[event]
pub struct OperatorRan {
    pub fund_info: FundInfo,
}
