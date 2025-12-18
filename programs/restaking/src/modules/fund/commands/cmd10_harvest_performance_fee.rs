use anchor_lang::prelude::*;
use anchor_spl::associated_token;
use spl_stake_pool::solana_program::native_token::LAMPORTS_PER_SOL;

use crate::constants::PROGRAM_REVENUE_ADDRESS;
use crate::errors::ErrorCode;
use crate::modules::pricing::TokenPricingSource;
use crate::utils::{AccountInfoExt, PDASeeds};

use super::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct HarvestPerformanceFeeCommand {
    state: HarvestPerformanceFeeState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub enum HarvestPerformanceFeeState {
    #[default]
    New,
    Execute,
    Unused {
        unused: bool,
    },
}

use HarvestPerformanceFeeState::*;

const RESTAKING_MINIMUM_PERFORMANCE_FEE_LAMPORTS: u64 = 1_000_000_000;

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct HarvestPerformanceFeeCommandResult {
    pub receipt_token_mint: Pubkey,
    pub receipt_token_minted_amount: u64,
    pub receipt_token_price: u64,
}

impl SelfExecutable for HarvestPerformanceFeeCommand {
    fn execute<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> ExecutionResult {
        let (result, entry) = match &self.state {
            New => self.execute_new(ctx, accounts)?,
            Execute => self.execute_execute(ctx, accounts)?,
            Unused { .. } => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
        };

        Ok((
            result,
            entry.or_else(|| {
                Some(HarvestRestakingYieldCommand::default().without_required_accounts())
            }),
        ))
    }
}

impl HarvestPerformanceFeeCommand {
    fn execute_new<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> ExecutionResult {
        if !&self.is_performance_fee_harvestable(ctx, accounts)? {
            return Ok((None, None));
        }

        // * (0) receipt token program
        // * (1) program revenue account
        // * (2) program receipt token revenue account
        // * (3) associated token program
        // * (4) system program
        let required_accounts = [
            (anchor_spl::token_2022::ID, false),
            (PROGRAM_REVENUE_ADDRESS, false),
            (
                associated_token::get_associated_token_address_with_program_id(
                    &PROGRAM_REVENUE_ADDRESS,
                    &ctx.receipt_token_mint.key(),
                    &anchor_spl::token_2022::ID,
                ),
                true,
            ),
            (anchor_spl::associated_token::ID, false),
            (system_program::ID, false),
        ]
        .into_iter();

        let command = Self { state: Execute };
        let entry = command.with_required_accounts(required_accounts);

        Ok((None, Some(entry)))
    }

    fn execute_execute<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        mut accounts: &[&'info AccountInfo<'info>],
    ) -> ExecutionResult {
        if !&self.is_performance_fee_harvestable(ctx, accounts)? {
            return Ok((None, None));
        }

        let [receipt_token_program, program_revenue_account, program_receipt_token_revenue_account, associated_token_program, system_program, remaining_accounts @ ..] =
            accounts
        else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };
        accounts = remaining_accounts;

        // validation
        require_keys_eq!(receipt_token_program.key(), anchor_spl::token_2022::ID);
        require_keys_eq!(program_revenue_account.key(), PROGRAM_REVENUE_ADDRESS);
        require_keys_eq!(
            program_receipt_token_revenue_account.key(),
            associated_token::get_associated_token_address_with_program_id(
                &PROGRAM_REVENUE_ADDRESS,
                &ctx.receipt_token_mint.key(),
                &anchor_spl::token_2022::ID,
            )
        );

        let fund_account = ctx.fund_account.load()?;

        let one_receipt_token_as_sol = fund_account.one_receipt_token_as_sol;
        let fee_harvested_one_receipt_token_as_sol =
            fund_account.fee_harvested_one_receipt_token_as_sol;

        let performance_gain_in_sol_amount = crate::utils::get_proportional_amount_u64(
            ctx.receipt_token_mint.supply,
            one_receipt_token_as_sol - fee_harvested_one_receipt_token_as_sol,
            LAMPORTS_PER_SOL,
        )?;

        let performance_fee_in_sol_amount = crate::utils::get_proportional_amount_u64(
            performance_gain_in_sol_amount,
            fund_account.performance_fee_rate_bps as u64,
            10_000,
        )?;

        if performance_fee_in_sol_amount < RESTAKING_MINIMUM_PERFORMANCE_FEE_LAMPORTS {
            return Ok((None, None));
        }

        drop(fund_account);

        let mut pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
            .new_pricing_service(accounts.iter().copied(), false)?;

        let performance_fee_in_receipt_token_amount = pricing_service.get_sol_amount_as_token(
            &ctx.receipt_token_mint.key(),
            performance_fee_in_sol_amount,
        )?;

        let result = if performance_fee_in_receipt_token_amount > 0 {
            // update high-water mark
            let mut fund_account = ctx.fund_account.load_mut()?;
            fund_account.fee_harvested_one_receipt_token_as_sol = one_receipt_token_as_sol;
            fund_account.performance_fee_last_harvested_at = Clock::get()?.unix_timestamp;
            drop(fund_account);

            // create receipt token revenue account if not initialized
            if !program_receipt_token_revenue_account.is_initialized() {
                anchor_spl::associated_token::create(CpiContext::new(
                    associated_token_program.to_account_info(),
                    anchor_spl::associated_token::Create {
                        payer: ctx.operator.to_account_info(),
                        associated_token: program_receipt_token_revenue_account.to_account_info(),
                        authority: program_revenue_account.to_account_info(),
                        mint: ctx.receipt_token_mint.to_account_info(),
                        system_program: system_program.to_account_info(),
                        token_program: receipt_token_program.to_account_info(),
                    },
                ))?;
            }

            // mint receipt token to revenue account
            anchor_spl::token_2022::mint_to(
                CpiContext::new_with_signer(
                    receipt_token_program.to_account_info(),
                    anchor_spl::token_2022::MintTo {
                        mint: ctx.receipt_token_mint.to_account_info(),
                        to: program_receipt_token_revenue_account.to_account_info(),
                        authority: ctx.fund_account.to_account_info(),
                    },
                    &[ctx.fund_account.load()?.get_seeds().as_ref()],
                ),
                performance_fee_in_receipt_token_amount,
            )?;

            let mut fund_account = ctx.fund_account.load_mut()?;
            fund_account.reload_receipt_token_supply(ctx.receipt_token_mint)?;
            drop(fund_account);

            // get updated receipt token price
            FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                .update_asset_values(&mut pricing_service, true)?;
            let fund_account = ctx.fund_account.load()?;
            let one_receipt_token_as_sol = fund_account.one_receipt_token_as_sol;

            Some(
                HarvestPerformanceFeeCommandResult {
                    receipt_token_mint: ctx.receipt_token_mint.key(),
                    receipt_token_minted_amount: performance_fee_in_receipt_token_amount,
                    receipt_token_price: one_receipt_token_as_sol,
                }
                .into(),
            )
        } else {
            None
        };

        Ok((result, None))
    }

    fn is_performance_fee_harvestable<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<bool> {
        let fund_account = ctx.fund_account.load()?;

        let is_sol_depositable = fund_account.sol.depositable == 1;
        let mut is_all_supported_tokens_stakable = true;

        for supported_token in fund_account.get_supported_tokens_iter() {
            match supported_token.pricing_source.try_deserialize()? {
                // stakable tokens
                Some(TokenPricingSource::SPLStakePool { .. })
                | Some(TokenPricingSource::MarinadeStakePool { .. })
                | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
                | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. }) => {}

                // not stakable tokens
                Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                | Some(TokenPricingSource::PeggedToken { .. }) => {
                    is_all_supported_tokens_stakable = false;
                    break;
                }

                // invalid configuration
                Some(TokenPricingSource::JitoRestakingVault { .. })
                | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                | Some(TokenPricingSource::SolvBTCVault { .. })
                | Some(TokenPricingSource::VirtualVault { .. })
                | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
                #[cfg(all(test, not(feature = "idl-build")))]
                Some(TokenPricingSource::Mock { .. }) => {
                    err!(ErrorCode::FundOperationCommandExecutionFailedException)?
                }
            }
        }

        if !(is_sol_depositable || is_all_supported_tokens_stakable) {
            return Ok(false);
        }

        drop(fund_account);

        FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
            .new_pricing_service(accounts.iter().copied(), true)?;

        let mut fund_account = ctx.fund_account.load_mut()?;
        let one_receipt_token_as_sol = fund_account.one_receipt_token_as_sol;
        if fund_account.performance_fee_rate_bps == 0 {
            fund_account.fee_harvested_one_receipt_token_as_sol = one_receipt_token_as_sol;
        }

        if fund_account.fee_harvested_one_receipt_token_as_sol
            >= fund_account.one_receipt_token_as_sol
        {
            return Ok(false);
        }

        Ok(true)
    }
}
