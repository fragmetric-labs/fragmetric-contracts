use anchor_lang::prelude::*;

use crate::constants::FUND_REVENUE_ADDRESS;
use crate::errors;
use crate::modules::fund::{
    FundService, WeightedAllocationParticipant, WeightedAllocationStrategy,
};
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

                    let receipt_token_amount_to_process =
                        fund_service.get_receipt_token_withdrawal_obligated_amount();

                    let mut required_accounts =
                        fund_service.find_accounts_to_process_withdrawal_batch()?;

                    let mut command = self.clone();
                    command.state = ProcessWithdrawalBatchCommandState::Process(
                        receipt_token_amount_to_process,
                    );

                    (command, required_accounts)
                };

                // to harvest fund revenue (prepended)
                required_accounts.insert(0, (FUND_REVENUE_ADDRESS, false));

                // to calculate LST cycle fee (appended)
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

                // to calculate VRT cycle fee (appended)
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
                let [fund_revenue_account, receipt_token_program, receipt_token_lock_account, fund_reserve_account, fund_treasury_account, remaining_accounts @ ..] =
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

                let (uninitialized_withdrawal_batch_accounts, remaining_accounts) =
                    remaining_accounts.split_at(ctx.fund_account.withdrawal.queued_batches.len());

                let (supported_token_pricing_sources, remaining_accounts) =
                    remaining_accounts.split_at(ctx.fund_account.supported_tokens.len());

                let (restaking_vault_pricing_sources, pricing_sources) =
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
                                let account = restaking_vault_pricing_sources[i];
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
                    if lrt_max_cycle_fee_rate_bps > u16::MAX as f32 {
                        err!(errors::ErrorCode::OperationCommandExecutionFailedException)?;
                    }
                    ctx.fund_account
                        .withdrawal
                        .set_sol_fee_rate_bps(lrt_max_cycle_fee_rate_bps as u16)?;
                }

                // do process withdrawal
                let (receipt_token_amount_processed, pricing_service) = {
                    let mut fund_service =
                        FundService::new(ctx.receipt_token_mint, ctx.fund_account)?;
                    let receipt_token_amount_processed = fund_service.process_withdrawal_batch(
                        ctx.operator,
                        ctx.system_program,
                        receipt_token_program,
                        receipt_token_lock_account,
                        fund_reserve_account,
                        fund_treasury_account,
                        uninitialized_withdrawal_batch_accounts,
                        pricing_sources,
                        self.forced,
                        receipt_token_amount_to_process,
                    )?;

                    fund_service.harvest_from_treasury_account(
                        ctx.system_program,
                        fund_treasury_account,
                        fund_revenue_account,
                    )?;

                    let pricing_service =
                        fund_service.new_pricing_service(pricing_sources.into_iter().cloned())?;
                    (receipt_token_amount_processed, pricing_service)
                };

                // adjust accumulated deposit capacity configuration as much as the SOL amount withdrawn
                // the policy is: half to SOL cap, half to LST caps based on their weighted allocation strategy
                let receipt_token_amount_processed_as_sol = pricing_service
                    .get_token_amount_as_sol(
                        &ctx.receipt_token_mint.key(),
                        receipt_token_amount_processed,
                    )?;
                let mut participants = ctx
                    .fund_account
                    .supported_tokens
                    .iter()
                    .map(|supported_token| {
                        Ok(WeightedAllocationParticipant::new(
                            supported_token.sol_allocation_weight,
                            pricing_service.get_token_amount_as_sol(
                                &supported_token.mint,
                                supported_token.operation_reserved_amount,
                            )?,
                            pricing_service.get_token_amount_as_sol(
                                &supported_token.mint,
                                supported_token.sol_allocation_capacity_amount,
                            )?,
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;

                let mut supported_token_increasing_capacity =
                    receipt_token_amount_processed_as_sol.div_ceil(2);
                supported_token_increasing_capacity -= WeightedAllocationStrategy::put(
                    &mut *participants,
                    receipt_token_amount_processed_as_sol,
                );
                for (i, participant) in participants.iter().enumerate() {
                    let supported_token = &mut ctx.fund_account.supported_tokens[i];
                    supported_token.set_accumulated_deposit_capacity_amount(
                        supported_token
                            .accumulated_deposit_capacity_amount
                            .saturating_add(pricing_service.get_sol_amount_as_token(
                                &supported_token.mint,
                                participant.get_last_put_amount()?,
                            )?),
                    )?;
                }

                let sol_increasing_capacity =
                    receipt_token_amount_processed_as_sol - supported_token_increasing_capacity;
                ctx.fund_account.sol_accumulated_deposit_capacity_amount = ctx
                    .fund_account
                    .sol_accumulated_deposit_capacity_amount
                    .saturating_add(sol_increasing_capacity);
            }
        }

        // TODO: ProcessWithdrawalBatchCommand.execute
        Ok(None)
    }
}
