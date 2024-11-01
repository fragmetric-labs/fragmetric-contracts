use crate::modules::fund::FundAccount;
use anchor_lang::prelude::*;

pub(crate) fn normalize_lst_operation_reserved(
    _fund_account: &mut Account<FundAccount>,
    _supported_token: &Pubkey,
    _amount_in: u64,
    _remaining_accounts: &[AccountInfo],
) -> Result<u64> {
    todo!()
}