use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};
use fragmetric_util::Upgradable;

use crate::{constants::*, fund::*, token::*, Empty};

#[derive(Accounts)]
pub struct FundProcessWithdrawalRequestsForTest<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [FUND_SEED, receipt_token_mint.key().as_ref()],
        bump,
        realloc = 8 + Fund::INIT_SPACE,
        // TODO must paid by fund
        realloc::payer = payer,
        realloc::zero = false,
    )]
    pub fund: Account<'info, Fund>,

    #[account(
        seeds = [FUND_TOKEN_AUTHORITY_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
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

impl<'info> FundProcessWithdrawalRequestsForTest<'info> {
    /// This is an instruction for test purpose:
    /// It mocks 4 instructions that should be performed by operator.
    pub fn process_withdrawal_requests_for_test(mut ctx: Context<Self>) -> Result<()> {
        let fund = ctx.accounts.fund.to_latest_version();

        // Operator Instruction - Decides to start processing pending withdrawals
        if fund
            .withdrawal_status
            .pending_batch_withdrawal
            .receipt_token_to_process
            > 0
        {
            fund.withdrawal_status
                .start_processing_pending_batch_withdrawal()?;
        }

        // Operator Instruction - Request for LST unstaking (to make SOL)
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

        // Operator Instruction - Record unstaking result to the fund
        // NOTE: assumes that the amount of unstaked SOL is equal to the amount of burned fragSOL
        let unstaking_ratio = 1; // unstaked SOL per 1 fragSOL

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
            let sol_amount = receipt_token_amount * unstaking_ratio;
            batch.record_unstaking_end(receipt_token_amount as u64, sol_amount as u64);
        }

        Self::call_burn_token_cpi(&mut ctx, receipt_token_amount_to_burn as u64)?;

        // Operator Instruction - Ends processing completed withdrawals
        ctx.accounts
            .fund
            .to_latest_version()
            .withdrawal_status
            .end_processing_completed_batch_withdrawals()
    }

    fn call_burn_token_cpi(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        let bump = ctx.bumps.fund_token_authority;
        let key = ctx.accounts.receipt_token_mint.key();
        let signer_seeds = [FUND_TOKEN_AUTHORITY_SEED, key.as_ref(), &[bump]];

        ctx.accounts.token_program.burn_token_cpi(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.receipt_token_lock_account,
            ctx.accounts.fund_token_authority.to_account_info(),
            Some(&[signer_seeds.as_ref()]),
            amount,
        )
    }
}
