use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{common::*, constants::*, fund::*};

#[derive(Accounts)]
pub struct FundUpdatePrice<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [Fund::SEED, receipt_token_mint.key().as_ref()],
        bump = fund.bump,
        has_one = receipt_token_mint,
    )]
    pub fund: Box<Account<'info, Fund>>,

    #[account(
        seeds = [FundTokenAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_token_authority.bump,
        has_one = receipt_token_mint,
    )]
    pub fund_token_authority: Account<'info, FundTokenAuthority>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    // TODO: use address lookup table!
    // TODO: rename properly!
    // TODO: use address constraint!
    /// CHECK: will be checked and deserialized when needed
    pub pricing_source0: UncheckedAccount<'info>,

    // TODO: use address lookup table!
    // TODO: rename properly!
    // TODO: use address constraint!
    /// CHECK: will be checked and deserialized when needed
    pub pricing_source1: UncheckedAccount<'info>,

    // TODO: use address lookup table!
    // TODO: rename properly!
    // TODO: use address constraint!
    /// CHECK: will be checked and deserialized when needed
    pub pricing_source2: UncheckedAccount<'info>,
}

impl<'info> FundUpdatePrice<'info> {
    pub fn update_price(ctx: Context<Self>) -> Result<()> {
        let fund = &mut ctx.accounts.fund;
        let receipt_token_mint = &mut ctx.accounts.receipt_token_mint;
        let sources = [
            ctx.accounts.pricing_source0.as_ref(),
            ctx.accounts.pricing_source1.as_ref(),
            ctx.accounts.pricing_source2.as_ref(),
        ];
        fund.update_token_prices(&sources)?;
        let receipt_token_total_supply = receipt_token_mint.supply;
        let receipt_token_price =
            fund.receipt_token_price(receipt_token_mint.decimals, receipt_token_total_supply)?;

        emit!(FundPriceUpdated {
            lrt_mint: receipt_token_mint.key(),
            lrt_price: receipt_token_price,
            fund_info: FundInfo::new_from_fund(fund),
        });

        Ok(())
    }
}
