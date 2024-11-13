use crate::errors::ErrorCode;
use crate::events;
use crate::modules::fund::{FundAccount, FundAccountInfo, UserFundAccount};
use crate::modules::pricing;
use crate::modules::pricing::TokenPricingSourceMap;
use crate::modules::reward::{RewardAccount, RewardService, UserRewardAccount};
use crate::utils;
use anchor_lang::prelude::*;
use anchor_spl::token::accessor::amount;
use anchor_spl::token_interface::{Mint, TokenAccount};

pub struct FundService<'info, 'a>
where
    'info: 'a,
{
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    fund_account: &'a mut Account<'info, FundAccount>,
    _current_slot: u64,
    _current_timestamp: i64,
}

impl<'info, 'a> FundService<'info, 'a> {
    pub fn new(
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        fund_account: &'a mut Account<'info, FundAccount>,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            receipt_token_mint,
            fund_account,
            _current_slot: clock.slot,
            _current_timestamp: clock.unix_timestamp,
        })
    }

    // TODO: receive pricing service "to extend pricing source/calculator"?
    pub(in crate::modules) fn create_pricing_source_map(
        &self,
        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<TokenPricingSourceMap<'info>> {
        let mints_and_pricing_sources = self
            .fund_account
            .supported_tokens
            .iter()
            .map(|token| (token.get_mint(), token.get_pricing_source()))
            .collect();

        pricing::create_pricing_source_map(mints_and_pricing_sources, pricing_sources)
    }

    pub fn process_update_prices(
        &mut self,
        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<()> {
        self.update_asset_prices(pricing_sources)?;

        emit!(events::OperatorUpdatedFundPrice {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: FundAccountInfo::from(self.fund_account, self.receipt_token_mint),
        });

        Ok(())
    }

    pub(in crate::modules) fn update_asset_prices(
        &mut self,
        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<()> {
        let pricing_source_map = self.create_pricing_source_map(pricing_sources)?;
        self.fund_account
            .supported_tokens
            .iter_mut()
            .try_for_each(|token| token.update_one_token_as_sol(&pricing_source_map))
    }

    pub fn process_transfer_hook(
        &self,
        reward_account: &'a mut AccountLoader<'info, RewardAccount>,
        source_receipt_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,
        destination_receipt_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,
        extra_accounts: &'info [AccountInfo<'info>],
        transfer_amount: u64,
    ) -> Result<()> {
        // parse extra accounts
        let source_fund_account_info = extra_accounts
            .get(0)
            .ok_or(ProgramError::NotEnoughAccountKeys)?;
        let source_fund_account_option =
            utils::parse_optional_account_boxed::<UserFundAccount>(source_fund_account_info)?;
        let source_reward_account_info = extra_accounts
            .get(1)
            .ok_or(ProgramError::NotEnoughAccountKeys)?;
        let mut source_reward_account_option = utils::parse_optional_account_loader_boxed::<
            UserRewardAccount,
        >(source_reward_account_info)?;
        let destination_fund_account_info = extra_accounts
            .get(2)
            .ok_or(ProgramError::NotEnoughAccountKeys)?;
        let destination_fund_account_option =
            utils::parse_optional_account_boxed::<UserFundAccount>(destination_fund_account_info)?;
        let destination_reward_account_info = extra_accounts
            .get(3)
            .ok_or(ProgramError::NotEnoughAccountKeys)?;
        let mut destination_reward_account_option = utils::parse_optional_account_loader_boxed::<
            UserRewardAccount,
        >(destination_reward_account_info)?;

        // transfer source's reward accrual rate to destination
        RewardService::new(self.receipt_token_mint, reward_account)?
            .update_reward_pools_token_allocation(
                source_reward_account_option
                    .as_mut()
                    .map(|account_loader| &mut **account_loader),
                destination_reward_account_option
                    .as_mut()
                    .map(|account_loader| &mut **account_loader),
                transfer_amount,
                None,
            )?;

        // sync user fund accounts
        if let Some(mut source_fund_account) = source_fund_account_option {
            source_fund_account.sync_receipt_token_amount(source_receipt_token_account)?;
            source_fund_account.exit(&crate::ID)?;
        }
        if let Some(mut destination_fund_account) = destination_fund_account_option {
            destination_fund_account
                .sync_receipt_token_amount(destination_receipt_token_account)?;
            destination_fund_account.exit(&crate::ID)?;
        }

        emit!(events::UserTransferredReceiptToken {
            receipt_token_mint: self.receipt_token_mint.key(),
            transferred_receipt_token_amount: transfer_amount,

            source_receipt_token_account: source_receipt_token_account.key(),
            source: source_receipt_token_account.owner,
            source_fund_account: UserFundAccount::placeholder(
                source_receipt_token_account.owner,
                self.receipt_token_mint.key(),
                source_receipt_token_account.amount,
            ),
            destination_receipt_token_account: destination_receipt_token_account.key(),
            destination: destination_receipt_token_account.owner,
            destination_fund_account: UserFundAccount::placeholder(
                destination_receipt_token_account.owner,
                self.receipt_token_mint.key(),
                destination_receipt_token_account.amount,
            ),
        });

        // TODO: token transfer is temporarily disabled
        err!(ErrorCode::TokenNotTransferableError)?;

        Ok(())
    }
}
