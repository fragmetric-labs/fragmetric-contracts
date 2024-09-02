use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::*;
use crate::modules::common::PDASignerSeeds;
use crate::modules::fund::{FundAccount, SupportedTokenAuthority, TokenPricingSource};

#[derive(Accounts)]
pub struct FundManagerFundSupportedTokenContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.bump,
        has_one = receipt_token_mint,
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,

    pub supported_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub supported_token_program: Interface<'info, TokenInterface>,

    #[account(
        init,
        payer = payer,
        seeds = [SupportedTokenAuthority::SEED, receipt_token_mint.key().as_ref(), supported_token_mint.key().as_ref()],
        bump,
        space = 8 + SupportedTokenAuthority::INIT_SPACE,
    )]
    pub fund_supported_token_authority: Box<Account<'info, SupportedTokenAuthority>>,

    #[account(
        init,
        payer = payer,
        token::mint = supported_token_mint,
        token::authority = fund_supported_token_authority,
        token::token_program = supported_token_program,
        seeds = [SupportedTokenAuthority::TOKEN_ACCOUNT_SEED, receipt_token_mint.key().as_ref(), supported_token_mint.key().as_ref()],
        bump,
    )]
    pub fund_supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's lst token account
}

impl<'info> FundManagerFundSupportedTokenContext<'info> {
    pub fn add_supported_token(
        ctx: Context<Self>,
        capacity_amount: u64,
        pricing_source: TokenPricingSource,
    ) -> Result<()> {
        ctx.accounts
            .fund_supported_token_authority
            .initialize_if_needed(
                ctx.bumps.fund_supported_token_authority,
                ctx.accounts.receipt_token_mint.key(),
                ctx.accounts.supported_token_mint.key(),
            );
        ctx.accounts.fund_account
            .add_supported_token(
                ctx.accounts.supported_token_mint.key(),
                ctx.accounts.supported_token_mint.decimals,
                capacity_amount,
                pricing_source,
                ctx.remaining_accounts,
            )?;

        Ok(())
    }
}
