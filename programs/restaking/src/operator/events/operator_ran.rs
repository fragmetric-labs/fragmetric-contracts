use anchor_lang::prelude::*;

use crate::fund::FundInfo;

#[event]
pub struct OperatorRan {
    pub fund_info: FundInfo,
}
