use super::{
    OperationCommand, OperationCommandContext, OperationCommandEntry, OperationCommandResult,
    SelfExecutable,
};
use crate::constants::PROGRAM_REVENUE_ADDRESS;
use crate::errors;
use crate::modules::fund::{
    FundService, WeightedAllocationParticipant, WeightedAllocationStrategy,
    FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
};
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::JitoRestakingVaultService;
use crate::modules::staking::{MarinadeStakePoolService, SPLStakePoolService};
use crate::utils::AccountInfoExt;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct ProcessWithdrawalBatchCommand {
    state: ProcessWithdrawalBatchCommandState,
    forced: bool,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum ProcessWithdrawalBatchCommandState {
    /// supported_token_mint, None means SOL ... this is temporary implementation
    New(Option<Pubkey>),
    /// supported_token_mint, max_receipt_token_amount to process withdrawal
    Process(Option<Pubkey>, u64),
}

impl Default for ProcessWithdrawalBatchCommandState {
    fn default() -> Self {
        Self::New(None)
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ProcessWithdrawalBatchCommandResult {
    pub supported_token_mint: Option<Pubkey>,
    pub requested_receipt_token_amount: u64,
    pub processed_receipt_token_amount: u64,
    pub reserved_asset_user_amount: u64,
    pub deducted_asset_fee_amount: u64,
}

impl SelfExecutable for ProcessWithdrawalBatchCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        match self.state {
            ProcessWithdrawalBatchCommandState::New(supported_token_mint_key) => {
                let (receipt_token_amount_to_process, mut required_accounts) = {
                    let fund_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?;

                    let receipt_token_amount_to_process = fund_service
                        .get_receipt_token_withdrawal_obligated_amount(supported_token_mint_key)?;

                    let required_accounts = fund_service
                        .find_accounts_to_process_withdrawal_batches(supported_token_mint_key)?;

                    (receipt_token_amount_to_process, required_accounts)
                };

                // to harvest program revenue (prepended)
                required_accounts.insert(0, (PROGRAM_REVENUE_ADDRESS, true));
                required_accounts.insert(
                    1,
                    supported_token_mint_key
                        .map(|mint| {
                            let fund_account = ctx.fund_account.load()?;
                            let supported_token = fund_account.get_supported_token(&mint)?;
                            Ok::<(Pubkey, bool), Error>((
                        spl_associated_token_account::get_associated_token_address_with_program_id(
                            &PROGRAM_REVENUE_ADDRESS,
                            &supported_token.mint,
                            &supported_token.program,
                        ),
                        true,
                    ))
                        })
                        .unwrap_or_else(|| Ok((Pubkey::default(), false)))?,
                );
                required_accounts.insert(
                    2,
                    supported_token_mint_key
                        .map(|_| (spl_associated_token_account::ID, false))
                        .unwrap_or_else(|| (Pubkey::default(), false)),
                );

                // to calculate LST cycle fee (appended)
                let fund_account = ctx.fund_account.load()?;
                for supported_token in fund_account.get_supported_tokens_iter() {
                    match &supported_token.pricing_source.try_deserialize()? {
                        Some(TokenPricingSource::MarinadeStakePool { address }) => {
                            required_accounts.push((*address, false));
                        }
                        Some(TokenPricingSource::SPLStakePool { address }) => {
                            required_accounts.push((*address, false));
                        }
                        _ => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
                    }
                }

                // to calculate VRT cycle fee (appended)
                for restaking_vault in fund_account.get_restaking_vaults_iter() {
                    match &restaking_vault
                        .receipt_token_pricing_source
                        .try_deserialize()?
                    {
                        Some(TokenPricingSource::JitoRestakingVault { address }) => {
                            required_accounts.push((*address, false));
                        }
                        _ => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
                    }
                }

                let mut command = self.clone();
                command.state = ProcessWithdrawalBatchCommandState::Process(
                    supported_token_mint_key,
                    receipt_token_amount_to_process,
                );

                Ok((
                    None,
                    Some(command.with_required_accounts(required_accounts)),
                ))
            }
            ProcessWithdrawalBatchCommandState::Process(
                supported_token_mint_key,
                requested_receipt_token_amount,
            ) => {
                let [program_revenue_account, program_supported_token_revenue_account, optional_associated_token_account_program, receipt_token_program, receipt_token_lock_account, fund_reserve_account, fund_treasury_account, optional_supported_token_mint, optional_supported_token_program, optional_fund_supported_token_reserve_account, optional_fund_supported_token_treasury_account, remaining_accounts @ ..] =
                    accounts
                else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };

                let fund_account = ctx.fund_account.load()?;
                let num_queued_batches = fund_account
                    .get_asset_state(supported_token_mint_key)?
                    .get_withdrawal_queued_batches_iter()
                    .count();
                let num_supported_token_pricing_sources = fund_account
                    .get_supported_tokens_iter()
                    .map(|supported_token| {
                        match &supported_token.pricing_source.try_deserialize()? {
                            Some(TokenPricingSource::MarinadeStakePool { .. })
                            | Some(TokenPricingSource::SPLStakePool { .. }) => Ok(1),
                            _ => err!(
                                errors::ErrorCode::FundOperationCommandExecutionFailedException
                            )?,
                        }
                    })
                    .collect::<Result<Vec<_>>>()?
                    .iter()
                    .sum();
                let num_restaking_vault_pricing_sources = fund_account
                    .get_restaking_vaults_iter()
                    .map(|restaking_vault| {
                        match &restaking_vault
                            .receipt_token_pricing_source
                            .try_deserialize()?
                        {
                            Some(TokenPricingSource::JitoRestakingVault { .. }) => Ok(1),
                            _ => err!(
                                errors::ErrorCode::FundOperationCommandExecutionFailedException
                            )?,
                        }
                    })
                    .collect::<Result<Vec<_>>>()?
                    .iter()
                    .sum();
                if remaining_accounts.len()
                    < num_queued_batches
                        + num_supported_token_pricing_sources
                        + num_restaking_vault_pricing_sources
                {
                    err!(ErrorCode::AccountNotEnoughKeys)?;
                }

                let (uninitialized_withdrawal_batch_accounts, remaining_accounts) =
                    remaining_accounts.split_at(num_queued_batches);

                let (supported_token_pricing_sources, remaining_accounts) =
                    remaining_accounts.split_at(num_supported_token_pricing_sources);

                let (restaking_vault_pricing_sources, pricing_sources) =
                    remaining_accounts.split_at(num_restaking_vault_pricing_sources);

                // calculate LST max cycle fee
                let mut lst_max_cycle_fee_numerator = 0u64;
                let mut lst_max_cycle_fee_denominator = 0u64;
                for (i, supported_token) in fund_account.get_supported_tokens_iter().enumerate() {
                    let (numerator, denominator) = match &supported_token
                        .pricing_source
                        .try_deserialize()?
                    {
                        Some(TokenPricingSource::MarinadeStakePool { address }) => {
                            let account = supported_token_pricing_sources[i];
                            require_keys_eq!(account.key(), *address);
                            MarinadeStakePoolService::get_max_cycle_fee(account)?
                        }
                        Some(TokenPricingSource::SPLStakePool { address }) => {
                            let account = supported_token_pricing_sources[i];
                            require_keys_eq!(account.key(), *address);
                            SPLStakePoolService::get_max_cycle_fee(account)?
                        }
                        _ => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
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
                for (i, restaking_vault) in fund_account.get_restaking_vaults_iter().enumerate() {
                    let (numerator, denominator) = match &restaking_vault
                        .receipt_token_pricing_source
                        .try_deserialize()?
                    {
                        Some(TokenPricingSource::JitoRestakingVault { address }) => {
                            let account = restaking_vault_pricing_sources[i];
                            require_keys_eq!(account.key(), *address);
                            JitoRestakingVaultService::get_max_cycle_fee(account)?
                        }
                        _ => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
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
                let lrt_max_cycle_fee_rate = 1.0
                    - (1.0
                        - (lst_max_cycle_fee_numerator as f32
                            / lst_max_cycle_fee_denominator.max(1) as f32))
                        * (1.0
                            - (vrt_max_cycle_fee_numerator as f32
                                / vrt_max_cycle_fee_denominator.max(1) as f32));
                let withdrawal_fee_rate = fund_account.withdrawal_fee_rate_bps as f32 / 10_000.0;
                drop(fund_account);

                // adjust withdrawal fee rate
                if lrt_max_cycle_fee_rate > withdrawal_fee_rate {
                    let lrt_max_cycle_fee_rate_bps = (lrt_max_cycle_fee_rate * 10_000.0).ceil();
                    if lrt_max_cycle_fee_rate_bps > u16::MAX as f32 {
                        err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?;
                    }
                    ctx.fund_account
                        .load_mut()?
                        .set_withdrawal_fee_rate_bps(lrt_max_cycle_fee_rate_bps as u16)?;
                }

                // do process withdrawal
                let (
                    processed_receipt_token_amount,
                    reserved_asset_user_amount,
                    deducted_asset_fee_amount,
                    pricing_service,
                ) = {
                    let mut fund_service =
                        FundService::new(ctx.receipt_token_mint, ctx.fund_account)?;
                    let (
                        processed_receipt_token_amount,
                        reserved_asset_user_amount,
                        deducted_asset_fee_amount,
                    ) = fund_service.process_withdrawal_batches(
                        ctx.operator,
                        ctx.system_program,
                        receipt_token_program,
                        receipt_token_lock_account,
                        fund_reserve_account,
                        fund_treasury_account,
                        optional_supported_token_mint.to_option(),
                        optional_supported_token_program.to_option(),
                        optional_fund_supported_token_reserve_account.to_option(),
                        optional_fund_supported_token_treasury_account.to_option(),
                        uninitialized_withdrawal_batch_accounts,
                        pricing_sources,
                        self.forced,
                        requested_receipt_token_amount,
                        0,
                    )?;

                    fund_service.harvest_from_treasury_account(
                        ctx.operator,
                        ctx.system_program,
                        fund_treasury_account,
                        program_revenue_account,
                        optional_associated_token_account_program.to_option(),
                        optional_supported_token_mint.to_option(),
                        optional_supported_token_program.to_option(),
                        optional_fund_supported_token_treasury_account.to_option(),
                        program_supported_token_revenue_account.to_option(),
                    )?;

                    let pricing_service =
                        fund_service.new_pricing_service(pricing_sources.into_iter().cloned())?;

                    (
                        processed_receipt_token_amount,
                        reserved_asset_user_amount,
                        deducted_asset_fee_amount,
                        pricing_service,
                    )
                };

                if processed_receipt_token_amount > 0 {
                    // adjust accumulated deposit capacity configuration as much as the asset value withdrawn
                    // the policy is: allocate half to SOL cap if it is currently greater than zero, then allocate rest to LST caps based on their weighted allocation strategy
                    let receipt_token_amount_processed_as_sol = pricing_service
                        .get_token_amount_as_sol(
                            &ctx.receipt_token_mint.key(),
                            processed_receipt_token_amount,
                        )?;

                    let mut fund_account = ctx.fund_account.load_mut()?;
                    let mut strategy =
                        WeightedAllocationStrategy::<FUND_ACCOUNT_MAX_SUPPORTED_TOKENS>::new(
                            fund_account
                                .get_supported_tokens_iter()
                                .map(|supported_token| {
                                    Ok(WeightedAllocationParticipant::new(
                                        supported_token.sol_allocation_weight,
                                        pricing_service.get_token_amount_as_sol(
                                            &supported_token.mint,
                                            supported_token.token.operation_reserved_amount,
                                        )?,
                                        supported_token.sol_allocation_capacity_amount,
                                    ))
                                })
                                .collect::<Result<Vec<_>>>()?,
                        );

                    let mut supported_token_increasing_capacity_as_sol =
                        if fund_account.sol.accumulated_deposit_capacity_amount > 0 {
                            receipt_token_amount_processed_as_sol.div_ceil(2)
                        } else {
                            receipt_token_amount_processed_as_sol
                        };
                    supported_token_increasing_capacity_as_sol -=
                        strategy.put(receipt_token_amount_processed_as_sol)?;

                    for (i, supported_token) in
                        fund_account.get_supported_tokens_iter_mut().enumerate()
                    {
                        supported_token
                            .token
                            .set_accumulated_deposit_capacity_amount(
                                supported_token
                                    .token
                                    .accumulated_deposit_capacity_amount
                                    .saturating_add(pricing_service.get_sol_amount_as_token(
                                        &supported_token.mint,
                                        strategy.get_participant_last_put_amount_by_index(i)?,
                                    )?),
                            )?;
                    }

                    if fund_account.sol.accumulated_deposit_capacity_amount > 0 {
                        let sol_increasing_capacity = receipt_token_amount_processed_as_sol
                            - supported_token_increasing_capacity_as_sol;
                        fund_account.sol.accumulated_deposit_capacity_amount = fund_account
                            .sol
                            .accumulated_deposit_capacity_amount
                            .saturating_add(sol_increasing_capacity);
                    }
                }

                return Ok((
                    Some(
                        ProcessWithdrawalBatchCommandResult {
                            supported_token_mint: supported_token_mint_key,
                            requested_receipt_token_amount,
                            processed_receipt_token_amount,
                            reserved_asset_user_amount,
                            deducted_asset_fee_amount,
                        }
                        .into(),
                    ),
                    // TODO: ProcessWithdrawalBatchCommand.execute
                    None,
                ));
            }
        }
    }
}
