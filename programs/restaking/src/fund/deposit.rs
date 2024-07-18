use anchor_lang::{prelude::*, system_program};

use crate::structs::Fund;

pub fn deposit_lst(amount: u64) -> Result<()> {
    Ok(())
}

pub fn deposit_sol<'info>(
    from: &mut Signer<'info>,
    to: &mut Account<'info, Fund>,
    system_program: &Program<'info, System>,
    amount: u64,
) -> Result<()> {
    msg!("depositor {}, fund {}", from.key, to.key());

    let sol_transfer_cpi_ctx = CpiContext::new(
        system_program.to_account_info(),
        system_program::Transfer {
            from: from.to_account_info(),
            to: to.to_account_info(),
        },
    );

    msg!("Transferring from {} to {}", from.key, to.key());

    let res = system_program::transfer(sol_transfer_cpi_ctx, amount)?;

    to.sol_amount_in += amount as u128;

    msg!("Transferred {} SOL", amount);
    Ok(res)
}
