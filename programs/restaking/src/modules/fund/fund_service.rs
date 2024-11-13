use crate::events;
use crate::modules::fund::{FundAccount, FundAccountInfo};
use crate::modules::pricing;
use crate::modules::pricing::TokenPricingSourceMap;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

pub struct FundService<'info, 'a>
where
    'info: 'a,
{
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    fund_account: &'a mut Account<'info, FundAccount>,
    pricing_sources: &'info [AccountInfo<'info>],
}

impl<'info, 'a> FundService<'info, 'a> {
    pub fn new(
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        fund_account: &'a mut Account<'info, FundAccount>,
        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<Self> {
        Ok(Self {
            receipt_token_mint,
            fund_account,
            pricing_sources,
        })
    }

    // TODO: receive pricing service "to extend pricing source/calculator"?
    pub fn create_pricing_source_map(&self) -> Result<TokenPricingSourceMap<'info>> {
        let mints_and_pricing_sources = self
            .fund_account
            .supported_tokens
            .iter()
            .map(|token| (token.get_mint(), token.get_pricing_source()))
            .collect();

        pricing::create_pricing_source_map(mints_and_pricing_sources, self.pricing_sources)
    }

    pub fn process_update_prices(&mut self) -> Result<()> {
        self.update_asset_prices()?;

        emit!(events::OperatorUpdatedFundPrice {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: FundAccountInfo::from(self.fund_account, self.receipt_token_mint),
        });

        Ok(())
    }

    pub fn update_asset_prices(&mut self) -> Result<()> {
        let pricing_source_map = self.create_pricing_source_map()?;
        self.fund_account
            .supported_tokens
            .iter_mut()
            .try_for_each(|token| token.update_one_token_as_sol(&pricing_source_map))
    }
}
