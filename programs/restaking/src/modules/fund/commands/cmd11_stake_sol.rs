use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::staking::*;

use super::{
    FundService, NormalizeSTCommand, OperationCommandContext, OperationCommandEntry,
    OperationCommandResult, SelfExecutable, WeightedAllocationParticipant,
    WeightedAllocationStrategy, FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct StakeSOLCommand {
    state: StakeSOLCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Copy)]
pub struct StakeSOLCommandItem {
    token_mint: Pubkey,
    allocated_sol_amount: u64,
}

impl std::fmt::Debug for StakeSOLCommandItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.token_mint, self.allocated_sol_amount)
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Default)]
pub enum StakeSOLCommandState {
    /// Initializes a command with items based on the fund state and strategy.
    #[default]
    New,
    /// Prepares to execute staking for the first item in the list.
    Prepare {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        items: Vec<StakeSOLCommandItem>,
    },
    /// Executes staking for the first item and transitions to the next command,
    /// either preparing the next item or performing a normalization operation.
    Execute {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        items: Vec<StakeSOLCommandItem>,
    },
}

impl std::fmt::Debug for StakeSOLCommandState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => f.write_str("New"),
            Self::Prepare { items } => {
                if items.is_empty() {
                    f.write_str("Prepare")
                } else {
                    f.debug_struct("Prepare").field("item", &items[0]).finish()
                }
            }
            Self::Execute { items } => {
                if items.is_empty() {
                    f.write_str("Execute")
                } else {
                    f.debug_struct("Execute").field("item", &items[0]).finish()
                }
            }
        }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize)]
pub struct StakeSOLCommandResult {
    pub token_mint: Pubkey,
    pub staked_sol_amount: u64,
    pub deducted_sol_fee_amount: u64,
    pub minted_token_amount: u64,
    pub operation_reserved_sol_amount: u64,
    pub operation_receivable_sol_amount: u64,
    pub operation_reserved_token_amount: u64,
}

impl SelfExecutable for StakeSOLCommand {
    fn execute<'a, 'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let (result, entry) = match &self.state {
            StakeSOLCommandState::New => self.execute_new(ctx, accounts)?,
            StakeSOLCommandState::Prepare { items } => {
                self.execute_prepare(ctx, accounts, items.clone(), None)?
            }
            StakeSOLCommandState::Execute { items } => {
                self.execute_execute(ctx, accounts, items)?
            }
        };

        Ok((
            result,
            entry.or_else(|| Some(NormalizeSTCommand::default().without_required_accounts())),
        ))
    }
}

// These are implementations of each command state.
#[deny(clippy::wildcard_enum_match_arm)]
impl StakeSOLCommand {
    /// An initial state of `StakeSOL` command.
    /// In this state, operator iterates the fund and
    /// decides where and how much to stake each.
    #[inline(never)]
    fn execute_new<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
            .new_pricing_service(accounts.iter().copied())?;
        let fund_account = ctx.fund_account.load()?;

        let sol_net_operation_reserved_amount =
            fund_account.get_asset_net_operation_reserved_amount(None, true, &pricing_service)?;

        // does not have enough reserved SOL amount to operate
        if sol_net_operation_reserved_amount <= 0 {
            return Ok((None, None));
        }
        let sol_staking_reserved_amount = u64::try_from(sol_net_operation_reserved_amount)?;

        let mut strategy = WeightedAllocationStrategy::<FUND_ACCOUNT_MAX_SUPPORTED_TOKENS>::new(
            fund_account
                .get_supported_tokens_iter()
                .map(|supported_token| {
                    Ok(match supported_token.pricing_source.try_deserialize()? {
                        // stakable tokens
                        Some(TokenPricingSource::SPLStakePool { .. })
                        | Some(TokenPricingSource::MarinadeStakePool { .. })
                        | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. }) => {
                            Some(WeightedAllocationParticipant::new(
                                supported_token.sol_allocation_weight,
                                fund_account.get_asset_total_amount_as_sol(
                                    Some(supported_token.mint),
                                    &pricing_service,
                                )?,
                                supported_token.sol_allocation_capacity_amount,
                            ))
                        }

                        // not stakable tokens
                        Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                        | Some(TokenPricingSource::PeggedToken { .. }) => None,

                        // invalid configuration
                        Some(TokenPricingSource::JitoRestakingVault { .. })
                        | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                        | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                        | Some(TokenPricingSource::SolvBTCVault { .. })
                        | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
                        #[cfg(all(test, not(feature = "idl-build")))]
                        Some(TokenPricingSource::Mock { .. }) => {
                            err!(ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                    })
                })
                .collect::<Result<Vec<Option<_>>>>()?
                .into_iter()
                .flatten(),
        );
        strategy.put(sol_staking_reserved_amount)?;

        let mut items =
            Vec::<StakeSOLCommandItem>::with_capacity(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS);
        for (i, supported_token) in fund_account.get_supported_tokens_iter().enumerate() {
            let allocated_sol_amount = strategy.get_participant_last_put_amount_by_index(i)?;
            if allocated_sol_amount >= 1_000_000_000 {
                items.push(StakeSOLCommandItem {
                    token_mint: supported_token.mint,
                    allocated_sol_amount,
                });
            }
        }

        // prepare state does not require additional accounts,
        // so we can execute directly.
        drop(fund_account);
        self.execute_prepare(ctx, accounts, items, None)
    }

    #[inline(never)]
    fn execute_prepare<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: Vec<StakeSOLCommandItem>,
        previous_execution_result: Option<OperationCommandResult>,
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if items.is_empty() {
            return Ok((previous_execution_result, None));
        }
        let item = &items[0];

        let fund_account = ctx.fund_account.load()?;
        let token_pricing_source = fund_account
            .get_supported_token(&item.token_mint)?
            .pricing_source
            .try_deserialize()?;

        let pool_account = match token_pricing_source {
            Some(TokenPricingSource::SPLStakePool { address })
            | Some(TokenPricingSource::MarinadeStakePool { address })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address }) => *accounts
                .iter()
                .find(|account| account.key() == address)
                .ok_or_else(|| error!(ErrorCode::FundOperationCommandExecutionFailedException))?,
            // fail when supported token is not stakable
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
            | Some(TokenPricingSource::SolvBTCVault { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        };

        let fund_reserve_account = fund_account.get_reserve_account_address()?;
        let fund_supported_token_reserve_account =
            fund_account.find_supported_token_reserve_account_address(&item.token_mint)?;

        let required_accounts = [
            (fund_reserve_account, true),
            (fund_supported_token_reserve_account, true),
        ]
        .into_iter();

        let command = Self {
            state: StakeSOLCommandState::Execute { items },
        };

        let entry = match token_pricing_source {
            Some(TokenPricingSource::SPLStakePool { .. }) => {
                command.with_required_accounts(required_accounts.chain(
                    <SPLStakePoolService>::find_accounts_to_deposit_sol(pool_account)?,
                ))
            }
            Some(TokenPricingSource::MarinadeStakePool { .. }) => {
                command.with_required_accounts(required_accounts.chain(
                    MarinadeStakePoolService::find_accounts_to_deposit_sol(pool_account)?,
                ))
            }
            Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. }) => command
                .with_required_accounts(required_accounts.chain(
                    SanctumSingleValidatorSPLStakePoolService::find_accounts_to_deposit_sol(
                        pool_account,
                    )?,
                )),
            // fail when supported token is not stakable
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
            | Some(TokenPricingSource::SolvBTCVault { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        };

        Ok((previous_execution_result, Some(entry)))
    }

    #[inline(never)]
    fn execute_execute<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: &[StakeSOLCommandItem],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if items.is_empty() {
            return Ok((None, None));
        }
        let item = &items[0];

        let token_pricing_source = ctx
            .fund_account
            .load()?
            .get_supported_token(&item.token_mint)?
            .pricing_source
            .try_deserialize()?;

        let result = match token_pricing_source {
            Some(TokenPricingSource::SPLStakePool { address }) => Some(
                self.spl_stake_pool_deposit_sol::<SPLStakePool>(ctx, accounts, item, &address)?,
            ),
            Some(TokenPricingSource::MarinadeStakePool { address }) => {
                self.marinade_stake_pool_deposit_sol(ctx, accounts, item, &address)?
            }
            Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address }) => Some(
                self.spl_stake_pool_deposit_sol::<SanctumSingleValidatorSPLStakePool>(
                    ctx, accounts, item, &address,
                )?,
            ),
            // fail when supported token is not stakable
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
            | Some(TokenPricingSource::SolvBTCVault { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        }
        .map(
            |(to_pool_token_account_amount, minted_pool_token_amount, deducted_sol_fee_amount)| {
                // Update fund account
                let mut fund_account = ctx.fund_account.load_mut()?;
                fund_account.sol.operation_reserved_amount -= item.allocated_sol_amount;
                fund_account.sol.operation_receivable_amount += deducted_sol_fee_amount;

                let supported_token = fund_account.get_supported_token_mut(&item.token_mint)?;
                supported_token.token.operation_reserved_amount += minted_pool_token_amount;

                require_gte!(
                    to_pool_token_account_amount,
                    supported_token.token.get_total_reserved_amount(),
                );

                Ok(StakeSOLCommandResult {
                    token_mint: item.token_mint,
                    staked_sol_amount: item.allocated_sol_amount,
                    deducted_sol_fee_amount,
                    minted_token_amount: minted_pool_token_amount,
                    operation_reserved_token_amount: supported_token
                        .token
                        .operation_reserved_amount,
                    operation_reserved_sol_amount: fund_account.sol.operation_reserved_amount,
                    operation_receivable_sol_amount: fund_account.sol.operation_receivable_amount,
                }
                .into())
            },
        )
        .transpose()?;

        // prepare state does not require additional accounts,
        // so we can execute directly.
        self.execute_prepare(ctx, accounts, items[1..].to_vec(), result)
    }

    /// returns [to_pool_token_account_amount, minted_pool_token_amount, deducted_sol_fee_amount]
    fn spl_stake_pool_deposit_sol<'info, T: SPLStakePoolInterface>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        item: &StakeSOLCommandItem,
        pool_account_address: &Pubkey,
    ) -> Result<(u64, u64, u64)> {
        let [fund_reserve_account, fund_supported_token_reserve_account, pool_program, pool_account, pool_token_mint, pool_token_program, withdraw_authority, reserve_stake_account, manager_fee_account, validator_list_account, clock, pricing_sources @ ..] =
            accounts
        else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };
        require_keys_eq!(*pool_account_address, pool_account.key());
        require_keys_eq!(item.token_mint, pool_token_mint.key());

        let spl_stake_pool_service = SPLStakePoolService::<T>::new(
            pool_program,
            pool_account,
            pool_token_mint,
            pool_token_program,
        )?;

        // first update stake pool balance
        spl_stake_pool_service.update_stake_pool_balance_if_needed(
            withdraw_authority,
            reserve_stake_account,
            manager_fee_account,
            validator_list_account,
            clock,
        )?;

        let (
            to_pool_token_account_amount,
            minted_pool_token_amount,
            deducted_pool_token_fee_amount,
        ) = spl_stake_pool_service.deposit_sol(
            withdraw_authority,
            reserve_stake_account,
            manager_fee_account,
            fund_supported_token_reserve_account,
            fund_reserve_account,
            &[&ctx.fund_account.load()?.get_reserve_account_seeds()],
            item.allocated_sol_amount,
        )?;

        // pricing service with updated token values
        let pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
            .new_pricing_service(pricing_sources.iter().copied())?;

        // validation (expects diff <= 1)
        let expected_pool_token_fee_amount = pricing_service
            .get_sol_amount_as_token(pool_token_mint.key, item.allocated_sol_amount)?
            .saturating_sub(minted_pool_token_amount);
        require_gte!(
            1,
            expected_pool_token_fee_amount.abs_diff(deducted_pool_token_fee_amount),
        );

        // calculate deducted fee as SOL (will be added to SOL receivable)
        let deducted_sol_fee_amount = pricing_service
            .get_token_amount_as_sol(pool_token_mint.key, deducted_pool_token_fee_amount)?;

        Ok((
            to_pool_token_account_amount,
            minted_pool_token_amount,
            deducted_sol_fee_amount,
        ))
    }

    /// returns [to_pool_token_account_amount, minted_pool_token_amount, deducted_sol_fee_amount]
    fn marinade_stake_pool_deposit_sol<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        item: &StakeSOLCommandItem,
        pool_account_address: &Pubkey,
    ) -> Result<Option<(u64, u64, u64)>> {
        let [fund_reserve_account, fund_supported_token_reserve_account, pool_program, pool_account, pool_token_mint, pool_token_program, liq_pool_sol_leg, liq_pool_token_leg, liq_pool_token_leg_authority, pool_reserve_account, pool_token_mint_authority, pricing_sources @ ..] =
            accounts
        else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };
        require_keys_eq!(*pool_account_address, pool_account.key());
        require_keys_eq!(item.token_mint, pool_token_mint.key());

        let marinade_stake_pool_service = MarinadeStakePoolService::new(
            pool_program,
            pool_account,
            pool_token_mint,
            pool_token_program,
        )?;

        // minimum deposit amount
        if item.allocated_sol_amount < marinade_stake_pool_service.get_min_deposit_sol_amount() {
            return Ok(None);
        }

        // no fee
        let (to_pool_token_account_amount, minted_pool_token_amount) = marinade_stake_pool_service
            .deposit_sol(
                ctx.system_program,
                liq_pool_sol_leg,
                liq_pool_token_leg,
                liq_pool_token_leg_authority,
                pool_reserve_account,
                pool_token_mint_authority,
                fund_supported_token_reserve_account,
                fund_reserve_account,
                &[&ctx.fund_account.load()?.get_reserve_account_seeds()],
                item.allocated_sol_amount,
            )?;

        // pricing service with updated token values
        let pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
            .new_pricing_service(pricing_sources.iter().copied())?;

        // validation (expects diff <= 1)
        let expected_minted_pool_token_amount = pricing_service
            .get_sol_amount_as_token(pool_token_mint.key, item.allocated_sol_amount)?;
        require_gte!(
            1,
            expected_minted_pool_token_amount.abs_diff(minted_pool_token_amount)
        );

        Ok(Some((
            to_pool_token_account_amount,
            minted_pool_token_amount,
            0,
        )))
    }
}
