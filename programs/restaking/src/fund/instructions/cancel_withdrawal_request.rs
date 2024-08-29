use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::{common::*, constants::*, error::ErrorCode, fund::*, reward::*, token::*};

#[derive(Accounts)]
pub struct FundCancelWithdrawalRequest<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [UserReceipt::SEED, user.key().as_ref(), receipt_token_mint.key().as_ref()],
        bump = user_receipt.bump,
        has_one = user,
        has_one = receipt_token_mint,
    )]
    pub user_receipt: Account<'info, UserReceipt>,

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

    #[account(
        seeds = [ReceiptTokenMintAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = receipt_token_mint_authority.bump,
        has_one = receipt_token_mint,
    )]
    pub receipt_token_mint_authority: Account<'info, ReceiptTokenMintAuthority>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // user's fragSOL token account
    #[account(
        mut,
        token::mint = receipt_token_mint,
        token::authority = receipt_token_lock_authority,
        seeds = [RECEIPT_TOKEN_LOCK_ACCOUNT_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's fragSOL lock account

    #[account(mut, address = REWARD_ACCOUNT_ADDRESS)]
    pub reward_account: Box<Account<'info, RewardAccount>>,

    #[account(
        mut,
        seeds = [UserRewardAccount::SEED, user.key().as_ref()],
        bump = user_reward_account.bump,
        has_one = user,
    )]
    pub user_reward_account: Box<Account<'info, UserRewardAccount>>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> FundCancelWithdrawalRequest<'info> {
    pub fn cancel_withdrawal_request(mut ctx: Context<Self>, request_id: u64) -> Result<()> {
        let withdrawal_status = &mut ctx.accounts.fund.withdrawal_status;

        // Verify
        require_gt!(withdrawal_status.next_request_id, request_id);

        // Step 1: Cancel withdrawal request
        let request = ctx
            .accounts
            .user_receipt
            .pop_withdrawal_request(request_id)?;
        withdrawal_status.check_batch_processing_not_started(request.batch_id)?;
        withdrawal_status.remove_withdrawal_request(request.receipt_token_amount)?;

        // Step 2: Unlock receipt token
        Self::call_burn_token_cpi(&mut ctx, request.receipt_token_amount)?;
        Self::call_mint_token_cpi(&mut ctx, request.receipt_token_amount)?;
        Self::call_transfer_hook(&mut ctx, request.receipt_token_amount)?;

        // Step 3: Update user_receipt's receipt_token_amount
        let receipt_token_account_total_amount = ctx.accounts.receipt_token_account.amount;
        ctx.accounts
            .user_receipt
            .set_receipt_token_amount(receipt_token_account_total_amount);

        emit!(UserCanceledWithdrawalRequestFromFund {
            user: ctx.accounts.user.key(),
            user_receipt_token_account: ctx.accounts.receipt_token_account.key(),
            user_receipt: Clone::clone(&ctx.accounts.user_receipt),
            request_id,
            requested_receipt_token_mint: ctx.accounts.receipt_token_mint.key(),
            requested_receipt_token_amount: request.receipt_token_amount,
        });

        Ok(())
    }

    fn call_burn_token_cpi(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts
            .token_program
            .burn_token_cpi(
                &mut ctx.accounts.receipt_token_mint,
                &mut ctx.accounts.receipt_token_lock_account,
                ctx.accounts.receipt_token_lock_authority.to_account_info(),
                Some(&[ctx
                    .accounts
                    .receipt_token_lock_authority
                    .signer_seeds()
                    .as_ref()]),
                amount,
            )
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailed))
    }

    fn call_mint_token_cpi(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts
            .token_program
            .mint_token_cpi(
                &mut ctx.accounts.receipt_token_mint,
                &mut ctx.accounts.receipt_token_account,
                ctx.accounts.receipt_token_mint_authority.to_account_info(),
                Some(&[ctx
                    .accounts
                    .receipt_token_mint_authority
                    .signer_seeds()
                    .as_ref()]),
                amount,
            )
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailed))
    }

    fn call_transfer_hook(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        ctx.accounts
            .reward_account
            .update_reward_pools_token_allocation(
                ctx.accounts.receipt_token_mint.key(),
                amount,
                None,
                None,
                Some(&mut ctx.accounts.user_reward_account),
                current_slot,
            )
    }
}
