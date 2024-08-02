use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use fragmetric_util::Upgradable;

use crate::{constants::*, fund::*};

#[derive(Accounts)]
pub struct FundWithdrawSOL<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [USER_RECEIPT_SEED, receipt_token_mint.key().as_ref()],
        bump,
        realloc = 8 + UserReceipt::INIT_SPACE,
        realloc::payer = user,
        realloc::zero = false,
    )]
    pub user_receipt: Account<'info, UserReceipt>,

    #[account(
        mut,
        seeds = [FUND_SEED, receipt_token_mint.key().as_ref()],
        bump,
        realloc = 8 + Fund::INIT_SPACE,
        // TODO must paid by fund
        realloc::payer = user,
        realloc::zero = false,
    )]
    pub fund: Account<'info, Fund>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub system_program: Program<'info, System>,
}

impl<'info> FundWithdrawSOL<'info> {
    pub fn withdraw_sol(ctx: Context<Self>, request_id: u64) -> Result<()> {
        let Self {
            user,
            user_receipt,
            fund,
            ..
        } = ctx.accounts;

        let request = user_receipt.pop_withdrawal_request(request_id)?;

        // TODO later we have to use oracle data, but now 1:1
        #[allow(clippy::identity_op)]
        let sol_amount = request.receipt_token_amount * 1;
        fund.to_latest_version()
            .withdraw_sol(request.batch_id, sol_amount)?;

        fund.sub_lamports(sol_amount)?;
        user.add_lamports(sol_amount)?;

        Ok(())
    }
}
