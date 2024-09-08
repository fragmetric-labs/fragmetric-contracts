use anchor_lang::{prelude::*, solana_program::sysvar::instructions as instructions_sysvar};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::modules::{common::*, fund::*, reward::*};

#[derive(Accounts)]
pub struct UserFundSupportedTokenContext<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub receipt_token_program: Program<'info, Token2022>,

    pub supported_token_program: Interface<'info, TokenInterface>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [ReceiptTokenMintAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = receipt_token_mint_authority.bump,
        has_one = receipt_token_mint,
    )]
    pub receipt_token_mint_authority: Box<Account<'info, ReceiptTokenMintAuthority>>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = receipt_token_mint,
        associated_token::token_program = receipt_token_program,
        associated_token::authority = user,
    )]
    pub user_receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub supported_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [SupportedTokenAuthority::SEED, receipt_token_mint.key().as_ref(), supported_token_mint.key().as_ref()],
        bump = supported_token_authority.bump,
        has_one = receipt_token_mint,
        has_one = supported_token_mint,
    )]
    pub supported_token_authority: Box<Account<'info, SupportedTokenAuthority>>,

    #[account(
        mut,
        token::mint = supported_token_mint,
        token::token_program = supported_token_program,
        token::authority = supported_token_authority,
        seeds = [SupportedTokenAuthority::TOKEN_ACCOUNT_SEED, receipt_token_mint.key().as_ref(), supported_token_mint.key().as_ref()],
        bump,
    )]
    pub supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = supported_token_mint,
        token::token_program = supported_token_program,
        token::authority = user.key(),
    )]
    pub user_supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

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
    pub instruction_sysvar: UncheckedAccount<'info>,
}

impl<'info> UserFundSupportedTokenContext<'info> {
    pub fn deposit_supported_token(
        mut ctx: Context<Self>,
        amount: u64,
        metadata: Option<DepositMetadata>,
    ) -> Result<()> {
        // verify metadata signature if given
        if let Some(metadata) = &metadata {
            verify_preceding_ed25519_instruction(
                &ctx.accounts.instruction_sysvar,
                metadata.try_to_vec()?.as_slice(),
            )?;
        }
        let (wallet_provider, contribution_accrual_rate) = metadata
            .map(|metadata| (metadata.wallet_provider, metadata.contribution_accrual_rate))
            .unzip();

        // Check balance
        require_gte!(ctx.accounts.user_supported_token_account.amount, amount);

        // Step 1: Calculate mint amount
        ctx.accounts
            .fund_account
            .update_token_prices(ctx.remaining_accounts)?;

        let receipt_token_total_supply = ctx.accounts.receipt_token_mint.supply;
        let supported_token_mint = ctx.accounts.supported_token_mint.key();
        let supported_token = ctx
            .accounts
            .fund_account
            .supported_token(supported_token_mint)?;

        let token_amount_to_sol_value = supported_token.calculate_sol_from_tokens(amount)?;
        let receipt_token_mint_amount = ctx
            .accounts
            .fund_account
            .receipt_token_mint_amount_for(token_amount_to_sol_value, receipt_token_total_supply)?;
        let receipt_token_price = ctx
            .accounts
            .fund_account
            .receipt_token_sol_value_per_token(
                ctx.accounts.receipt_token_mint.decimals,
                receipt_token_total_supply,
            )?;

        // Step 2: Deposit Token
        Self::cpi_transfer_token_to_fund(&ctx, amount)?;
        ctx.accounts
            .fund_account
            .supported_token_mut(supported_token_mint)?
            .deposit_token(amount)?;

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

        emit!(UserDepositedSupportedTokenToFund {
            user: ctx.accounts.user.key(),
            user_receipt_token_account: ctx.accounts.user_receipt_token_account.key(),
            user_fund_account: Clone::clone(&ctx.accounts.user_fund_account),
            supported_token_mint: ctx.accounts.supported_token_mint.key(),
            supported_token_user_account: ctx.accounts.user_supported_token_account.key(),
            deposited_supported_token_amount: amount,
            receipt_token_mint: ctx.accounts.fund_account.receipt_token_mint.key(),
            minted_receipt_token_amount: receipt_token_mint_amount,
            wallet_provider,
            contribution_accrual_rate,
            fund_account: FundAccountInfo::new(
                &ctx.accounts.fund_account,
                receipt_token_price,
                receipt_token_total_supply
            ),
        });

        Ok(())
    }

    fn cpi_transfer_token_to_fund(ctx: &Context<Self>, amount: u64) -> Result<()> {
        let token_transfer_cpi_ctx = CpiContext::new(
            ctx.accounts.supported_token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.user_supported_token_account.to_account_info(),
                to: ctx.accounts.supported_token_account.to_account_info(),
                mint: ctx.accounts.supported_token_mint.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );

        transfer_checked(
            token_transfer_cpi_ctx,
            amount,
            ctx.accounts.supported_token_mint.decimals,
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

    fn mock_transfer_hook_from_fund_to_user(
        ctx: &mut Context<Self>,
        amount: u64,
        contribution_accrual_rate: Option<u8>,
    ) -> Result<()> {
        let current_slot = Clock::get()?.slot;

        let mut reward_account = ctx.accounts.reward_account.load_mut()?;
        let mut user_reward_account = ctx.accounts.user_reward_account.load_mut()?;
        let (from_user_update, to_user_update) = reward_account
            .update_reward_pools_token_allocation(
                ctx.accounts.receipt_token_mint.key(),
                amount,
                contribution_accrual_rate,
                None,
                Some(&mut user_reward_account),
                current_slot,
            )?;

        emit!(UserUpdatedRewardPool::new(
            ctx.accounts.receipt_token_mint.key(),
            from_user_update.into_iter().chain(to_user_update).collect(),
        ));

        Ok(())
    }
}
