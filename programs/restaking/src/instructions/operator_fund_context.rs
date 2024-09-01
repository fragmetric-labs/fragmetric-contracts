use anchor_lang::prelude::*;
use anchor_spl::{token_2022::Token2022, token_interface::{Mint, TokenAccount}};

use crate::constants::*;
use crate::events::OperatorProcessedJob;
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

    // TODO: use address lookup table!
    #[account(address = BSOL_STAKE_POOL_ADDRESS)]
    /// CHECK: will be checked and deserialized when needed
    pub token_pricing_source_0: UncheckedAccount<'info>,

    // TODO: use address lookup table!
    #[account(address = MSOL_STAKE_POOL_ADDRESS)]
    /// CHECK: will be checked and deserialized when needed
    pub token_pricing_source_1: UncheckedAccount<'info>,
}

impl<'info> OperatorFundContext<'info> {
    pub fn process_fund_withdrawal_job(ctx: Context<Self>, forced: bool) -> Result<()> {
        if !(forced && ctx.accounts.operator.key() == ADMIN_PUBKEY) {
            FundWithdrawalJob::check_threshold(&ctx.accounts.fund_account.withdrawal_status)?;
        }

        let (receipt_token_price, receipt_token_total_supply) = FundWithdrawalJob::new(
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.receipt_token_program,
            &mut ctx.accounts.receipt_token_lock_authority,
            &mut ctx.accounts.receipt_token_lock_account,
            &mut ctx.accounts.fund_account,
            &[
                ctx.accounts.token_pricing_source_0.as_ref(),
                ctx.accounts.token_pricing_source_1.as_ref(),
            ],
        )
            .process()?;

        emit!(OperatorProcessedJob {
            fund_account: FundAccountInfo::new(
                &ctx.accounts.fund_account,
                receipt_token_price,
                receipt_token_total_supply
            ),
        });

        Ok(())
    }
}
