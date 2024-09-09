use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::*;
use crate::events::FundManagerUpdatedFund;
use crate::modules::{common::PDASignerSeeds, fund::*};

#[derive(Accounts)]
pub struct FundManagerFundSupportedTokenAuthorityInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub supported_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = payer,
        seeds = [SupportedTokenAuthority::SEED, receipt_token_mint.key().as_ref(), supported_token_mint.key().as_ref()],
        bump,
        space = 8 + SupportedTokenAuthority::INIT_SPACE,
    )]
    pub supported_token_authority: Box<Account<'info, SupportedTokenAuthority>>,
}

impl<'info> FundManagerFundSupportedTokenAuthorityInitialContext<'info> {
    pub fn initialize_supported_token_authority(ctx: Context<Self>) -> Result<()> {
        ctx.accounts.supported_token_authority.initialize_if_needed(
            ctx.bumps.supported_token_authority,
            ctx.accounts.receipt_token_mint.key(),
            ctx.accounts.supported_token_mint.key(),
        );

        Ok(())
    }
}

#[derive(Accounts)]
pub struct FundManagerFundSupportedTokenAccountInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub supported_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub supported_token_program: Interface<'info, TokenInterface>,

    #[account(
        mut,
        seeds = [SupportedTokenAuthority::SEED, receipt_token_mint.key().as_ref(), supported_token_mint.key().as_ref()],
        bump = supported_token_authority.bump,
    )]
    pub supported_token_authority: Box<Account<'info, SupportedTokenAuthority>>,

    #[account(
        init,
        payer = payer,
        token::mint = supported_token_mint,
        token::authority = supported_token_authority,
        token::token_program = supported_token_program,
        seeds = [SupportedTokenAuthority::TOKEN_ACCOUNT_SEED, receipt_token_mint.key().as_ref(), supported_token_mint.key().as_ref()],
        bump,
    )]
    pub supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
}

impl<'info> FundManagerFundSupportedTokenAccountInitialContext<'info> {
    pub fn intialize_supported_token_account(_ctx: Context<Self>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct FundManagerFundSupportedTokenContext<'info> {
    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

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
        mut,
        seeds = [SupportedTokenAuthority::SEED, receipt_token_mint.key().as_ref(), supported_token_mint.key().as_ref()],
        bump = supported_token_authority.bump,
    )]
    pub supported_token_authority: Box<Account<'info, SupportedTokenAuthority>>,

    #[account(
        token::mint = supported_token_mint,
        token::authority = supported_token_authority,
        token::token_program = supported_token_program,
        seeds = [SupportedTokenAuthority::TOKEN_ACCOUNT_SEED, receipt_token_mint.key().as_ref(), supported_token_mint.key().as_ref()],
        bump,
    )]
    pub supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
}

impl<'info> FundManagerFundSupportedTokenContext<'info> {
    pub fn add_supported_token(
        ctx: Context<Self>,
        capacity_amount: u64,
        pricing_source: TokenPricingSource,
    ) -> Result<()> {
        ctx.accounts.fund_account.add_supported_token(
            ctx.accounts.supported_token_mint.key(),
            ctx.accounts.supported_token_program.key.key(),
            ctx.accounts.supported_token_mint.decimals,
            capacity_amount,
            pricing_source,
            ctx.remaining_accounts,
        )?;

        let receipt_token_total_supply = ctx.accounts.receipt_token_mint.supply;
        let receipt_token_price = ctx
            .accounts
            .fund_account
            .receipt_token_sol_value_per_token(
                ctx.accounts.receipt_token_mint.decimals,
                receipt_token_total_supply,
            )?;

        emit!(FundManagerUpdatedFund {
            receipt_token_mint: ctx.accounts.receipt_token_mint.key(),
            fund_account: FundAccountInfo::new(
                &ctx.accounts.fund_account,
                receipt_token_price,
                receipt_token_total_supply
            ),
        });

        Ok(())
    }
}
