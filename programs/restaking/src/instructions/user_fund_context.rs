use anchor_lang::prelude::*;
use anchor_lang::{system_program, solana_program::sysvar::instructions as instructions_sysvar};
use anchor_spl::{associated_token::AssociatedToken, token_2022::Token2022, token_interface::{Mint, TokenAccount}};

use crate::constants::*;
use crate::events::{UserCanceledWithdrawalRequestFromFund, UserDepositedSOLToFund, UserRequestedWithdrawalFromFund, UserUpdatedRewardPool, UserWithdrewSOLFromFund};
use crate::errors::ErrorCode;
use crate::modules::common::*;
use crate::modules::fund::{DepositMetadata, FundAccount, FundAccountInfo, ReceiptTokenLockAuthority, ReceiptTokenMintAuthority, UserFundAccount};
use crate::modules::reward::{RewardAccount, UserRewardAccount};

#[derive(Accounts)]
pub struct UserFundContext<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub receipt_token_program: Program<'info, Token2022>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [ReceiptTokenMintAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = receipt_token_mint_authority.bump,
        has_one = receipt_token_mint,
    )]
    pub receipt_token_mint_authority: Account<'info, ReceiptTokenMintAuthority>,

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
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = receipt_token_mint,
        associated_token::token_program = receipt_token_program,
        associated_token::authority = user,
    )]
    pub user_receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.bump,
        has_one = receipt_token_mint,
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [UserFundAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump,
        space = 8 + UserFundAccount::INIT_SPACE,
    )]
    pub user_fund_account: Box<Account<'info, UserFundAccount>>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.bump,
        has_one = receipt_token_mint,
    )]
    pub reward_account: Box<Account<'info, RewardAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [UserRewardAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump,
        space = 8 + UserRewardAccount::INIT_SPACE,
    )]
    pub user_reward_account: Box<Account<'info, UserRewardAccount>>,

    /// CHECK: This is safe that checks it's ID
    #[account(address = instructions_sysvar::ID)]
    pub instructions_sysvar: UncheckedAccount<'info>,
}

impl<'info> UserFundContext<'info> {
    pub fn update_accounts_if_needed(ctx: Context<Self>) -> Result<()> {
        // Initialize
        ctx.accounts
            .user_fund_account
            .initialize_if_needed(
                ctx.bumps.user_fund_account,
                ctx.accounts.receipt_token_mint.key(),
                ctx.accounts.user.key(),
            );
        ctx.accounts
            .user_reward_account
            .initialize_if_needed(
                ctx.bumps.user_reward_account,
                ctx.accounts.receipt_token_mint.key(),
                ctx.accounts.user.key(),
            );
        Ok(())
    }

    pub fn deposit_sol(
        mut ctx: Context<Self>,
        amount: u64,
        metadata: Option<DepositMetadata>,
    ) -> Result<()> {
        // verify metadata signature if given
        if let Some(metadata) = &metadata {
            verify_preceding_ed25519_instruction(
                &ctx.accounts.instructions_sysvar,
                metadata.try_to_vec()?.as_slice(),
            )?;
        }
        let (wallet_provider, contribution_accrual_rate) = metadata
            .map(|metadata| (metadata.wallet_provider, metadata.contribution_accrual_rate))
            .unzip();

        // Check balance
        require_gte!(ctx.accounts.user.lamports(), amount);

        // Step 1: Calculate mint amount
        let fund = &mut ctx.accounts.fund_account;
        let receipt_token_total_supply = ctx.accounts.receipt_token_mint.supply;
        fund.update_token_prices(ctx.remaining_accounts)?;
        let receipt_token_mint_amount = fund.receipt_token_mint_amount_for(
            amount,
            receipt_token_total_supply,
        )?;
        let receipt_token_price = fund.receipt_token_sol_value_per_token(
            ctx.accounts.receipt_token_mint.decimals,
            receipt_token_total_supply,
        )?;

        // Step 2: Deposit SOL
        Self::cpi_transfer_sol_to_fund(&ctx, amount)?;
        ctx.accounts.fund_account.deposit_sol(amount)?;

        // Step 3: Mint receipt token
        Self::cpi_mint_token_to_user(&mut ctx, receipt_token_mint_amount)?;
        Self::mock_transfer_hook_from_fund_to_user(
            &mut ctx,
            receipt_token_mint_amount,
            contribution_accrual_rate,
        )?;

        // Step 4: Update user_receipt's receipt_token_amount
        let receipt_token_account_total_amount = ctx.accounts.user_receipt_token_account.amount;
        ctx.accounts
            .user_fund_account
            .set_receipt_token_amount(receipt_token_account_total_amount);

        emit!(UserDepositedSOLToFund {
            user: ctx.accounts.user.key(),
            user_receipt_token_account: ctx.accounts.user_receipt_token_account.key(),
            user_fund_account: Clone::clone(&ctx.accounts.user_fund_account),
            deposited_sol_amount: amount,
            receipt_token_mint: ctx.accounts.receipt_token_mint.key(),
            minted_receipt_token_amount: receipt_token_mint_amount,
            wallet_provider,
            contribution_accrual_rate,
            fund_account: FundAccountInfo::new(
                ctx.accounts.fund_account.as_ref(),
                receipt_token_price,
                receipt_token_total_supply
            ),
        });

        Ok(())
    }

    fn cpi_transfer_sol_to_fund(ctx: &Context<Self>, amount: u64) -> Result<()> {
        let sol_transfer_cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.fund_account.to_account_info(),
            },
        );

        system_program::transfer(sol_transfer_cpi_ctx, amount)
            .map_err(|_| error!(ErrorCode::FundSOLTransferFailedException))
    }

    fn mock_transfer_hook_from_fund_to_user(
        ctx: &mut Context<Self>,
        amount: u64,
        contribution_accrual_rate: Option<f32>,
    ) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        let contribution_accrual_rate =
            contribution_accrual_rate.map(|float| (100f32 * float).round() as u8);

        let (from_user_update, to_user_update) = ctx
            .accounts
            .reward_account
            .update_reward_pools_token_allocation(
                ctx.accounts.receipt_token_mint.key(),
                amount,
                contribution_accrual_rate,
                None,
                Some(&mut ctx.accounts.user_reward_account),
                current_slot,
            )?;

        emit!(UserUpdatedRewardPool::new(
            ctx.accounts.receipt_token_mint.key(),
            from_user_update,
            to_user_update
        ));

        Ok(())
    }

    pub fn request_withdrawal(mut ctx: Context<Self>, receipt_token_amount: u64) -> Result<()> {
        // Verify
        require_gte!(
            ctx.accounts.user_receipt_token_account.amount,
            receipt_token_amount
        );

        // Step 1: Create withdrawal request
        ctx.accounts
            .fund_account
            .withdrawal_status
            .check_withdrawal_enabled()?;
        let withdrawal_request = ctx
            .accounts
            .fund_account
            .withdrawal_status
            .create_withdrawal_request(receipt_token_amount)?;
        let batch_id = withdrawal_request.batch_id;
        let request_id = withdrawal_request.request_id;
        ctx.accounts
            .user_fund_account
            .push_withdrawal_request(withdrawal_request)?;

        // Step 2: Lock receipt token
        Self::cpi_burn_token_from_user(&mut ctx, receipt_token_amount)?;
        Self::cpi_mint_token_to_fund(&mut ctx, receipt_token_amount)?;
        Self::mock_transfer_hook_from_user_to_null(&mut ctx, receipt_token_amount)?;

        // Step 3: Update user_receipt's receipt_token_amount
        let receipt_token_account_total_amount = ctx.accounts.user_receipt_token_account.amount;
        ctx.accounts
            .user_fund_account
            .set_receipt_token_amount(receipt_token_account_total_amount);

        emit!(UserRequestedWithdrawalFromFund {
            user: ctx.accounts.user.key(),
            user_receipt_token_account: ctx.accounts.user_receipt_token_account.key(),
            user_fund_account: Clone::clone(&ctx.accounts.user_fund_account),
            batch_id,
            request_id,
            receipt_token_mint: ctx.accounts.receipt_token_mint.key(),
            requested_receipt_token_amount: receipt_token_amount,
        });

        Ok(())
    }

    fn cpi_burn_token_from_user(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts
            .receipt_token_program
            .burn_token_cpi(
                &mut ctx.accounts.receipt_token_mint,
                &mut ctx.accounts.user_receipt_token_account,
                ctx.accounts.user.to_account_info(),
                None,
                amount,
            )
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))
    }

    fn cpi_mint_token_to_fund(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts
            .receipt_token_program
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
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))
    }

    fn mock_transfer_hook_from_user_to_null(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
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

        emit!(UserUpdatedRewardPool::new(
            ctx.accounts.receipt_token_mint.key(),
            from_user_update,
            to_user_update
        ));

        Ok(())
    }

    pub fn cancel_withdrawal_request(mut ctx: Context<Self>, request_id: u64) -> Result<()> {
        let withdrawal_status = &mut ctx.accounts.fund_account.withdrawal_status;

        // Verify
        require_gt!(withdrawal_status.next_request_id, request_id);

        // Step 1: Cancel withdrawal request
        let request = ctx
            .accounts
            .user_fund_account
            .pop_withdrawal_request(request_id)?;
        withdrawal_status.check_batch_processing_not_started(request.batch_id)?;
        withdrawal_status.remove_withdrawal_request(request.receipt_token_amount)?;

        // Step 2: Unlock receipt token
        Self::cpi_burn_token(&mut ctx, request.receipt_token_amount)?;
        Self::cpi_mint_token_to_user(&mut ctx, request.receipt_token_amount)?;
        Self::mock_transfer_hook_from_null_to_user(&mut ctx, request.receipt_token_amount)?;

        // Step 3: Update user_receipt's receipt_token_amount
        let receipt_token_account_total_amount = ctx.accounts.user_receipt_token_account.amount;
        ctx.accounts
            .user_fund_account
            .set_receipt_token_amount(receipt_token_account_total_amount);

        emit!(UserCanceledWithdrawalRequestFromFund {
            user: ctx.accounts.user.key(),
            user_receipt_token_account: ctx.accounts.user_receipt_token_account.key(),
            user_fund_account: Clone::clone(&ctx.accounts.user_fund_account),
            request_id,
            receipt_token_mint: ctx.accounts.receipt_token_mint.key(),
            requested_receipt_token_amount: request.receipt_token_amount,
        });

        Ok(())
    }

    fn cpi_burn_token(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts
            .receipt_token_program
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
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))
    }

    fn cpi_mint_token_to_user(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts
            .receipt_token_program
            .mint_token_cpi(
                &mut ctx.accounts.receipt_token_mint,
                &mut ctx.accounts.user_receipt_token_account,
                ctx.accounts.receipt_token_mint_authority.to_account_info(),
                Some(&[ctx
                    .accounts
                    .receipt_token_mint_authority
                    .signer_seeds()
                    .as_ref()]),
                amount,
            )
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))
    }

    fn mock_transfer_hook_from_null_to_user(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        let (from_user_update, to_user_update) = ctx
            .accounts
            .reward_account
            .update_reward_pools_token_allocation(
                ctx.accounts.receipt_token_mint.key(),
                amount,
                None,
                None,
                Some(&mut ctx.accounts.user_reward_account),
                current_slot,
            )?;

        emit!(UserUpdatedRewardPool::new(
            ctx.accounts.receipt_token_mint.key(),
            from_user_update,
            to_user_update
        ));

        Ok(())
    }

    pub fn withdraw(ctx: Context<Self>, request_id: u64) -> Result<()> {
        let fund = &mut ctx.accounts.fund_account;

        // Verify
        require_gt!(fund.withdrawal_status.next_request_id, request_id);

        // Step 1: Update price
        fund.update_token_prices(ctx.remaining_accounts)?;

        // Step 2: Complete withdrawal request
        fund.withdrawal_status.check_withdrawal_enabled()?;
        let request = ctx
            .accounts
            .user_fund_account
            .pop_withdrawal_request(request_id)?;
        fund.withdrawal_status
            .check_batch_processing_completed(request.batch_id)?;

        // Step 3: Calculate withdraw amount
        let receipt_token_total_supply = ctx.accounts.receipt_token_mint.supply;
        let receipt_token_price = fund.receipt_token_sol_value_per_token(
            ctx.accounts.receipt_token_mint.decimals,
            receipt_token_total_supply,
        )?;
        let sol_amount = fund.receipt_token_sol_value_for(
            request.receipt_token_amount,
            receipt_token_total_supply,
        )?;
        let sol_fee_amount = fund
            .withdrawal_status
            .calculate_sol_withdrawal_fee(sol_amount)?;
        let sol_withdraw_amount = sol_amount
            .checked_sub(sol_fee_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        // Step 4: Withdraw
        fund.withdrawal_status.withdraw(sol_withdraw_amount)?;
        ctx.accounts.fund_account.sub_lamports(sol_withdraw_amount)?;
        ctx.accounts.user.add_lamports(sol_withdraw_amount)?;

        emit!(UserWithdrewSOLFromFund {
            receipt_token_mint: ctx.accounts.receipt_token_mint.key(),
            fund_account: FundAccountInfo::new(
                ctx.accounts.fund_account.as_ref(),
                receipt_token_price,
                receipt_token_total_supply
            ),
            request_id,
            user_fund_account: Clone::clone(&ctx.accounts.user_fund_account),
            user: ctx.accounts.user.key(),
            burnt_receipt_token_amount: request.receipt_token_amount,
            withdrawn_sol_amount: sol_withdraw_amount,
            deducted_sol_fee_amount: sol_fee_amount,
        });

        Ok(())
    }
}
