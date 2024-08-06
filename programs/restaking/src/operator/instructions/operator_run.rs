use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::{constants::*, error::ErrorCode, fund::*, token::*, Empty};
use fragmetric_util::Upgradable;

#[derive(Accounts)]
pub struct OperatorRun<'info> {
    // Only the admin can run the operator manually.
    #[account(mut, address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [FUND_SEED, receipt_token_mint.key().as_ref()],
        bump,
        realloc = 8 + Fund::INIT_SPACE,
        // TODO must paid by fund
        realloc::payer = admin,
        realloc::zero = false,
    )]
    pub fund: Account<'info, Fund>,

    #[account(
        mut,
        seeds = [FUND_TOKEN_AUTHORITY_SEED, receipt_token_mint.key().as_ref()],
        bump,)]
    pub fund_token_authority: Account<'info, Empty>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = fund_token_authority,
        associated_token::token_program = token_program,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's fragSOL lock account

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> OperatorRun<'info> {
    /// Manually run the operator.
    /// This instruction is only available to ADMIN
    pub fn operator_run(ctx: Context<Self>) -> Result<()> {
        let fund = ctx.accounts.fund.to_latest_version();

        if fund
            .withdrawal_status
            .pending_batch_withdrawal
            .receipt_token_to_process
            > 0
        {
            fund.withdrawal_status
                .start_processing_pending_batch_withdrawal()?;
        }

        let receipt_token_amount_to_burn = fund
            .withdrawal_status
            .batch_withdrawals_in_progress
            .iter_mut()
            .map(|batch| {
                let amount = batch.receipt_token_to_process;
                batch.record_unstaking_start(amount as u64);
                amount
            })
            .sum();

        let unstaking_ratio = 1;

        let mut burned_receipt_token_amount = receipt_token_amount_to_burn;
        for batch in fund
            .withdrawal_status
            .batch_withdrawals_in_progress
            .iter_mut()
        {
            if burned_receipt_token_amount == 0 {
                break;
            }

            let receipt_token_amount = std::cmp::min(
                burned_receipt_token_amount,
                batch.receipt_token_being_processed,
            );

            burned_receipt_token_amount -= receipt_token_amount;
            let sol_reserved = receipt_token_amount * unstaking_ratio;
            batch.record_unstaking_end(receipt_token_amount as u64, sol_reserved as u64);
        }

        Self::burn_token_cpi(&ctx, receipt_token_amount_to_burn as u64)?;

        let fund = ctx.accounts.fund.to_latest_version();

        let sol_amount_moved = unstaking_ratio * receipt_token_amount_to_burn;

        fund.sol_amount_in = fund
            .sol_amount_in
            .checked_sub(sol_amount_moved)
            .ok_or_else(|| error!(ErrorCode::FundWithdrawalRequestExceedsSOLAmountsInTemp))?;

        fund.withdrawal_status.last_batch_processing_started_at =
            Some(Clock::get()?.unix_timestamp);

        ctx.accounts
            .fund
            .to_latest_version()
            .withdrawal_status
            .end_processing_completed_batch_withdrawals()
    }

    fn burn_token_cpi(ctx: &Context<Self>, amount: u64) -> Result<()> {
        let bump = ctx.bumps.fund_token_authority;
        let key = ctx.accounts.receipt_token_mint.key();
        let signer_seeds = [FUND_TOKEN_AUTHORITY_SEED, key.as_ref(), &[bump]];

        ctx.accounts.token_program.burn_token_cpi(
            &ctx.accounts.receipt_token_mint,
            &ctx.accounts.receipt_token_lock_account,
            ctx.accounts.fund_token_authority.to_account_info(),
            Some(&[signer_seeds.as_ref()]),
            amount,
        )
    }
}
