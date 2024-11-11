use crate::constants::ADMIN_PUBKEY;
use crate::errors::ErrorCode;
use crate::events;
use crate::modules::fund::command::OperationCommandContext;
use crate::modules::fund::{
    FundAccount, FundAccountInfo, UserFundAccount, UserFundConfigurationService,
};
use crate::modules::pricing::{PricingService, TokenPricingSource};
use crate::modules::reward::{RewardAccount, RewardService, UserRewardAccount};
use crate::modules::{fund, pricing};
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
    current_slot: u64,
    current_timestamp: i64,
}

impl Drop for FundService<'_, '_> {
    fn drop(&mut self) {
        self.fund_account.exit(&crate::ID).unwrap();
    }
}

impl<'info, 'a> FundService<'info, 'a> {
    pub fn new(
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        fund_account: &'a mut Account<'info, FundAccount>,
    ) -> Result<Self>
    where
        'info: 'a,
    {
        let clock = Clock::get()?;
        Ok(Self {
            receipt_token_mint,
            fund_account,
            current_slot: clock.slot,
            current_timestamp: clock.unix_timestamp,
        })
    }

    // create a pricing service and register fund assets' value resolvers
    pub(in crate::modules) fn new_pricing_service(
        &mut self,
        pricing_sources: &[AccountInfo<'info>],
    ) -> Result<PricingService<'info>> {
        // ensure any update on fund account written before do pricing
        self.fund_account.exit(&crate::ID)?;

        let mut pricing_service = PricingService::new(pricing_sources)?;
        pricing_service
            .register_token_pricing_source_account(self.fund_account.as_ref())
            .register_token_pricing_source_account(self.receipt_token_mint.as_ref())
            .resolve_token_pricing_source(
                &self.fund_account.receipt_token_mint.key(),
                &TokenPricingSource::FundReceiptToken {
                    mint_address: self.fund_account.receipt_token_mint.key(),
                    fund_address: self.fund_account.key(),
                },
            )?;

        // try to update current underlying assets' price
        self.update_asset_prices(&pricing_service)?;

        Ok(pricing_service)
    }

    // values being updated below are informative, only for event emission.
    fn update_asset_prices(&mut self, pricing_service: &PricingService) -> Result<()> {
        self.fund_account
            .supported_tokens
            .iter_mut()
            .try_for_each(|supported_token| {
                supported_token.one_token_as_sol = pricing_service.get_token_amount_as_sol(
                    &supported_token.get_mint(),
                    10u64
                        .checked_pow(supported_token.get_decimals() as u32)
                        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
                )?;

                Ok::<(), Error>(())
            })?;

        self.fund_account.one_receipt_token_as_sol = pricing_service.get_token_amount_as_sol(
            &self.receipt_token_mint.key(),
            10u64
                .checked_pow(self.receipt_token_mint.decimals as u32)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
        )?;

        Ok(())
    }

    pub fn process_update_prices(
        &mut self,
        token_pricing_source_accounts: &'a [AccountInfo<'info>],
    ) -> Result<()> {
        self.new_pricing_service(token_pricing_source_accounts)?;

        emit!(events::OperatorUpdatedFundPrice {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: FundAccountInfo::from(self.fund_account, self.receipt_token_mint),
        });

        Ok(())
    }

    pub fn process_transfer_hook(
        &self,
        reward_account: &mut AccountLoader<'info, RewardAccount>,
        source_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
        destination_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
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

        // TODO v0.4/transfer: token transfer is temporarily disabled
        err!(ErrorCode::TokenNotTransferableError)?;

        Ok(())
    }

    // TODO v0.3/operation: integrate into lib.rs
    pub fn process_run(&mut self, remaining_accounts: &'info [AccountInfo<'info>]) -> Result<()>
    where
        'info: 'a,
    {
        let mut operation_state = std::mem::take(&mut self.fund_account.operation);

        operation_state.run_commands(
            &OperationCommandContext {
                fund_account: self.fund_account,
                receipt_token_mint: self.receipt_token_mint.key(),
            },
            remaining_accounts.to_vec(),
            self.current_timestamp,
            self.current_slot,
            false,
        )?;

        self.fund_account.operation = operation_state;

        emit!(events::OperatorProcessedJob {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: fund::FundAccountInfo::from(self.fund_account, self.receipt_token_mint),
        });

        Ok(())
    }
}
