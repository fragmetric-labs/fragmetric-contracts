use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::{common::*, constants::*, fund::*, reward::*, utils::InitIfNeededByUser};

#[derive(Accounts)]
pub struct FundInitializeUserAccounts<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [UserReceipt::SEED, user.key().as_ref(), receipt_token_mint.key().as_ref()],
        bump,
        space = 8 + UserReceipt::INIT_SPACE,
        constraint = user_receipt.data_version == 0 || user_receipt.user == user.key(),
        constraint = user_receipt.data_version == 0 || user_receipt.receipt_token_mint == receipt_token_mint.key(),
    )]
    pub user_receipt: Box<Account<'info, UserReceipt>>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // user's fragSOL token account

    /// CHECK: will create at initialize
    #[account(
        mut,
        seeds = [UserRewardAccount::SEED, user.key().as_ref()],
        bump,
        // constraint = user_reward_account.data_version == 0 || user_reward_account.user == user.key(),
    )]
    pub user_reward_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundInitializeUserAccounts<'info> {
    pub fn initialize(ctx: Context<Self>) -> Result<()> {
        msg!(
            "user: {}, user_reward_account: {}",
            ctx.accounts.user.key(),
            ctx.accounts.user_reward_account.key()
        );

        // Custom deserialize
        let mut user_reward_account = ctx
            .accounts
            .user_reward_account
            .init_if_needed_by_user::<UserRewardAccount>(
                "user_reward_account",
                AsRef::as_ref(&ctx.accounts.user),
                8 + UserRewardAccount::INIT_SPACE,
                &ctx.accounts.system_program,
            )?;
        if user_reward_account.data_version != 0 {
            require_eq!(user_reward_account.bump, ctx.bumps.user_reward_account);
        }

        // Initialize
        ctx.accounts.user_receipt.initialize_if_needed(
            ctx.bumps.user_receipt,
            ctx.accounts.user.key(),
            ctx.accounts.receipt_token_mint.key(),
        );
        user_reward_account
            .initialize_if_needed(ctx.bumps.user_reward_account, ctx.accounts.user.key());

        Ok(())
    }
}
