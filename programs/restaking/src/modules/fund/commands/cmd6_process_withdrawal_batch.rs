use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account;

use crate::constants::PROGRAM_REVENUE_ADDRESS;
use crate::errors;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::{JitoRestakingVaultService, SolvBTCVaultService};
use crate::modules::staking::{
    MarinadeStakePoolService, SPLStakePoolService, SanctumMultiValidatorSPLStakePoolService,
    SanctumSingleValidatorSPLStakePoolService,
};
use crate::utils::AccountInfoExt;

use super::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct ProcessWithdrawalBatchCommand {
    state: ProcessWithdrawalBatchCommandState,
    forced: bool,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum ProcessWithdrawalBatchCommandState {
    /// Initializes a prepare command for an eligible asset.
    #[default]
    New,
    /// Prepares to execute withdrawal for a specific asset.
    Prepare { asset_token_mint: Option<Pubkey> },
    /// Executes withdrawal for a specific asset and transitions to the next command,
    /// either preparing the next eligible asset or performing a staking operation.
    Execute {
        asset_token_mint: Option<Pubkey>,
        num_processing_batches: u8,
        receipt_token_amount: u64,
    },
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ProcessWithdrawalBatchCommandResult {
    pub requested_receipt_token_amount: u64,
    pub processed_receipt_token_amount: u64,
    pub processed_batch_accounts: Vec<Pubkey>,
    pub asset_token_mint: Option<Pubkey>,
    pub required_asset_amount: u64,
    pub reserved_asset_user_amount: u64,
    pub deducted_asset_fee_amount: u64,
    pub offsetted_asset_receivables: Vec<ProcessWithdrawalBatchCommandResultAssetReceivable>,
    pub transferred_asset_revenue_amount: u64,
    pub withdrawal_fee_rate_bps: u16,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ProcessWithdrawalBatchCommandResultAssetReceivable {
    pub asset_token_mint: Option<Pubkey>,
    pub asset_amount: u64,
}

impl SelfExecutable for ProcessWithdrawalBatchCommand {
    fn execute<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let mut result: Option<OperationCommandResult> = None;

        match self.state {
            ProcessWithdrawalBatchCommandState::New => {}
            ProcessWithdrawalBatchCommandState::Prepare { asset_token_mint } => {
                let (num_processing_batches, mut required_accounts) =
                    FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                        .find_accounts_to_process_withdrawal_batches(
                            asset_token_mint,
                            self.forced,
                        )?;
                let fund_account = ctx.fund_account.load()?;
                let requested_receipt_token_amount = fund_account
                    .get_asset_receipt_token_withdrawal_obligated_amount(asset_token_mint)?;

                // to harvest program revenue (prepended)
                required_accounts.insert(0, (PROGRAM_REVENUE_ADDRESS, true));
                required_accounts.insert(
                    1,
                    asset_token_mint
                        .map(|mint| {
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
                    asset_token_mint
                        .map(|_| (spl_associated_token_account::ID, false))
                        .unwrap_or_else(|| (Pubkey::default(), false)),
                );

                // to calculate LST cycle fee (appended)
                for supported_token in fund_account.get_supported_tokens_iter() {
                    match &supported_token.pricing_source.try_deserialize()? {
                        // LST supported tokens
                        Some(TokenPricingSource::MarinadeStakePool { address })
                        | Some(TokenPricingSource::SPLStakePool { address })
                        | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool {
                            address,
                        })
                        | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { address }) =>
                        {
                            required_accounts.push((*address, false));
                        }
                        // Non-LST supported tokens (no-op)
                        Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                        | Some(TokenPricingSource::PeggedToken { .. }) => {}
                        // otherwise fails
                        Some(TokenPricingSource::JitoRestakingVault { .. })
                        | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                        | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                        | Some(TokenPricingSource::SolvBTCVault { .. })
                        | Some(TokenPricingSource::VirtualVault { .. })
                        | None => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                        #[cfg(all(test, not(feature = "idl-build")))]
                        Some(TokenPricingSource::Mock { .. }) => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                    }
                }

                // to calculate VRT cycle fee (appended)
                for restaking_vault in fund_account.get_restaking_vaults_iter() {
                    match &restaking_vault
                        .receipt_token_pricing_source
                        .try_deserialize()?
                    {
                        // restaking vaults
                        Some(TokenPricingSource::JitoRestakingVault { address })
                        | Some(TokenPricingSource::SolvBTCVault { address }) => {
                            required_accounts.push((*address, false));
                        }
                        // virtual vault (no-op)
                        Some(TokenPricingSource::VirtualVault { .. }) => {}
                        // otherwise fails
                        Some(TokenPricingSource::SPLStakePool { .. })
                        | Some(TokenPricingSource::MarinadeStakePool { .. })
                        | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
                        | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. })
                        | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                        | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                        | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                        | Some(TokenPricingSource::PeggedToken { .. })
                        | None => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                        #[cfg(all(test, not(feature = "idl-build")))]
                        Some(TokenPricingSource::Mock { .. }) => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                    }
                }

                return Ok((
                    None,
                    Some(
                        ProcessWithdrawalBatchCommand {
                            state: ProcessWithdrawalBatchCommandState::Execute {
                                asset_token_mint,
                                num_processing_batches,
                                receipt_token_amount: requested_receipt_token_amount,
                            },
                            forced: self.forced,
                        }
                        .with_required_accounts(required_accounts),
                    ),
                ));
            }
            ProcessWithdrawalBatchCommandState::Execute {
                asset_token_mint,
                num_processing_batches,
                receipt_token_amount: requested_receipt_token_amount,
            } => {
                let [program_revenue_account, program_supported_token_revenue_account, optional_associated_token_account_program, receipt_token_program, receipt_token_lock_account, fund_reserve_account, fund_treasury_account, optional_supported_token_mint, optional_supported_token_program, optional_fund_supported_token_reserve_account, optional_fund_supported_token_treasury_account, remaining_accounts @ ..] =
                    accounts
                else {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                };

                let fund_account = ctx.fund_account.load()?;
                let num_supported_token_pricing_sources = fund_account
                    .get_supported_tokens_iter()
                    .try_fold(0usize, |count, supported_token| {
                        match &supported_token.pricing_source.try_deserialize()? {
                            // LST supported tokens
                            Some(TokenPricingSource::MarinadeStakePool { .. })
                            | Some(TokenPricingSource::SPLStakePool { .. })
                            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool {
                                ..
                            })
                            | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool {
                                ..
                            }) => Ok(count + 1),
                            // Non-LST supported tokens (no-op)
                            Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                            | Some(TokenPricingSource::PeggedToken { .. }) => Ok(count),
                            // otherwise fails
                            Some(TokenPricingSource::JitoRestakingVault { .. })
                            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                            | Some(TokenPricingSource::SolvBTCVault { .. })
                            | Some(TokenPricingSource::VirtualVault { .. })
                            | None => err!(
                                errors::ErrorCode::FundOperationCommandExecutionFailedException
                            ),
                            #[cfg(all(test, not(feature = "idl-build")))]
                            Some(TokenPricingSource::Mock { .. }) => err!(
                                errors::ErrorCode::FundOperationCommandExecutionFailedException
                            ),
                        }
                    })?;
                let num_restaking_vault_pricing_sources = fund_account
                    .get_restaking_vaults_iter()
                    .try_fold(0usize, |count, restaking_vault| {
                        match &restaking_vault
                            .receipt_token_pricing_source
                            .try_deserialize()?
                        {
                            // restaking vaults
                            Some(TokenPricingSource::JitoRestakingVault { .. })
                            | Some(TokenPricingSource::SolvBTCVault { .. }) => Ok(count + 1),
                            // virtual vault (no-op)
                            Some(TokenPricingSource::VirtualVault { .. }) => Ok(count),
                            // otherwise fails
                            Some(TokenPricingSource::SPLStakePool { .. })
                            | Some(TokenPricingSource::MarinadeStakePool { .. })
                            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool {
                                ..
                            })
                            | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool {
                                ..
                            })
                            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                            | Some(TokenPricingSource::PeggedToken { .. })
                            | None => err!(
                                errors::ErrorCode::FundOperationCommandExecutionFailedException
                            ),
                            #[cfg(all(test, not(feature = "idl-build")))]
                            Some(TokenPricingSource::Mock { .. }) => err!(
                                errors::ErrorCode::FundOperationCommandExecutionFailedException
                            ),
                        }
                    })?;
                if remaining_accounts.len()
                    < num_processing_batches as usize
                        + num_supported_token_pricing_sources
                        + num_restaking_vault_pricing_sources
                {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?;
                }

                let (uninitialized_withdrawal_batch_accounts, remaining_accounts) =
                    remaining_accounts.split_at(num_processing_batches as usize);

                let (supported_token_pricing_sources, remaining_accounts) =
                    remaining_accounts.split_at(num_supported_token_pricing_sources);

                let (restaking_vault_pricing_sources, pricing_sources) =
                    remaining_accounts.split_at(num_restaking_vault_pricing_sources);

                // calculate LST max cycle fee
                let mut i = 0;
                let mut lst_max_cycle_fee_numerator = 0u64;
                let mut lst_max_cycle_fee_denominator = 0u64;
                for supported_token in fund_account.get_supported_tokens_iter() {
                    let (numerator, denominator) = match &supported_token
                        .pricing_source
                        .try_deserialize()?
                    {
                        // LST supported tokens
                        Some(TokenPricingSource::MarinadeStakePool { address }) => {
                            let account = supported_token_pricing_sources[i];
                            i += 1;
                            require_keys_eq!(account.key(), *address);
                            MarinadeStakePoolService::get_max_cycle_fee(account)?
                        }
                        Some(TokenPricingSource::SPLStakePool { address }) => {
                            let account = supported_token_pricing_sources[i];
                            i += 1;
                            require_keys_eq!(account.key(), *address);
                            <SPLStakePoolService>::get_max_cycle_fee(account)?
                        }
                        Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool {
                            address,
                        }) => {
                            let account = supported_token_pricing_sources[i];
                            i += 1;
                            require_keys_eq!(account.key(), *address);
                            SanctumSingleValidatorSPLStakePoolService::get_max_cycle_fee(account)?
                        }
                        Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { address }) => {
                            let account = supported_token_pricing_sources[i];
                            i += 1;
                            require_keys_eq!(account.key(), *address);
                            SanctumMultiValidatorSPLStakePoolService::get_max_cycle_fee(account)?
                        }
                        // Non-LST supported tokens
                        Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                        | Some(TokenPricingSource::PeggedToken { .. }) => (0, 0),
                        // otherwise fails
                        Some(TokenPricingSource::JitoRestakingVault { .. })
                        | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                        | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                        | Some(TokenPricingSource::SolvBTCVault { .. })
                        | Some(TokenPricingSource::VirtualVault { .. })
                        | None => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                        #[cfg(all(test, not(feature = "idl-build")))]
                        Some(TokenPricingSource::Mock { .. }) => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                    };

                    // numerator/denominator > lst_max_cycle_fee_numerator/lst_max_cycle_fee_denominator
                    if denominator != 0
                        && numerator * lst_max_cycle_fee_denominator
                            > lst_max_cycle_fee_numerator * denominator
                    {
                        lst_max_cycle_fee_numerator = numerator;
                        lst_max_cycle_fee_denominator = denominator;
                    }
                }
                require_eq!(i, num_supported_token_pricing_sources);

                // calculate VRT max cycle fee
                let mut vrt_max_cycle_fee_numerator = 0u64;
                let mut vrt_max_cycle_fee_denominator = 0u64;
                let mut i = 0;
                for restaking_vault in fund_account.get_restaking_vaults_iter() {
                    let (numerator, denominator) = match &restaking_vault
                        .receipt_token_pricing_source
                        .try_deserialize()?
                    {
                        // restaking vaults
                        Some(TokenPricingSource::JitoRestakingVault { address }) => {
                            let account = restaking_vault_pricing_sources[i];
                            i += 1;
                            require_keys_eq!(account.key(), *address);
                            JitoRestakingVaultService::get_max_cycle_fee(account)?
                        }
                        Some(TokenPricingSource::SolvBTCVault { address }) => {
                            let account = restaking_vault_pricing_sources[i];
                            i += 1;
                            require_keys_eq!(account.key(), *address);
                            SolvBTCVaultService::get_max_cycle_fee(account)?
                        }
                        // virtual vault (no-op)
                        Some(TokenPricingSource::VirtualVault { .. }) => (0, 0),
                        // otherwise fails
                        Some(TokenPricingSource::SPLStakePool { .. })
                        | Some(TokenPricingSource::MarinadeStakePool { .. })
                        | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
                        | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. })
                        | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                        | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                        | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                        | Some(TokenPricingSource::PeggedToken { .. })
                        | None => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                        #[cfg(all(test, not(feature = "idl-build")))]
                        Some(TokenPricingSource::Mock { .. }) => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                    };
                    // numerator/denominator > vrt_max_cycle_fee_numerator/vrt_max_cycle_fee_denominator
                    if denominator != 0
                        && numerator * vrt_max_cycle_fee_denominator
                            > vrt_max_cycle_fee_numerator * denominator
                    {
                        vrt_max_cycle_fee_numerator = numerator;
                        vrt_max_cycle_fee_denominator = denominator;
                    }
                }
                require_eq!(i, num_restaking_vault_pricing_sources);

                // calculate LRT max cycle fee to ensure withdrawal fee is equal or greater than max fee expense during cash-in/out
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
                    processed_batch_accounts,
                    required_asset_amount,
                    reserved_asset_user_amount,
                    deducted_asset_fee_amount,
                    offsetted_asset_receivables,
                    transferred_asset_revenue_amount,
                ) = {
                    let mut fund_service =
                        FundService::new(ctx.receipt_token_mint, ctx.fund_account)?;

                    let mut pricing_service =
                        fund_service.new_pricing_service(pricing_sources.iter().copied(), false)?;

                    let (
                        processed_receipt_token_amount,
                        processed_batch_accounts,
                        required_asset_amount,
                        reserved_asset_user_amount,
                        deducted_asset_fee_amount,
                        offsetted_asset_receivables,
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
                        self.forced,
                        requested_receipt_token_amount,
                        0,
                        &pricing_service,
                    )?;

                    // harvest revenue from temporary treasury only if current pending batch processed
                    let transferred_asset_revenue_amount = if processed_receipt_token_amount > 0 {
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
                        )?
                    } else {
                        0
                    };

                    fund_service.update_asset_values(&mut pricing_service, true)?;
                    (
                        processed_receipt_token_amount,
                        processed_batch_accounts,
                        required_asset_amount,
                        reserved_asset_user_amount,
                        deducted_asset_fee_amount,
                        offsetted_asset_receivables,
                        transferred_asset_revenue_amount,
                    )
                };

                result = Some(
                    ProcessWithdrawalBatchCommandResult {
                        asset_token_mint,
                        requested_receipt_token_amount,
                        processed_receipt_token_amount,
                        processed_batch_accounts,
                        required_asset_amount,
                        reserved_asset_user_amount,
                        deducted_asset_fee_amount,
                        offsetted_asset_receivables: offsetted_asset_receivables
                            .into_iter()
                            .map(|(asset_token_mint, asset_amount)| {
                                ProcessWithdrawalBatchCommandResultAssetReceivable {
                                    asset_token_mint,
                                    asset_amount,
                                }
                            })
                            .collect::<Vec<_>>(),
                        transferred_asset_revenue_amount,
                        withdrawal_fee_rate_bps: ctx.fund_account.load()?.withdrawal_fee_rate_bps,
                    }
                    .into(),
                );
            }
        }

        // transition to next command
        Ok({
            let current_asset_token_mint = match self.state {
                ProcessWithdrawalBatchCommandState::New => None,
                ProcessWithdrawalBatchCommandState::Prepare { asset_token_mint }
                | ProcessWithdrawalBatchCommandState::Execute {
                    asset_token_mint, ..
                } => Some(asset_token_mint),
            };

            let next_asset_token_mint = {
                let fund_account = ctx.fund_account.load()?;
                let mut assets = fund_account.get_asset_states_iter();

                match current_asset_token_mint {
                    None => assets
                        .find(|asset| asset.get_receipt_token_withdrawal_obligated_amount() > 0)
                        .map(|asset| asset.get_token_mint_and_program().map(|(mint, _)| mint)),
                    Some(current) => assets
                        .skip_while(|asset| {
                            asset.get_token_mint_and_program().map(|(mint, _)| mint) != current
                        })
                        .skip(1)
                        .find(|asset| asset.get_receipt_token_withdrawal_obligated_amount() > 0)
                        .map(|asset| asset.get_token_mint_and_program().map(|(mint, _)| mint)),
                }
            };

            match next_asset_token_mint {
                Some(asset_token_mint) => {
                    // directly execute prepare commands
                    (
                        result,
                        ProcessWithdrawalBatchCommand {
                            state: ProcessWithdrawalBatchCommandState::Prepare { asset_token_mint },
                            forced: self.forced,
                        }
                        .execute(ctx, accounts)?
                        .1,
                    )
                }
                None => (
                    result,
                    Some(UnstakeLSTCommand::default().without_required_accounts()),
                ),
            }
        })
    }
}
