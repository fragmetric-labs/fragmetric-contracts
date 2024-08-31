use anchor_lang::prelude::*;
use crate::modules::fund::FundInfo;

#[event]
pub struct OperatorRan {
    pub fund_info: FundInfo,
}
