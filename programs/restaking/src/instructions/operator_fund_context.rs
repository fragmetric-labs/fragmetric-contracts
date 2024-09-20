use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::constants::*;
use crate::events::{OperatorProcessedJob, OperatorUpdatedFundPrice};
use crate::modules::common::PDASignerSeeds;
use crate::modules::fund::{FundAccount, FundAccountInfo, ReceiptTokenLockAuthority};
use crate::modules::operator::FundWithdrawalJob;

#[derive(Accounts)]
pub struct OperatorFundContext<'info> {
    pub operator: Signer<'info>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub receipt_token_program: Program<'info, Token2022>,

    #[account(
        seeds = [ReceiptTokenLockAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = receipt_token_lock_authority.bump,
        has_one = receipt_token_mint,
    )]
    pub receipt_token_lock_authority: Account<'info, ReceiptTokenLockAuthority>,

    #[account(
        mut,
        token::mint = receipt_token_mint,
        token::authority = receipt_token_lock_authority,
        token::token_program = receipt_token_program,
        seeds = [ReceiptTokenLockAuthority::TOKEN_ACCOUNT_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's fragSOL lock account

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.bump,
        has_one = receipt_token_mint,
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,
}

impl<'info> OperatorFundContext<'info> {
    pub fn process_fund_withdrawal_job(
        ctx: Context<'_, '_, '_, 'info, Self>,
        forced: bool,
    ) -> Result<()> {
        let mut job = FundWithdrawalJob::new(
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.receipt_token_program,
            &mut ctx.accounts.receipt_token_lock_authority,
            &mut ctx.accounts.receipt_token_lock_account,
            &mut ctx.accounts.fund_account,
            ctx.remaining_accounts,
        );

        if !(forced && ctx.accounts.operator.key() == ADMIN_PUBKEY) {
            job.check_threshold()?;
        }

        let (receipt_token_price, receipt_token_total_supply) = job.process()?;

        emit!(OperatorProcessedJob {
            receipt_token_mint: ctx.accounts.receipt_token_mint.key(),
            fund_account: FundAccountInfo::new(
                &ctx.accounts.fund_account,
                receipt_token_price,
                receipt_token_total_supply
            ),
        });

        Ok(())
    }

    pub fn update_prices(ctx: Context<Self>) -> Result<()> {
        ctx.accounts
            .fund_account
            .update_token_prices(ctx.remaining_accounts)?;

        let receipt_token_total_supply = ctx.accounts.receipt_token_mint.supply;
        let receipt_token_price = ctx
            .accounts
            .fund_account
            .receipt_token_sol_value_per_token(
                ctx.accounts.receipt_token_mint.decimals,
                receipt_token_total_supply,
            )?;

        emit!(OperatorUpdatedFundPrice {
            receipt_token_mint: ctx.accounts.receipt_token_mint.key(),
            fund_account: FundAccountInfo::new(
                &ctx.accounts.fund_account,
                receipt_token_price,
                receipt_token_total_supply
            ),
        });

        Ok(())
    }
}
