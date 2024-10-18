use anchor_lang::prelude::*;
use anchor_lang::{solana_program::sysvar::instructions as instructions_sysvar, system_program};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::modules::{common::*, fund::*, reward::*};
use crate::utils::{AccountLoaderExt, PDASeeds};

#[derive(Accounts)]
pub struct UserFundReceiptTokenAccountInitialContext<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub receipt_token_program: Program<'info, Token2022>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = user,
        associated_token::mint = receipt_token_mint,
        associated_token::token_program = receipt_token_program,
        associated_token::authority = user,
    )]
    pub user_receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
}

impl<'info> UserFundReceiptTokenAccountInitialContext<'info> {
    pub fn initialize_receipt_token_account(_ctx: Context<Self>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct UserFundAccountInitialContext<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = user,
        seeds = [UserFundAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump,
        space = 8 + UserFundAccount::INIT_SPACE,
    )]
    pub user_fund_account: Box<Account<'info, UserFundAccount>>,
}

impl<'info> UserFundAccountInitialContext<'info> {
    pub fn initialize_fund_account(ctx: Context<Self>) -> Result<()> {
        ctx.accounts.user_fund_account.initialize_if_needed(
            ctx.bumps.user_fund_account,
            ctx.accounts.receipt_token_mint.key(),
            ctx.accounts.user.key(),
        );

        Ok(())
    }
}

#[derive(Accounts)]
pub struct UserFundUpdateContext<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [UserFundAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump = user_fund_account.bump,
    )]
    pub user_fund_account: Box<Account<'info, UserFundAccount>>,
}

impl<'info> UserFundUpdateContext<'info> {
    pub fn update_fund_account_if_needed(ctx: Context<Self>) -> Result<()> {
        let bump = ctx.accounts.user_fund_account.bump;
        ctx.accounts.user_fund_account.initialize_if_needed(
            bump,
            ctx.accounts.receipt_token_mint.key(),
            ctx.accounts.user.key(),
        );

        Ok(())
    }
}

#[derive(Accounts)]
pub struct UserFundContext<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

    // pub associated_token_program: Program<'info, AssociatedToken>,
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
        mut,
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
        mut,
        seeds = [UserFundAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump = user_fund_account.bump,
    )]
    pub user_fund_account: Box<Account<'info, UserFundAccount>>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.bump()?,
        has_one = receipt_token_mint,
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,

    #[account(
        mut,
        seeds = [UserRewardAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump = user_reward_account.bump()?,
        has_one = receipt_token_mint,
        has_one = user,
    )]
    pub user_reward_account: AccountLoader<'info, UserRewardAccount>,

    /// CHECK: This is safe that checks it's ID
    #[account(address = instructions_sysvar::ID)]
    pub instructions_sysvar: UncheckedAccount<'info>,
}

impl<'info> UserFundContext<'info> {
    pub fn deposit_sol(
        ctx: Context<Self>,
        amount: u64,
        metadata: Option<DepositMetadata>,
    ) -> Result<()> {
        let fund = &mut ctx.accounts.fund_account;
        let receipt_token_mint = &ctx.accounts.receipt_token_mint;

        // verify metadata signature if given
        if let Some(metadata) = &metadata {
            verify_preceding_ed25519_instruction(
                &ctx.accounts.instructions_sysvar,
                metadata.try_to_vec()?.as_slice(),
            )?;
            metadata.verify_expiration()?;
        }
        let (wallet_provider, contribution_accrual_rate) = metadata
            .map(|metadata| (metadata.wallet_provider, metadata.contribution_accrual_rate))
            .unzip();

        // Check balance
        require_gte!(ctx.accounts.user.lamports(), amount);

        // Step 1: Calculate mint amount
        fund.update_token_prices(ctx.remaining_accounts)?;

        let receipt_token_mint_amount =
            fund.receipt_token_mint_amount_for(amount, receipt_token_mint.supply)?;
        let receipt_token_price = fund.receipt_token_sol_value_per_token(
            receipt_token_mint.decimals,
            receipt_token_mint.supply,
        )?;

        // Step 2: Deposit SOL
        fund.deposit_sol(amount)?;
        ctx.accounts.cpi_transfer_sol_to_fund(amount)?;

        // Step 3: Mint receipt token
        ctx.accounts
            .cpi_mint_token_to_user(receipt_token_mint_amount)?;
        ctx.accounts
            .mock_transfer_hook_from_fund_to_user(receipt_token_mint_amount, contribution_accrual_rate)?;
        ctx.accounts.update_user_receipt_token_amount();

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
                ctx.accounts.receipt_token_mint.supply,
            ),
        });

        Ok(())
    }

    fn cpi_transfer_sol_to_fund(&self, amount: u64) -> Result<()> {
        let sol_transfer_cpi_ctx = CpiContext::new(
            self.system_program.to_account_info(),
            system_program::Transfer {
                from: self.user.to_account_info(),
                to: self.fund_account.to_account_info(),
            },
        );

        system_program::transfer(sol_transfer_cpi_ctx, amount)
            .map_err(|_| error!(ErrorCode::FundSOLTransferFailedException))
    }

    fn mock_transfer_hook_from_fund_to_user(
        &mut self,
        amount: u64,
        contribution_accrual_rate: Option<u8>, // 100 -> 1.0
    ) -> Result<()> {
        let current_slot = Clock::get()?.slot;

        let mut reward_account = self.reward_account.load_mut()?;
        let mut user_reward_account = self.user_reward_account.load_mut()?;
        reward_account.update_reward_pools_token_allocation(
            self.receipt_token_mint.key(),
            amount,
            contribution_accrual_rate,
            None,
            Some(&mut user_reward_account),
            current_slot,
        )?;

        emit!(UserUpdatedRewardPool::new(
            self.receipt_token_mint.key(),
            vec![self.user_reward_account.key()],
        ));

        Ok(())
    }

    fn update_user_receipt_token_amount(&mut self) {
        let receipt_token_account_total_amount = self.user_receipt_token_account.amount;
        self.user_fund_account
            .set_receipt_token_amount(receipt_token_account_total_amount);
    }

    pub fn request_withdrawal(ctx: Context<Self>, receipt_token_amount: u64) -> Result<()> {
        let user_fund_account = &mut ctx.accounts.user_fund_account;
        let withdrawal_status = &mut ctx.accounts.fund_account.withdrawal_status;

        // Verify
        require_gte!(
            ctx.accounts.user_receipt_token_account.amount,
            receipt_token_amount
        );

        // Step 1: Create withdrawal request
        let withdrawal_request =
            user_fund_account.create_withdrawal_request(withdrawal_status, receipt_token_amount)?;
        let batch_id = withdrawal_request.batch_id;
        let request_id = withdrawal_request.request_id;

        // Step 2: Lock receipt token
        ctx.accounts
            .cpi_burn_token_from_user(receipt_token_amount)?;
        ctx.accounts.cpi_mint_token_to_fund(receipt_token_amount)?;
        ctx.accounts.update_user_receipt_token_amount();
        ctx.accounts
            .mock_transfer_hook_from_user_to_null(receipt_token_amount)?;

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

    fn cpi_burn_token_from_user(&mut self, amount: u64) -> Result<()> {
        self.receipt_token_program
            .burn_token_cpi(
                &mut self.receipt_token_mint,
                &mut self.user_receipt_token_account,
                self.user.to_account_info(),
                None,
                amount,
            )
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))
    }

    fn cpi_mint_token_to_fund(&mut self, amount: u64) -> Result<()> {
        self.receipt_token_program
            .mint_token_cpi(
                &mut self.receipt_token_mint,
                &mut self.receipt_token_lock_account,
                self.receipt_token_mint_authority.to_account_info(),
                Some(&[self.receipt_token_mint_authority.signer_seeds().as_ref()]),
                amount,
            )
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))
    }

    fn mock_transfer_hook_from_user_to_null(&mut self, amount: u64) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        let mut reward_account = self.reward_account.load_mut()?;
        let mut user_reward_account = self.user_reward_account.load_mut()?;
        reward_account.update_reward_pools_token_allocation(
            self.receipt_token_mint.key(),
            amount,
            None,
            Some(&mut user_reward_account),
            None,
            current_slot,
        )?;

        emit!(UserUpdatedRewardPool::new(
            self.receipt_token_mint.key(),
            vec![self.user_reward_account.key()],
        ));

        Ok(())
    }

    pub fn cancel_withdrawal_request(ctx: Context<Self>, request_id: u64) -> Result<()> {
        let user_fund_account = &mut ctx.accounts.user_fund_account;
        let withdrawal_status = &mut ctx.accounts.fund_account.withdrawal_status;

        // Verify
        require_gt!(
            withdrawal_status.next_request_id,
            request_id,
            ErrorCode::FundWithdrawalRequestNotFoundError,
        );

        // Step 1: Cancel withdrawal request
        let request = user_fund_account.cancel_withdrawal_request(withdrawal_status, request_id)?;

        // Step 2: Unlock receipt token
        ctx.accounts
            .cpi_burn_token_from_fund(request.receipt_token_amount)?;
        ctx.accounts
            .cpi_mint_token_to_user(request.receipt_token_amount)?;
        ctx.accounts.update_user_receipt_token_amount();
        ctx.accounts
            .mock_transfer_hook_from_null_to_user(request.receipt_token_amount)?;

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

    fn cpi_burn_token_from_fund(&mut self, amount: u64) -> Result<()> {
        self.receipt_token_program
            .burn_token_cpi(
                &mut self.receipt_token_mint,
                &mut self.receipt_token_lock_account,
                self.receipt_token_lock_authority.to_account_info(),
                Some(&[self.receipt_token_lock_authority.signer_seeds().as_ref()]),
                amount,
            )
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))
    }

    fn cpi_mint_token_to_user(&mut self, amount: u64) -> Result<()> {
        self.receipt_token_program
            .mint_token_cpi(
                &mut self.receipt_token_mint,
                &mut self.user_receipt_token_account,
                self.receipt_token_mint_authority.to_account_info(),
                Some(&[self.receipt_token_mint_authority.signer_seeds().as_ref()]),
                amount,
            )
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))
    }

    fn mock_transfer_hook_from_null_to_user(&mut self, amount: u64) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        let mut reward_account = self.reward_account.load_mut()?;
        let mut user_reward_account = self.user_reward_account.load_mut()?;
        reward_account.update_reward_pools_token_allocation(
            self.receipt_token_mint.key(),
            amount,
            None,
            None,
            Some(&mut user_reward_account),
            current_slot,
        )?;

        emit!(UserUpdatedRewardPool::new(
            self.receipt_token_mint.key(),
            vec![self.user_reward_account.key()],
        ));

        Ok(())
    }

    pub fn withdraw(ctx: Context<Self>, request_id: u64) -> Result<()> {
        let fund = &mut ctx.accounts.fund_account;
        let user_fund_account = &mut ctx.accounts.user_fund_account;
        let receipt_token_mint = &ctx.accounts.receipt_token_mint;

        // Verify
        require_gt!(fund.withdrawal_status.next_request_id, request_id);

        // Step 1: Complete withdrawal request
        let request = user_fund_account
            .pop_completed_withdrawal_request(&mut fund.withdrawal_status, request_id)?;

        // Step 2: Calculate withdraw amount
        let sol_amount = fund
            .withdrawal_status
            .reserved_fund
            .calculate_sol_amount_for_receipt_token_amount(request.receipt_token_amount)?;
        let sol_fee_amount = fund
            .withdrawal_status
            .calculate_sol_withdrawal_fee(sol_amount)?;
        let sol_withdraw_amount = sol_amount
            .checked_sub(sol_fee_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        // Step 3: Withdraw
        fund.withdrawal_status.withdraw(
            sol_amount,
            sol_fee_amount,
            request.receipt_token_amount,
        )?;
        ctx.accounts
            .fund_account
            .sub_lamports(sol_withdraw_amount)?;
        ctx.accounts.user.add_lamports(sol_withdraw_amount)?;

        emit!(UserWithdrewSOLFromFund {
            receipt_token_mint: ctx.accounts.receipt_token_mint.key(),
            fund_account: FundAccountInfo::new(
                ctx.accounts.fund_account.as_ref(),
                ctx.accounts
                    .fund_account
                    .receipt_token_sol_value_per_token(
                        ctx.accounts.receipt_token_mint.decimals,
                        ctx.accounts.receipt_token_mint.supply,
                    )?,
                receipt_token_mint.supply
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
