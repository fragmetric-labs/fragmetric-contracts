use anchor_lang::prelude::{Account, AccountInfo};
use crate::modules::fund::FundAccount;

pub(crate) fn restake_nt_operation_reserved(
    _fund_account: &mut Account<FundAccount>,
    _amount_in: u64,
    _remaining_accounts: &[AccountInfo],
) {
    todo!()
}