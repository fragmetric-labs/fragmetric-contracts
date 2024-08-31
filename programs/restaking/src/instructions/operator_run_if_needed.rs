use anchor_lang::prelude::*;
use anchor_spl::{token_2022::Token2022, token_interface::{Mint, TokenAccount}};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::OperatorRan;
use crate::modules::common::PDASignerSeeds;
use crate::modules::fund::{Fund, FundInfo, ReceiptTokenLockAuthority};
use crate::modules::operator::Run;

#[derive(Accounts)]
pub struct OperatorRunIfNeeded<'info> {
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [Fund::SEED, receipt_token_mint.key().as_ref()],
        bump = fund.bump,
        has_one = receipt_token_mint,
    )]
    pub fund: Box<Account<'info, Fund>>,

    #[account(
        seeds = [ReceiptTokenLockAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = receipt_token_lock_authority.bump,
        has_one = receipt_token_mint,
    )]
    pub receipt_token_lock_authority: Account<'info, ReceiptTokenLockAuthority>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        token::mint = receipt_token_mint,
        token::authority = receipt_token_lock_authority,
        seeds = [ReceiptTokenLockAuthority::TOKEN_ACCOUNT_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's fragSOL lock account

    // TODO: use address lookup table!
    #[account(address = BSOL_STAKE_POOL_ADDRESS)]
    /// CHECK: will be checked and deserialized when needed
    pub token_pricing_source_0: UncheckedAccount<'info>,

    // TODO: use address lookup table!
    #[account(address = MSOL_STAKE_POOL_ADDRESS)]
    /// CHECK: will be checked and deserialized when needed
    pub token_pricing_source_1: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
}

impl<'info> OperatorRunIfNeeded<'info> {
    /// Run operator if conditions are met.
    /// This instructions is available to anyone.
    /// However, the threshold should be met
    pub fn operator_run_if_needed(ctx: Context<Self>) -> Result<()> {
        let withdrawal_status = &mut ctx.accounts.fund.withdrawal_status;

        // if last_process_time is more than TODO_FUND_DURATION_THRESHOLD_CONFIG ago
        let current_time = crate::utils::timestamp_now()?;

        let mut threshold_satified = matches!(
            withdrawal_status.last_batch_processing_started_at,
            Some(x) if (current_time - x) > withdrawal_status.batch_processing_threshold_duration
        );

        if withdrawal_status
            .pending_batch_withdrawal
            .receipt_token_to_process
            > withdrawal_status.batch_processing_threshold_amount
        {
            threshold_satified = true;
        }

        if !threshold_satified {
            return err!(ErrorCode::OperatorUnmetThreshold);
        }

        let (receipt_token_price, receipt_token_total_supply) = Run::new(
            &mut ctx.accounts.fund,
            &mut ctx.accounts.receipt_token_lock_authority,
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.receipt_token_lock_account,
            &[
                ctx.accounts.token_pricing_source_0.as_ref(),
                ctx.accounts.token_pricing_source_1.as_ref(),
            ],
            &ctx.accounts.token_program,
        )
        .run()?;

        emit!(OperatorRan {
            fund_info: FundInfo::new_from_fund(
                &ctx.accounts.fund,
                receipt_token_price,
                receipt_token_total_supply
            ),
        });

        Ok(())
    }
}
