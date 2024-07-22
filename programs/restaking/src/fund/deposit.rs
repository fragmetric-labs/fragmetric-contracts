use anchor_lang::{prelude::*, system_program};

use crate::structs::Fund;

pub fn deposit_token(amount: u64) -> Result<()> {
    Ok(())
}

pub fn deposit_sol<'info>(
    fund: &mut Account<'info, Fund>,
    amount: u64,
) -> Result<()> {
    fund.sol_amount_in += amount as u128;

    // Ok(res)
    Ok(())
}
