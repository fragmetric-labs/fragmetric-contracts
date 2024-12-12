use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::*;
use crate::modules::normalization::{NormalizedTokenPoolAccount, NormalizedTokenWithdrawalAccount};
use crate::utils::PDASeeds;

#[derive(Accounts)]
pub struct SlasherNormalizedTokenWithdrawalAccountInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub slasher: Signer<'info>,

    #[account(mut, address = FRAGSOL_NORMALIZED_TOKEN_MINT_ADDRESS)]
    pub normalized_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [NormalizedTokenPoolAccount::SEED, normalized_token_mint.key().as_ref()],
        bump = normalized_token_pool_account.get_bump(),
        has_one = normalized_token_mint,
    )]
    pub normalized_token_pool_account: Box<Account<'info, NormalizedTokenPoolAccount>>,

    pub normalized_token_program: Program<'info, Token>,

    #[account(
        init,
        payer = payer,
        space = 8 + NormalizedTokenWithdrawalAccount::INIT_SPACE,
        seeds = [NormalizedTokenWithdrawalAccount::SEED, normalized_token_mint.key().as_ref(), slasher.key().as_ref()],
        bump,
    )]
    pub slasher_normalized_token_withdrawal_ticket_account:
        Box<Account<'info, NormalizedTokenWithdrawalAccount>>,

    #[account(
        mut,
        token::mint = normalized_token_mint,
        token::token_program = normalized_token_program,
        token::authority = slasher.key(),
    )]
    pub slasher_normalized_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SlasherNormalizedTokenWithdrawContext<'info> {
    #[account(mut)]
    pub slasher: Signer<'info>,

    #[account(mut, address = FRAGSOL_NORMALIZED_TOKEN_MINT_ADDRESS)]
    pub normalized_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [NormalizedTokenPoolAccount::SEED, normalized_token_mint.key().as_ref()],
        bump = normalized_token_pool_account.get_bump(),
        has_one = normalized_token_mint,
    )]
    pub normalized_token_pool_account: Box<Account<'info, NormalizedTokenPoolAccount>>,

    pub normalized_token_program: Program<'info, Token>,

    #[account(
        mut,
        seeds = [NormalizedTokenWithdrawalAccount::SEED, normalized_token_mint.key().as_ref(), slasher.key().as_ref()],
        bump = slasher_normalized_token_withdrawal_ticket_account.get_bump(),
        has_one = normalized_token_mint,
    )]
    pub slasher_normalized_token_withdrawal_ticket_account:
        Box<Account<'info, NormalizedTokenWithdrawalAccount>>,

    pub supported_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub supported_token_program: Interface<'info, TokenInterface>,

    #[account(
        mut,
        associated_token::mint = supported_token_mint,
        associated_token::authority = normalized_token_pool_account,
        associated_token::token_program = supported_token_program,
    )]
    pub normalized_token_pool_supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = supported_token_mint,
        token::token_program = supported_token_program,
    )]
    pub destination_supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    /// CHECK: any destination account to retrieve rent fee when close the ticket after all tokens settled.
    pub destination_rent_lamports_account: UncheckedAccount<'info>,
}
