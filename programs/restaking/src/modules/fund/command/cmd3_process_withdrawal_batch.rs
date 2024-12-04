use anchor_lang::prelude::*;

use crate::constants::FUND_REVENUE_ADDRESS;
use crate::errors;
use crate::modules::fund::FundService;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::JitoRestakingVaultService;
use crate::modules::staking::{MarinadeStakePoolService, SPLStakePoolService};

use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct ProcessWithdrawalBatchCommand {
    state: ProcessWithdrawalBatchCommandState,
    forced: bool,
}

impl From<ProcessWithdrawalBatchCommand> for OperationCommand {
    fn from(command: ProcessWithdrawalBatchCommand) -> Self {
        Self::ProcessWithdrawalBatch(command)
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum ProcessWithdrawalBatchCommandState {
    #[default]
    New,
    /// max receipt_token_amount to process withdrawal
    Process(u64),
    HarvestRevenue,
}

impl SelfExecutable for ProcessWithdrawalBatchCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        match self.state {
            ProcessWithdrawalBatchCommandState::New => {
                let (command, mut required_accounts) = {
                    let mut fund_service =
                        FundService::new(ctx.receipt_token_mint, ctx.fund_account)?;
                    let pricing_service =
                        fund_service.new_pricing_service(accounts.into_iter().cloned())?;

                    let receipt_token_amount_to_process =
                        fund_service.get_sol_withdrawal_execution_amount(&pricing_service)?;
                    let mut required_accounts =
                        fund_service.find_accounts_to_process_withdrawal_batch()?;

                    let mut command = self.clone();
                    command.state = ProcessWithdrawalBatchCommandState::Process(
                        receipt_token_amount_to_process,
                    );

                    (command, required_accounts)
                };

                // to calculate LST cycle fee
                for supported_token in ctx.fund_account.supported_tokens.iter() {
                    match &supported_token.pricing_source {
                        TokenPricingSource::MarinadeStakePool { address } => {
                            required_accounts.push((*address, false));
                        }
                        TokenPricingSource::SPLStakePool { address } => {
                            required_accounts.push((*address, false));
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    }
                }

                // to calculate VRT cycle fee
                for restaking_vault in ctx.fund_account.restaking_vaults.iter() {
                    match &restaking_vault.receipt_token_pricing_source {
                        TokenPricingSource::JitoRestakingVault { address } => {
                            required_accounts.push((*address, false));
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    }
                }

                return Ok(Some(command.with_required_accounts(required_accounts)));
            }
            ProcessWithdrawalBatchCommandState::Process(receipt_token_amount_to_process) => {
                let [receipt_token_program, receipt_token_lock_account, fund_reserve_account, treasury_account, remaining_accounts @ ..] =
                    accounts
                else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };

                if remaining_accounts.len()
                    < ctx.fund_account.withdrawal.queued_batches.len()
                        + ctx.fund_account.supported_tokens.len()
                        + ctx.fund_account.restaking_vaults.len()
                {
                    err!(ErrorCode::AccountNotEnoughKeys)?;
                }

                let (uninitialized_batch_withdrawal_tickets, remaining_accounts) =
                    remaining_accounts.split_at(ctx.fund_account.withdrawal.queued_batches.len());

                let (supported_token_pricing_sources, remaining_accounts) =
                    remaining_accounts.split_at(ctx.fund_account.supported_tokens.len());

                let (restaing_vault_pricing_sources, pricing_sources) =
                    remaining_accounts.split_at(ctx.fund_account.restaking_vaults.len());

                // calculate LST max cycle fee
                let mut lst_max_cycle_fee_numerator = 0u64;
                let mut lst_max_cycle_fee_denominator = 0u64;
                for (i, supported_token) in ctx.fund_account.supported_tokens.iter().enumerate() {
                    let (numerator, denominator) = match &supported_token.pricing_source {
                        TokenPricingSource::MarinadeStakePool { address } => {
                            let account = supported_token_pricing_sources[i];
                            require_keys_eq!(account.key(), *address);
                            MarinadeStakePoolService::get_max_cycle_fee(account)?
                        }
                        TokenPricingSource::SPLStakePool { address } => {
                            let account = supported_token_pricing_sources[i];
                            require_keys_eq!(account.key(), *address);
                            SPLStakePoolService::get_max_cycle_fee(account)?
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    };

                    // numerator/denominator > lst_max_cycle_fee_numerator/lst_max_cycle_fee_denominator
                    if denominator != 0
                        || numerator * lst_max_cycle_fee_denominator
                            > lst_max_cycle_fee_numerator * denominator
                    {
                        lst_max_cycle_fee_numerator = numerator;
                        lst_max_cycle_fee_denominator = denominator;
                    }
                }

                // calculate VRT max cycle fee
                let mut vrt_max_cycle_fee_numerator = 0u64;
                let mut vrt_max_cycle_fee_denominator = 0u64;
                for (i, restaking_vault) in ctx.fund_account.restaking_vaults.iter().enumerate() {
                    let (numerator, denominator) =
                        match &restaking_vault.receipt_token_pricing_source {
                            TokenPricingSource::JitoRestakingVault { address } => {
                                let account = restaing_vault_pricing_sources[i];
                                require_keys_eq!(account.key(), *address);
                                JitoRestakingVaultService::get_max_cycle_fee(account)?
                            }
                            _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                        };
                    // numerator/denominator > vrt_max_cycle_fee_numerator/vrt_max_cycle_fee_denominator
                    if denominator != 0
                        || numerator * vrt_max_cycle_fee_denominator
                            > vrt_max_cycle_fee_numerator * denominator
                    {
                        vrt_max_cycle_fee_numerator = numerator;
                        vrt_max_cycle_fee_denominator = denominator;
                    }
                }

                // calculate LRT max cycle fee to ensure withdrawal fee is equal or greater than max fee expense during cash-out
                let mut lrt_max_cycle_fee_rate = 1.0
                    - (1.0
                        - (lst_max_cycle_fee_numerator as f32
                            / lst_max_cycle_fee_denominator.max(1) as f32))
                        * (1.0
                            - (vrt_max_cycle_fee_numerator as f32
                                / vrt_max_cycle_fee_denominator.max(1) as f32));
                let withdrawal_fee_rate =
                    ctx.fund_account.withdrawal.sol_fee_rate_bps as f32 / 10_000.0;

                // adjust withdrawal fee rate
                if lrt_max_cycle_fee_rate > withdrawal_fee_rate {
                    let lrt_max_cycle_fee_rate_bps = (lrt_max_cycle_fee_rate * 10_000.0).ceil();
                    if lrt_max_cycle_fee_rate_bps > u32::MAX as f32 {
                        err!(errors::ErrorCode::OperationCommandExecutionFailedException)?;
                    }
                    ctx.fund_account
                        .withdrawal
                        .set_sol_fee_rate_bps(lrt_max_cycle_fee_rate_bps as u16)?;
                }

                // do process withdrawal
                FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                    .process_withdrawal_batch(
                        ctx.operator,
                        ctx.system_program,
                        receipt_token_program,
                        receipt_token_lock_account,
                        fund_reserve_account,
                        treasury_account,
                        uninitialized_batch_withdrawal_tickets,
                        pricing_sources,
                        self.forced,
                        receipt_token_amount_to_process,
                    )?;

                let mut command = self.clone();
                command.state = ProcessWithdrawalBatchCommandState::HarvestRevenue;
                return Ok(Some(command.with_required_accounts(vec![
                    (treasury_account.key(), true),
                    (FUND_REVENUE_ADDRESS, false),
                ])));
            }
            ProcessWithdrawalBatchCommandState::HarvestRevenue => {
                let [treasury_account, fund_revenue_account, _remaining_accounts @ ..] = accounts
                else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };

                FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                    .harvest_from_treasury_account(
                        ctx.system_program,
                        treasury_account,
                        fund_revenue_account,
                    )?;
            }
        }

        // TODO: ProcessWithdrawalBatchCommand.execute
        Ok(None)
    }
}
