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

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    // TODO: use address lookup table!
    #[account(address = BSOL_STAKE_POOL_ADDRESS)]
    /// CHECK: will be checked and deserialized when needed
    pub token_pricing_source_0: UncheckedAccount<'info>,

    // TODO: use address lookup table!
    #[account(address = MSOL_STAKE_POOL_ADDRESS)]
    /// CHECK: will be checked and deserialized when needed
    pub token_pricing_source_1: UncheckedAccount<'info>,
}

impl<'info> FundUpdatePrice<'info> {
    pub fn update_price(ctx: Context<Self>) -> Result<()> {
        let fund = &mut ctx.accounts.fund;
        let receipt_token_mint = &mut ctx.accounts.receipt_token_mint;
        let sources = [
            ctx.accounts.token_pricing_source_0.as_ref(),
            ctx.accounts.token_pricing_source_1.as_ref(),
        ];
        fund.update_token_prices(&sources)?;
        let receipt_token_total_supply = receipt_token_mint.supply;
        let receipt_token_price =
            fund.receipt_token_price(receipt_token_mint.decimals, receipt_token_total_supply)?;

        emit!(FundPriceUpdated {
            receipt_token_mint: receipt_token_mint.key(),
            receipt_token_price,
            fund_info: FundInfo::new_from_fund(fund),
        });

        Ok(())
    }
}
