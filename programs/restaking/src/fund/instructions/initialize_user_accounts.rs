use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::{common::*, constants::*, fund::*, reward::*};

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

    #[account(
        init_if_needed,
        payer = user,
        seeds = [UserRewardAccount::SEED, user.key().as_ref()],
        bump,
        space = 8 + UserRewardAccount::INIT_SPACE,
    )]
    pub user_reward_account: Box<Account<'info, UserRewardAccount>>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundInitializeUserAccounts<'info> {
    pub fn initialize_user_accounts(ctx: Context<Self>) -> Result<()> {
        // Initialize
        ctx.accounts.user_receipt.initialize_if_needed(
            ctx.bumps.user_receipt,
            ctx.accounts.user.key(),
            ctx.accounts.receipt_token_mint.key(),
        );
        ctx.accounts
            .user_reward_account
            .initialize_if_needed(ctx.bumps.user_reward_account, ctx.accounts.user.key());

        Ok(())
    }
}
