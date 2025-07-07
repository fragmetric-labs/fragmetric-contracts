use anchor_lang::prelude::*;
use std::ops::Neg;

use crate::errors::ErrorCode;
use crate::modules::fund::{WeightedAllocationParticipant, WeightedAllocationStrategy};
use crate::modules::pricing::TokenPricingSource;
use crate::modules::staking::*;
use crate::utils::{AccountInfoExt, AsAccountInfo, PDASeeds};

use super::{
    FundAccount, FundService, OperationCommandContext, OperationCommandEntry,
    OperationCommandResult, SelfExecutable, UnrestakeVRTCommand, FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct UnstakeLSTCommand {
    state: UnstakeLSTCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Copy)]
pub struct UnstakeLSTCommandItem {
    token_mint: Pubkey,
    allocated_token_amount: u64,
}

impl std::fmt::Debug for UnstakeLSTCommandItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.token_mint, self.allocated_token_amount)
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Default)]
pub enum UnstakeLSTCommandState {
    /// Initializes a command with items based on the fund state and strategy.
    #[default]
    New,
    /// Prepares to execute unstaking for the first item in the list.
    Prepare {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        items: Vec<UnstakeLSTCommandItem>,
    },
    /// Before execute unstaking, finds extra withdraw stake items
    /// to handle when withdrawable sol is not enough.
    GetWithdrawStakeItems {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        items: Vec<UnstakeLSTCommandItem>,
    },
    /// Executes unstaking for the first item with withdraw stake items if needed,
    /// and transitions to the next command, either preparing the next item or
    /// performing a staking operation.
    Execute {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        items: Vec<UnstakeLSTCommandItem>,
        withdraw_sol: bool,
        #[max_len(5)]
        withdraw_stake_items: Vec<WithdrawStakeItem>,
    },
}

impl std::fmt::Debug for UnstakeLSTCommandState {
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
            Self::GetWithdrawStakeItems { items } => {
                if items.is_empty() {
                    f.write_str("GetWithdrawStakeItems")
                } else {
                    f.debug_struct("GetWithdrawStakeItems")
                        .field("item", &items[0])
                        .finish()
                }
            }
            Self::Execute {
                items: unstake_command_items,
                ..
            } => {
                if unstake_command_items.is_empty() {
                    f.write_str("Execute")
                } else {
                    f.debug_struct("Execute")
                        .field("item", &unstake_command_items[0])
                        .finish()
                }
            }
        }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawStakeItem {
    validator_stake_account: Pubkey,
    fund_stake_account: Pubkey,
    fund_stake_account_index: u8,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize)]
pub struct UnstakeLSTCommandResult {
    pub token_mint: Pubkey,
    pub burnt_token_amount: u64,
    pub deducted_sol_fee_amount: u64,
    pub unstaked_sol_amount: u64,
    pub unstaking_sol_amount: u64,
    pub total_unstaking_sol_amount: u64,
    pub operation_reserved_sol_amount: u64,
    pub operation_receivable_sol_amount: u64,
    pub operation_reserved_token_amount: u64,
}

struct UnstakeResult {
    to_sol_account_amount: u64,
    burnt_token_amount: u64,
    unstaked_sol_amount: u64,
    unstaking_sol_amount: u64,
    deducted_sol_fee_amount: u64,
}

// SPL stake program requires at least 1SOL for a stake account.
// So here it assumes value of any supported LST is equal or greater than SOL.
// ref: https://github.com/solana-program/stake/blob/f5026696559ea501211e8c7e0fd0847ce3e7391c/program/src/lib.rs#L39
const SPL_STAKE_MINIMUM_DELEGATION_LAMPORTS: u64 = 1_000_000_000;

impl SelfExecutable for UnstakeLSTCommand {
    fn execute<'a, 'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let (result, entry) = match &self.state {
            UnstakeLSTCommandState::New => self.execute_new(ctx, accounts)?,
            UnstakeLSTCommandState::Prepare { items } => {
                self.execute_prepare(ctx, accounts, items.clone(), None)?
            }
            UnstakeLSTCommandState::GetWithdrawStakeItems { items } => {
                self.execute_get_withdraw_stake_items(ctx, accounts, items)?
            }
            UnstakeLSTCommandState::Execute {
                items: unstake_command_items,
                withdraw_sol,
                withdraw_stake_items,
            } => self.execute_execute(
                ctx,
                accounts,
                unstake_command_items,
                *withdraw_sol,
                withdraw_stake_items,
            )?,
        };

        Ok((
            result,
            entry.or_else(|| Some(UnrestakeVRTCommand::default().without_required_accounts())),
        ))
    }
}

// These are implementations of each command state.
#[deny(clippy::wildcard_enum_match_arm)]
impl UnstakeLSTCommand {
    /// An initial state of `UnstakeLST` command.
    /// In this state, operator iterates the fund and
    /// decides which token and how much to unstake each.
    #[inline(never)]
    fn execute_new<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let mut pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
            .new_pricing_service(accounts.iter().copied(), false)?;
        let fund_account = ctx.fund_account.load()?;
        let unstaking_obligated_amount_as_sol =
            fund_account.get_total_unstaking_obligated_amount_as_sol(&pricing_service)?;
        let mut supported_tokens_net_operation_reserved_amount =
            [0u64; FUND_ACCOUNT_MAX_SUPPORTED_TOKENS];

        if unstaking_obligated_amount_as_sol == 0 {
            Ok((None, None))
        } else {
            let mut unstaking_strategy = WeightedAllocationStrategy::<
                FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
            >::new(
                fund_account
                    .get_supported_tokens_iter()
                    .enumerate()
                    .map(|(index, supported_token)| {
                        Ok(match supported_token.pricing_source.try_deserialize()? {
                            // stakable tokens
                            Some(TokenPricingSource::SPLStakePool { .. })
                            | Some(TokenPricingSource::MarinadeStakePool { .. })
                            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool {
                                ..
                            })
                            | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool {
                                ..
                            }) => {
                                supported_tokens_net_operation_reserved_amount[index] =
                                    u64::try_from(
                                        fund_account
                                            .get_asset_net_operation_reserved_amount(
                                                Some(supported_token.mint),
                                                false,
                                                &pricing_service,
                                            )?
                                            .max(0),
                                    )?;
                                WeightedAllocationParticipant::new(
                                    supported_token.sol_allocation_weight,
                                    pricing_service.get_token_amount_as_sol(
                                        &supported_token.mint,
                                        supported_tokens_net_operation_reserved_amount[index],
                                    )? + supported_token.pending_unstaking_amount_as_sol,
                                    supported_token.sol_allocation_capacity_amount,
                                )
                            }

                            // not stakable tokens
                            Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                            | Some(TokenPricingSource::PeggedToken { .. }) => {
                                WeightedAllocationParticipant::new(0, 0, 0)
                            }

                            // invalid configuration
                            Some(TokenPricingSource::JitoRestakingVault { .. })
                            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                            | Some(TokenPricingSource::SolvBTCVault { .. })
                            | Some(TokenPricingSource::VirtualVault { .. })
                            | None => {
                                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
                            }
                            #[cfg(all(test, not(feature = "idl-build")))]
                            Some(TokenPricingSource::Mock { .. }) => {
                                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
                            }
                        })
                    })
                    .collect::<Result<Vec<_>>>()?
                    .into_iter(),
            );
            unstaking_strategy.cut_greedy(
                unstaking_obligated_amount_as_sol
                    .saturating_sub(fund_account.sol.operation_receivable_amount),
            )?;

            let mut items = Vec::with_capacity(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS);
            for (index, supported_token) in fund_account.get_supported_tokens_iter().enumerate() {
                let allocated_sol_amount =
                    unstaking_strategy.get_participant_last_cut_amount_by_index(index)?;
                let allocated_token_amount = pricing_service
                    .get_sol_amount_as_token(&supported_token.mint, allocated_sol_amount)?;

                if allocated_token_amount >= SPL_STAKE_MINIMUM_DELEGATION_LAMPORTS {
                    items.push(UnstakeLSTCommandItem {
                        token_mint: supported_token.mint,
                        // try to withdraw extra lamports to compensate for flooring errors for each token
                        allocated_token_amount: (allocated_token_amount + 1)
                            .min(supported_tokens_net_operation_reserved_amount[index]),
                    });
                }
            }
            drop(fund_account);
            FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                .update_asset_values(&mut pricing_service, true)?;

            self.execute_prepare(ctx, accounts, items, None)
        }
    }

    #[inline(never)]
    fn execute_prepare<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: Vec<UnstakeLSTCommandItem>,
        previous_execution_result: Option<OperationCommandResult>,
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if items.is_empty() {
            return Ok((previous_execution_result, None));
        }
        let item = &items[0];

        let token_pricing_source = ctx
            .fund_account
            .load()?
            .get_supported_token(&item.token_mint)?
            .pricing_source
            .try_deserialize()?;
        let pool_account = match token_pricing_source {
            Some(TokenPricingSource::SPLStakePool { address })
            | Some(TokenPricingSource::MarinadeStakePool { address })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address })
            | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { address }) => *accounts
                .iter()
                .find(|account| account.key() == address)
                .ok_or_else(|| error!(ErrorCode::FundOperationCommandExecutionFailedException))?,
            // fail when supported token is not unstakable
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
            | Some(TokenPricingSource::SolvBTCVault { .. })
            | Some(TokenPricingSource::VirtualVault { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        };

        let entry = match token_pricing_source {
            Some(TokenPricingSource::SPLStakePool { .. }) => {
                self
                .spl_stake_pool_prepare_get_withdraw_stake_items::<SPLStakePool>(
                    ctx,
                    pool_account,
                    items,
                )?},
            Some(TokenPricingSource::MarinadeStakePool { .. }) => {
                let fund_account = ctx.fund_account.load()?;
                let fund_reserve_account = fund_account.get_reserve_account_address()?;
                let fund_supported_token_reserve_account = fund_account
                    .find_supported_token_reserve_account_address(&item.token_mint)?;
                let accounts_to_order_unstake =
                    MarinadeStakePoolService::find_accounts_to_order_unstake(pool_account)?;
                let withdrawal_ticket_accounts = {
                    (0..5).map(|index| {
                        let address = *FundAccount::find_unstaking_ticket_account_address(
                            &ctx.fund_account.key(),
                            pool_account.key,
                            index,
                        );
                        (address, true)
                    })
                };

                let required_accounts = [
                    (fund_reserve_account, false),
                    (fund_supported_token_reserve_account, true)
                ]
                .into_iter()
                .chain(accounts_to_order_unstake)
                .chain(withdrawal_ticket_accounts);

                Self {
                    // Neither withdraw sol nor stake... will order unstake!
                    state: UnstakeLSTCommandState::Execute {
                        items,
                        withdraw_sol: false,
                        withdraw_stake_items: vec![],
                    },
                }
                .with_required_accounts(required_accounts)
            }
            Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. }) => {
                self.spl_stake_pool_prepare_get_withdraw_stake_items::<SanctumSingleValidatorSPLStakePool>(
                    ctx,
                    pool_account,
                    items,
                )?
            }
            Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. }) => {
                self.spl_stake_pool_prepare_get_withdraw_stake_items::<SanctumMultiValidatorSPLStakePool>(
                    ctx,
                    pool_account,
                    items,
                )?
            }
            // fail when supported token is not unstakable
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
            | Some(TokenPricingSource::SolvBTCVault { .. })
            | Some(TokenPricingSource::VirtualVault { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        };

        Ok((previous_execution_result, Some(entry)))
    }

    fn spl_stake_pool_prepare_get_withdraw_stake_items<'info, T: SPLStakePoolInterface>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        pool_account: &'info AccountInfo<'info>,
        items: Vec<UnstakeLSTCommandItem>,
    ) -> Result<OperationCommandEntry> {
        let accounts_to_get_validator_stake_accounts =
            SPLStakePoolService::<T>::find_accounts_to_get_validator_stake_accounts(pool_account)?;
        let fund_stake_accounts = {
            (0..5).map(|index| {
                let address = *FundAccount::find_stake_account_address(
                    &ctx.fund_account.key(),
                    pool_account.key,
                    index,
                );
                (address, false)
            })
        };

        let required_accounts = accounts_to_get_validator_stake_accounts.chain(fund_stake_accounts);

        Ok(Self {
            state: UnstakeLSTCommandState::GetWithdrawStakeItems { items },
        }
        .with_required_accounts(required_accounts))
    }

    #[inline(never)]
    fn execute_get_withdraw_stake_items<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: &[UnstakeLSTCommandItem],
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

        let entry = match token_pricing_source {
            Some(TokenPricingSource::SPLStakePool { address }) => self
                .spl_stake_pool_get_withdraw_stake_items::<SPLStakePool>(
                    ctx, accounts, items, item, address,
                ),
            Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address }) => {
                self.spl_stake_pool_get_withdraw_stake_items::<SanctumSingleValidatorSPLStakePool>(
                    ctx, accounts, items, item, address,
                )
            }
            Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { address }) => {
                self.spl_stake_pool_get_withdraw_stake_items::<SanctumMultiValidatorSPLStakePool>(
                    ctx, accounts, items, item, address,
                )
            }
            // otherwise fails
            Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
            | Some(TokenPricingSource::SolvBTCVault { .. })
            | Some(TokenPricingSource::VirtualVault { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        }?;

        Ok((None, Some(entry)))
    }

    fn spl_stake_pool_get_withdraw_stake_items<'info, T: SPLStakePoolInterface>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: &[UnstakeLSTCommandItem],
        current_item: &UnstakeLSTCommandItem,
        pool_account_address: Pubkey,
    ) -> Result<OperationCommandEntry> {
        let [pool_program, pool_account, pool_token_mint, pool_token_program, validator_list_account, remaining_accounts @ ..] =
            accounts
        else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };
        let fund_stake_accounts = {
            if remaining_accounts.len() < 5 {
                err!(error::ErrorCode::AccountNotEnoughKeys)?;
            }
            &remaining_accounts[..5]
        };

        require_keys_eq!(pool_account_address, pool_account.key());
        require_keys_eq!(current_item.token_mint, pool_token_mint.key());
        for (index, fund_stake_account) in fund_stake_accounts.iter().enumerate() {
            let fund_stake_account_address = *FundAccount::find_stake_account_address(
                &ctx.fund_account.key(),
                pool_account.key,
                index as u8,
            );
            require_keys_eq!(fund_stake_account_address, fund_stake_account.key());
        }

        let spl_stake_pool_service = SPLStakePoolService::<T>::new(
            pool_program,
            pool_account,
            pool_token_mint,
            pool_token_program,
        )?;

        let available_fund_stake_accounts_indices = fund_stake_accounts
            .iter()
            .enumerate()
            .filter_map(|(index, fund_stake_account)| {
                (!fund_stake_account.is_initialized()).then_some(index)
            })
            .collect::<Vec<_>>();

        // Maximum number of validators = # of available(uninitialized) fund stake accounts
        let max_num_validators = available_fund_stake_accounts_indices.len();
        let validator_stake_accounts = spl_stake_pool_service
            .get_validator_stake_accounts(validator_list_account, max_num_validators)?;
        // Update to actual # of validators
        let num_validators = validator_stake_accounts.len();

        let withdraw_stake_items = available_fund_stake_accounts_indices
            .iter()
            .zip(&validator_stake_accounts)
            .map(
                |(&fund_stake_account_index, &validator_stake_account)| WithdrawStakeItem {
                    validator_stake_account,
                    fund_stake_account: fund_stake_accounts[fund_stake_account_index].key(),
                    fund_stake_account_index: fund_stake_account_index as u8,
                },
            )
            .collect();

        let fund_account = ctx.fund_account.load()?;
        let fund_reserve_account = fund_account.get_reserve_account_address()?;
        let fund_supported_token_reserve_account =
            fund_account.find_supported_token_reserve_account_address(pool_token_mint.key)?;

        let accounts_to_withdraw =
            SPLStakePoolService::<T>::find_accounts_to_withdraw(pool_account)?;
        let fund_stake_accounts = available_fund_stake_accounts_indices
            .into_iter()
            .take(num_validators)
            .map(|index| (fund_stake_accounts[index].key(), true));
        let validator_stake_accounts = validator_stake_accounts
            .into_iter()
            .map(|address| (address, true));

        let required_accounts = [
            (fund_reserve_account, true),
            (fund_supported_token_reserve_account, true),
        ]
        .into_iter()
        .chain(accounts_to_withdraw)
        .chain(fund_stake_accounts)
        .chain(validator_stake_accounts);

        Ok(Self {
            state: UnstakeLSTCommandState::Execute {
                items: items.to_vec(),
                withdraw_sol: true,
                withdraw_stake_items,
            },
        }
        .with_required_accounts(required_accounts))
    }

    #[inline(never)]
    fn execute_execute<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        unstake_command_items: &[UnstakeLSTCommandItem],
        withdraw_sol: bool,
        withdraw_stake_items: &[WithdrawStakeItem],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if unstake_command_items.is_empty() {
            return Ok((None, None));
        }
        let item = &unstake_command_items[0];
        let token_pricing_source = ctx
            .fund_account
            .load()?
            .get_supported_token(&item.token_mint)?
            .pricing_source
            .try_deserialize()?;

        // Execute command might be paused...
        // mostly due to memory limit.
        let mut resume_execution_command = None;
        let result = match token_pricing_source {
            Some(TokenPricingSource::SPLStakePool { address }) => self
                .spl_stake_pool_withdraw_sol_or_stake::<SPLStakePool>(
                    ctx,
                    accounts,
                    unstake_command_items,
                    withdraw_sol,
                    withdraw_stake_items,
                    address,
                    &mut resume_execution_command,
                )?,
            Some(TokenPricingSource::MarinadeStakePool { address }) => {
                require_eq!(withdraw_stake_items.len(), 0);

                // Marinade stake pool neither withdraw sol nor stake... just order unstake at once!
                self.marinade_stake_pool_order_unstake(ctx, accounts, item, address)?
            }
            Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address }) => {
                self.spl_stake_pool_withdraw_sol_or_stake::<SanctumSingleValidatorSPLStakePool>(
                    ctx,
                    accounts,
                    unstake_command_items,
                    withdraw_sol,
                    withdraw_stake_items,
                    address,
                    &mut resume_execution_command,
                )?
            }
            Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { address }) => {
                self.spl_stake_pool_withdraw_sol_or_stake::<SanctumMultiValidatorSPLStakePool>(
                    ctx,
                    accounts,
                    unstake_command_items,
                    withdraw_sol,
                    withdraw_stake_items,
                    address,
                    &mut resume_execution_command,
                )?
            }
            // fail when supported token is not unstakable
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
            | Some(TokenPricingSource::SolvBTCVault { .. })
            | Some(TokenPricingSource::VirtualVault { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        }
        .map(
            |UnstakeResult {
                 to_sol_account_amount,
                 burnt_token_amount,
                 unstaked_sol_amount,
                 unstaking_sol_amount,
                 deducted_sol_fee_amount,
             }| {
                // Update fund account
                let mut fund_account = ctx.fund_account.load_mut()?;
                fund_account.sol.operation_reserved_amount += unstaked_sol_amount;
                fund_account.sol.operation_receivable_amount +=
                    unstaking_sol_amount + deducted_sol_fee_amount;

                require_gte!(
                    to_sol_account_amount,
                    fund_account.sol.get_total_reserved_amount(),
                );

                let supported_token = fund_account.get_supported_token_mut(&item.token_mint)?;
                supported_token.token.operation_reserved_amount -= burnt_token_amount;
                supported_token.pending_unstaking_amount_as_sol += unstaking_sol_amount;

                Ok(UnstakeLSTCommandResult {
                    token_mint: item.token_mint,
                    burnt_token_amount,
                    deducted_sol_fee_amount,
                    unstaked_sol_amount,
                    unstaking_sol_amount,
                    total_unstaking_sol_amount: supported_token.pending_unstaking_amount_as_sol,
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

        if resume_execution_command.is_some() {
            return Ok((result, resume_execution_command));
        }

        // prepare state does not require additional accounts,
        // so we can execute directly.
        self.execute_prepare(ctx, accounts, unstake_command_items[1..].to_vec(), result)
    }

    fn spl_stake_pool_withdraw_sol_or_stake<'info, T: SPLStakePoolInterface>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        unstake_command_items: &[UnstakeLSTCommandItem],
        withdraw_sol: bool,
        withdraw_stake_items: &[WithdrawStakeItem],
        pool_account_address: Pubkey,
        resume_withdraw_stake_command: &mut Option<OperationCommandEntry>,
    ) -> Result<Option<UnstakeResult>> {
        let unstake_command_item = &unstake_command_items[0];
        let [fund_reserve_account, fund_supported_token_reserve_account, pool_program, pool_account, pool_token_mint, pool_token_program, withdraw_authority, reserve_stake_account, manager_fee_account, validator_list_account, clock, stake_history, stake_program, remaining_accounts @ ..] =
            accounts
        else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };

        let num_items = withdraw_stake_items.len();
        if remaining_accounts.len() < 2 * num_items {
            err!(error::ErrorCode::AccountNotEnoughKeys)?;
        }

        let (fund_stake_accounts, remaining_accounts) = remaining_accounts.split_at(num_items);
        let (validator_stake_accounts, pricing_sources) = remaining_accounts.split_at(num_items);

        require_keys_eq!(pool_account_address, pool_account.key());
        require_keys_eq!(unstake_command_item.token_mint, pool_token_mint.key());
        for index in 0..num_items {
            require_keys_eq!(
                withdraw_stake_items[index].fund_stake_account,
                fund_stake_accounts[index].key()
            );
            require_keys_eq!(
                withdraw_stake_items[index].validator_stake_account,
                validator_stake_accounts[index].key(),
            );
        }

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

        let fund_account = ctx.fund_account.load()?;

        // Statistics
        let mut total_token_amount_to_burn = unstake_command_item.allocated_token_amount;
        let mut total_unstaked_sol_amount = 0;
        let mut total_unstaking_sol_amount = 0;
        let mut total_deducted_pool_token_fee_amount = 0;

        // Withdraw SOL first
        // To test withdraw stake, comment out this block or adjust `pool_token_amount` parameter
        if withdraw_sol {
            let (burnt_pool_token_amount, unstaked_sol_amount, deducted_pool_token_fee_amount) =
                spl_stake_pool_service.withdraw_sol(
                    withdraw_authority,
                    reserve_stake_account,
                    manager_fee_account,
                    clock,
                    stake_history,
                    stake_program,
                    fund_reserve_account,
                    fund_supported_token_reserve_account,
                    fund_reserve_account,
                    &[&fund_account.get_reserve_account_seeds()],
                    total_token_amount_to_burn,
                )?;
            total_token_amount_to_burn -= burnt_pool_token_amount;
            total_unstaked_sol_amount += unstaked_sol_amount;
            total_deducted_pool_token_fee_amount += deducted_pool_token_fee_amount;
        }

        // Withdraw stake limit at a single operation tx due to memory limit
        const WITHDRAW_STAKE_LIMIT: usize = 2;
        let mut withdraw_stake_count = 0;
        let mut withdraw_stake_paused = false;
        let mut withdraw_stake_resuming_index = 0;

        for index in 0..num_items {
            // No more tokens to burn... terminate withdraw stake
            if total_token_amount_to_burn == 0 {
                break;
            }

            // Reached withdraw stake limit... pause withdraw stake
            if withdraw_stake_count == WITHDRAW_STAKE_LIMIT {
                withdraw_stake_paused = true;
                withdraw_stake_resuming_index = index;
                break;
            }

            let (burnt_pool_token_amount, unstaking_sol_amount, deducted_pool_token_fee_amount) =
                spl_stake_pool_service.withdraw_stake(
                    ctx.system_program,
                    withdraw_authority,
                    manager_fee_account,
                    validator_list_account,
                    clock,
                    stake_program,
                    validator_stake_accounts[index],
                    fund_stake_accounts[index],
                    &[&FundAccount::find_stake_account_address(
                        &ctx.fund_account.key(),
                        pool_account.key,
                        withdraw_stake_items[index].fund_stake_account_index,
                    )
                    .get_seeds()],
                    ctx.operator,
                    ctx.fund_account.as_account_info(),
                    &[&fund_account.get_seeds()],
                    fund_supported_token_reserve_account,
                    fund_reserve_account,
                    &[&fund_account.get_reserve_account_seeds()],
                    total_token_amount_to_burn,
                )?;
            total_token_amount_to_burn -= burnt_pool_token_amount;
            total_unstaking_sol_amount += unstaking_sol_amount;
            total_deducted_pool_token_fee_amount += deducted_pool_token_fee_amount;

            if burnt_pool_token_amount > 0 {
                withdraw_stake_count += 1;
            }
        }

        // Neither withdraw sol nor stake was impossible
        if total_token_amount_to_burn == unstake_command_item.allocated_token_amount {
            return Ok(None);
        }

        // When withdraw stake has paused, we will continue execution on next transaction
        if withdraw_stake_paused {
            let withdraw_stake_items =
                withdraw_stake_items[withdraw_stake_resuming_index..].to_vec();
            let mut unstake_command_items = unstake_command_items.to_vec();
            unstake_command_items[0].allocated_token_amount = total_token_amount_to_burn;

            let command = Self {
                state: UnstakeLSTCommandState::Execute {
                    items: unstake_command_items,
                    withdraw_sol: false,
                    withdraw_stake_items,
                },
            };

            // fund_reserve_account .. stake_program (13 accounts)
            let accounts_to_execute = (0..13).map(|i| (accounts[i].key(), accounts[i].is_writable));
            let fund_stake_accounts = fund_stake_accounts[withdraw_stake_resuming_index..]
                .iter()
                .map(|account| (account.key(), account.is_writable));
            let validator_stake_accounts = validator_stake_accounts
                [withdraw_stake_resuming_index..]
                .iter()
                .map(|account| (account.key(), account.is_writable));

            let required_accounts = accounts_to_execute
                .chain(fund_stake_accounts)
                .chain(validator_stake_accounts);

            // create command entry
            *resume_withdraw_stake_command =
                Some(command.with_required_accounts(required_accounts));
        }

        drop(fund_account);

        // pricing service with updated token values
        let mut pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
            .new_pricing_service(pricing_sources.iter().copied(), false)?;

        // fee validation
        let total_burnt_token_amount =
            unstake_command_item.allocated_token_amount - total_token_amount_to_burn;
        let expected_pool_token_fee_amount =
            total_burnt_token_amount.saturating_sub(pricing_service.get_sol_amount_as_token(
                pool_token_mint.key,
                total_unstaking_sol_amount + total_unstaked_sol_amount,
            )?);
        require_gte!(
            1,
            expected_pool_token_fee_amount.abs_diff(total_deducted_pool_token_fee_amount)
        );

        // calculate deducted fee as SOL (will be added to SOL receivable)
        let total_deducted_sol_fee_amount = pricing_service
            .get_token_amount_as_sol(pool_token_mint.key, total_deducted_pool_token_fee_amount)?;

        FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
            .update_asset_values(&mut pricing_service, true)?;

        Ok(Some(UnstakeResult {
            to_sol_account_amount: fund_reserve_account.lamports(),
            burnt_token_amount: total_burnt_token_amount,
            unstaked_sol_amount: total_unstaked_sol_amount,
            unstaking_sol_amount: total_unstaking_sol_amount,
            deducted_sol_fee_amount: total_deducted_sol_fee_amount,
        }))
    }

    fn marinade_stake_pool_order_unstake<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        item: &UnstakeLSTCommandItem,
        pool_account_address: Pubkey,
    ) -> Result<Option<UnstakeResult>> {
        let [fund_reserve_account, fund_supported_token_reserve_account, pool_program, pool_account, pool_token_mint, pool_token_program, clock, rent, remaining_accounts @ ..] =
            accounts
        else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };

        if remaining_accounts.len() < 5 {
            err!(error::ErrorCode::AccountNotEnoughKeys)?;
        }

        let (withdrawal_ticket_accounts, pricing_sources) = remaining_accounts.split_at(5);

        require_keys_eq!(pool_account_address, pool_account.key());
        require_keys_eq!(item.token_mint, pool_token_mint.key());
        for (index, withdrawal_ticket_account) in withdrawal_ticket_accounts.iter().enumerate() {
            let withdrawal_ticket_account_address =
                *FundAccount::find_unstaking_ticket_account_address(
                    &ctx.fund_account.key(),
                    pool_account.key,
                    index as u8,
                );
            require_keys_eq!(
                withdrawal_ticket_account_address,
                withdrawal_ticket_account.key()
            );
        }

        let Some((withdrawal_ticket_account_index, withdrawal_ticket_account)) =
            withdrawal_ticket_accounts
                .iter()
                .enumerate()
                .find(|(_, account)| !account.is_initialized())
        else {
            // there is no available(uninitialized) withdrawal ticket account
            return Ok(None);
        };

        let marinade_stake_pool_service = MarinadeStakePoolService::new(
            pool_program,
            pool_account,
            pool_token_mint,
            pool_token_program,
        )?;

        let (unstaking_sol_amount, deducted_sol_fee_amount) = {
            let fund_account = ctx.fund_account.load()?;
            marinade_stake_pool_service.order_unstake(
                ctx.system_program,
                clock,
                rent,
                withdrawal_ticket_account,
                &[&FundAccount::find_unstaking_ticket_account_address(
                    &ctx.fund_account.key(),
                    pool_account.key,
                    withdrawal_ticket_account_index as u8,
                )
                .get_seeds()],
                ctx.operator, // here, operator pays rent
                fund_supported_token_reserve_account,
                fund_reserve_account,
                &[&fund_account.get_reserve_account_seeds()],
                item.allocated_token_amount,
            )?
        };

        // pricing service with updated token values
        let pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
            .new_pricing_service(pricing_sources.iter().copied(), true)?;

        // fee validation
        let expected_sol_fee_amount = pricing_service
            .get_token_amount_as_sol(pool_token_mint.key, item.allocated_token_amount)?
            .saturating_sub(unstaking_sol_amount);
        require_gte!(expected_sol_fee_amount, deducted_sol_fee_amount);

        Ok(Some(UnstakeResult {
            to_sol_account_amount: fund_reserve_account.lamports(),
            burnt_token_amount: item.allocated_token_amount,
            unstaked_sol_amount: 0,
            unstaking_sol_amount,
            deducted_sol_fee_amount,
        }))
    }
}
