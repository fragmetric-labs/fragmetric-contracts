use anchor_lang::prelude::*;

use crate::constants::*;
use crate::utils::SystemProgramExt;

/// TODO change name
///
/// Description: This instruction manages a special account (for now, called "payer")
/// - initialize, or transfer sol into it.
///
/// Why do we need this account?
/// In order to create source/destination user_receipt and user_reward_account within transfer hook,
/// someone needs to pay for it.
///
/// However, there is no signer or payer in transfer hook,
/// since all accounts from the initial transfer are converted to read-only.
/// (See https://solana.com/developers/guides/token-extensions/transfer-hook)
///
/// "Payer" is a special PDA account. It is special because the owner is system program,
/// not our program. Therefore "Payer" account can pay for `create_account` CPI,
/// although it is an account off the curve.
#[derive(Accounts)]
pub struct TokenInitializePayerAccount<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [PAYER_ACCOUNT_SEED],
        bump,
    )]
    /// CHECK: "Payer" account does not have any data.
    pub payer_account: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> TokenInitializePayerAccount<'info> {
    pub fn initialize_payer_account(ctx: Context<Self>) -> Result<()> {
        let rent = Rent::get()?;
        ctx.accounts.system_program.create_account(
            &ctx.accounts.admin,
            None,
            &ctx.accounts.payer_account,
            Some(&[&[PAYER_ACCOUNT_SEED, &[ctx.bumps.payer_account]]]),
            0,
            rent.minimum_balance(0),
            &anchor_lang::solana_program::system_program::ID,
        )?;

        Ok(())
    }

    pub fn add_payer_account_lamports(ctx: Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts.system_program.transfer(
            &ctx.accounts.admin,
            None,
            &ctx.accounts.payer_account,
            amount,
        )?;
        Ok(())
    }
}
