use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_2022::Token2022, token_interface::{Mint, TokenAccount}};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::{UserRequestedWithdrawalFromFund, UserUpdatedRewardPool};
use crate::modules::common::PDASignerSeeds;
use crate::modules::fund::{Fund, ReceiptTokenLockAuthority, ReceiptTokenMintAuthority, UserReceipt};
use crate::modules::reward::{RewardAccount, UserRewardAccount};
use crate::modules::token::{BurnExt, MintExt};

#[derive(Accounts)]
pub struct FundRequestWithdrawal<'info> {
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
        seeds = [ReceiptTokenLockAuthority::TOKEN_ACCOUNT_SEED, receipt_token_mint.key().as_ref()],
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

impl<'info> FundRequestWithdrawal<'info> {
    pub fn request_withdrawal(mut ctx: Context<Self>, receipt_token_amount: u64) -> Result<()> {
        // Verify
        require_gte!(
            ctx.accounts.receipt_token_account.amount,
            receipt_token_amount
        );

        // Step 1: Create withdrawal request
        ctx.accounts
            .fund
            .withdrawal_status
            .check_withdrawal_enabled()?;
        let withdrawal_request = ctx
            .accounts
            .fund
            .withdrawal_status
            .create_withdrawal_request(receipt_token_amount)?;
        let batch_id = withdrawal_request.batch_id;
        let request_id = withdrawal_request.request_id;
        ctx.accounts
            .user_receipt
            .push_withdrawal_request(withdrawal_request)?;

        // Step 2: Lock receipt token
        Self::call_burn_token_cpi(&mut ctx, receipt_token_amount)?;
        Self::call_mint_token_cpi(&mut ctx, receipt_token_amount)?;
        Self::call_transfer_hook(&mut ctx, receipt_token_amount)?;

        // Step 3: Update user_receipt's receipt_token_amount
        let receipt_token_account_total_amount = ctx.accounts.receipt_token_account.amount;
        ctx.accounts
            .user_receipt
            .set_receipt_token_amount(receipt_token_account_total_amount);

        emit!(UserRequestedWithdrawalFromFund {
            user: ctx.accounts.user.key(),
            user_receipt_token_account: ctx.accounts.receipt_token_account.key(),
            user_receipt: Clone::clone(&ctx.accounts.user_receipt),
            batch_id,
            request_id,
            requested_receipt_token_mint: ctx.accounts.receipt_token_mint.key(),
            requested_receipt_token_amount: receipt_token_amount,
        });

        Ok(())
    }

    fn call_burn_token_cpi(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts
            .token_program
            .burn_token_cpi(
                &mut ctx.accounts.receipt_token_mint,
                &mut ctx.accounts.receipt_token_account,
                ctx.accounts.user.to_account_info(),
                None,
                amount,
            )
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailed))
    }

    fn call_mint_token_cpi(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts
            .token_program
            .mint_token_cpi(
                &mut ctx.accounts.receipt_token_mint,
                &mut ctx.accounts.receipt_token_lock_account,
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
        let (from_user_update, to_user_update) = ctx
            .accounts
            .reward_account
            .update_reward_pools_token_allocation(
                ctx.accounts.receipt_token_mint.key(),
                amount,
                None,
                Some(&mut ctx.accounts.user_reward_account),
                None,
                current_slot,
            )?;

        emit!(UserUpdatedRewardPool::new_from_updates(
            from_user_update,
            to_user_update
        ));

        Ok(())
    }
}
