use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use std::ops::Neg;

use crate::errors;
use crate::modules::normalization::*;

use super::{
    FundService, OperationCommandContext, OperationCommandEntry, OperationCommandResult,
    RestakeVSTCommand, SelfExecutable, WeightedAllocationParticipant, WeightedAllocationStrategy,
    FUND_ACCOUNT_MAX_RESTAKING_VAULTS, FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct NormalizeSTCommand {
    state: NormalizeSTCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Copy)]
pub struct NormalizeSTCommandItem {
    supported_token_mint: Pubkey,
    allocated_token_amount: u64,
}

impl std::fmt::Debug for NormalizeSTCommandItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({})",
            self.supported_token_mint, self.allocated_token_amount
        )
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Default)]
pub enum NormalizeSTCommandState {
    /// Initializes a command with items based on the fund state and strategy.
    #[default]
    New,
    /// Prepares to execute normalization for the first item in the list.
    Prepare {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        items: Vec<NormalizeSTCommandItem>,
    },
    /// Executes normalization for the first item and transitions to the next command,
    /// either preparing the next item or performing a restaking operation.
    Execute {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        items: Vec<NormalizeSTCommandItem>,
    },
}

impl std::fmt::Debug for NormalizeSTCommandState {
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
pub struct NormalizeSTCommandResult {
    pub supported_token_mint: Pubkey,
    pub normalized_supported_token_amount: u64,
    pub minted_token_amount: u64,
    pub operation_reserved_token_amount: u64,
}

const NTP_MINIMUM_DEPOSIT_LAMPORTS: u64 = 1_000_000_000;

impl SelfExecutable for NormalizeSTCommand {
    fn execute<'a, 'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let (result, entry) = match &self.state {
            NormalizeSTCommandState::New => self.execute_new(ctx, accounts)?,
            NormalizeSTCommandState::Prepare { items } => {
                self.execute_prepare(ctx, accounts, items.clone(), None)?
            }
            NormalizeSTCommandState::Execute { items } => {
                self.execute_execute(ctx, accounts, items)?
            }
        };

        Ok((
            result,
            entry.or_else(|| Some(RestakeVSTCommand::default().without_required_accounts())),
        ))
    }
}

// These are implementations of each command state.
impl NormalizeSTCommand {
    /// An initial state of `NormalizeST` command.
    /// In this state, operator iterates the fund and
    /// decides which token and how much to normalize each.
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

        // If fund does not support normalization then nothing to normalize.
        let Some(normalized_token) = fund_account.get_normalized_token() else {
            return Ok((None, None));
        };

        // If none of the restaking vaults support normalized token,
        // we cannot restake normalized token so we don't need to normalize.
        if !fund_account
            .get_restaking_vaults_iter()
            .any(|restaking_vault| restaking_vault.supported_token_mint == normalized_token.mint)
        {
            return Ok((None, None));
        }

        let normalized_token_pool_account = fund_account
            .get_normalized_token_pool_address()
            .and_then(|address| {
                accounts
                    .iter()
                    .find(|account| account.key() == address)
                    .copied()
            })
            .ok_or_else(|| error!(errors::ErrorCode::FundOperationCommandExecutionFailedException))
            .and_then(Account::<NormalizedTokenPoolAccount>::try_from)?;

        // here, we allocate with maximum capacity to ensure that
        // the program will not run out of memory even when more
        // supported tokens are added to fund in the future.
        let mut items =
            Vec::<NormalizeSTCommandItem>::with_capacity(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS);

        // create a strategy to reflect unstaking obligated amount for lack of reserved SOL
        let mut token_strategy =
            WeightedAllocationStrategy::<FUND_ACCOUNT_MAX_SUPPORTED_TOKENS>::new(
                fund_account
                    .get_supported_tokens_iter()
                    .map(|supported_token| {
                        Ok(WeightedAllocationParticipant::new(
                            supported_token.sol_allocation_weight,
                            pricing_service.get_token_amount_as_sol(
                                &supported_token.mint,
                                u64::try_from(
                                    fund_account
                                        .get_asset_net_operation_reserved_amount(
                                            Some(supported_token.mint),
                                            true,
                                            &pricing_service,
                                        )?
                                        .max(0),
                                )?,
                            )?,
                            supported_token.sol_allocation_capacity_amount,
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?,
            );
        let total_unstaking_required_amount_as_sol = u64::try_from(
            fund_account
                .get_asset_net_operation_reserved_amount(None, true, &pricing_service)?
                .min(0)
                .neg(),
        )?;
        token_strategy.cut_greedy(total_unstaking_required_amount_as_sol)?;

        for (index, supported_token) in fund_account.get_supported_tokens_iter().enumerate() {
            // if supported token is not normalizable then we cannot normalize.
            if !normalized_token_pool_account.has_supported_token(&supported_token.mint) {
                continue;
            }

            // if supported token does not have enough reserved amount then we cannot normalize.
            let supported_token_restakable_amount_as_sol = token_strategy
                .get_participant_by_index(index)?
                .allocated_amount;
            if supported_token_restakable_amount_as_sol == 0 {
                continue;
            }

            // find vaults that supported token can be move into, either directly or indirectly(normalized).
            let restakable_vaults =
                fund_account
                    .get_restaking_vaults_iter()
                    .filter(|restaking_vault| {
                        restaking_vault.supported_token_mint == supported_token.mint
                            || restaking_vault.supported_token_mint == normalized_token.mint
                    });

            let mut vault_strategy =
                WeightedAllocationStrategy::<FUND_ACCOUNT_MAX_RESTAKING_VAULTS>::new(
                    restakable_vaults
                        .map(|restaking_vault| {
                            Ok(WeightedAllocationParticipant::new(
                                restaking_vault.sol_allocation_weight,
                                pricing_service.get_token_amount_as_sol(
                                    &restaking_vault.receipt_token_mint,
                                    restaking_vault.receipt_token_operation_reserved_amount,
                                )?,
                                restaking_vault.sol_allocation_capacity_amount,
                            ))
                        })
                        .collect::<Result<Vec<_>>>()?,
                );
            vault_strategy.put(
                // try to withdraw extra lamports to compensate for flooring errors for each token
                supported_token_restakable_amount_as_sol
                    + normalized_token_pool_account.get_num_supported_tokens() as u64,
            )?;

            let mut allocated_sol_amount_for_normalized_token_vaults = 0;

            // to avoid memory allocation we iterate vaults again.
            let restakable_vaults =
                fund_account
                    .get_restaking_vaults_iter()
                    .filter(|restaking_vault| {
                        restaking_vault.supported_token_mint == supported_token.mint
                            || restaking_vault.supported_token_mint == normalized_token.mint
                    });
            for (index, restakable_vault) in restakable_vaults.enumerate() {
                if restakable_vault.supported_token_mint == normalized_token.mint {
                    allocated_sol_amount_for_normalized_token_vaults +=
                        vault_strategy.get_participant_last_put_amount_by_index(index)?;
                }
            }

            if allocated_sol_amount_for_normalized_token_vaults >= NTP_MINIMUM_DEPOSIT_LAMPORTS {
                items.push(NormalizeSTCommandItem {
                    supported_token_mint: supported_token.mint,
                    allocated_token_amount: pricing_service
                        .get_sol_amount_as_token(
                            &supported_token.mint,
                            allocated_sol_amount_for_normalized_token_vaults,
                        )?
                        .min(supported_token.token.operation_reserved_amount),
                });
            }
        }

        // prepare state does not require additional accounts,
        // so we can execute directly.
        drop(fund_account);
        FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
            .update_asset_values(&mut pricing_service, true)?;

        self.execute_prepare(ctx, accounts, items, None)
    }

    #[inline(never)]
    fn execute_prepare<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: Vec<NormalizeSTCommandItem>,
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
        let supported_token = fund_account.get_supported_token(&item.supported_token_mint)?;
        let normalized_token_pool_account_info = fund_account
            .get_normalized_token_pool_address()
            .and_then(|address| {
                accounts
                    .iter()
                    .find(|account| account.key() == address)
                    .copied()
            })
            .ok_or_else(|| {
                error!(errors::ErrorCode::FundOperationCommandExecutionFailedException)
            })?;

        // Prepare
        let accounts_to_normalize_supported_token =
            NormalizedTokenPoolService::find_accounts_to_normalize_supported_token(
                normalized_token_pool_account_info,
                &supported_token.mint,
            )?;
        let fund_normalized_token_reserve_account =
            fund_account.find_normalized_token_reserve_account_address()?;
        let fund_supported_token_reserve_account =
            fund_account.find_supported_token_reserve_account_address(&supported_token.mint)?;
        let fund_reserve_account = fund_account.get_reserve_account_address()?;

        let required_accounts = accounts_to_normalize_supported_token.chain([
            // to_normalized_token_account
            (fund_normalized_token_reserve_account, true),
            // from_supported_token_account
            (fund_supported_token_reserve_account, true),
            // from_supported_token_account_signer
            (fund_reserve_account, false),
        ]);

        let command = Self {
            state: NormalizeSTCommandState::Execute { items },
        }
        .with_required_accounts(required_accounts);

        Ok((previous_execution_result, Some(command)))
    }

    #[inline(never)]
    fn execute_execute<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: &[NormalizeSTCommandItem],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if items.is_empty() {
            return Ok((None, None));
        }
        let item = items[0];

        let [normalized_token_pool_account, normalized_token_mint, normalized_token_program, supported_token_mint, supported_token_program, supported_token_reserve_account, to_normalized_token_account, from_supported_token_account, fund_reserve_account, pricing_sources @ ..] =
            accounts
        else {
            err!(ErrorCode::AccountNotEnoughKeys)?
        };

        let mut pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
            .new_pricing_service(pricing_sources.iter().copied(), false)?;

        let normalized_token_mint_address = normalized_token_mint.key();
        let mut normalized_token_pool_account =
            Account::<NormalizedTokenPoolAccount>::try_from(normalized_token_pool_account)?;
        let mut normalized_token_mint = InterfaceAccount::<Mint>::try_from(normalized_token_mint)?;
        let normalized_token_program = Program::<Token>::try_from(*normalized_token_program)?;
        let supported_token_mint = InterfaceAccount::<Mint>::try_from(supported_token_mint)?;
        let supported_token_program =
            Interface::<TokenInterface>::try_from(*supported_token_program)?;
        let supported_token_reserve_account =
            InterfaceAccount::<TokenAccount>::try_from(supported_token_reserve_account)?;
        let mut to_normalized_token_account =
            InterfaceAccount::<TokenAccount>::try_from(to_normalized_token_account)?;
        let from_supported_token_account =
            InterfaceAccount::<TokenAccount>::try_from(from_supported_token_account)?;

        let mut normalized_token_pool_service = NormalizedTokenPoolService::new(
            &mut normalized_token_pool_account,
            &mut normalized_token_mint,
            &normalized_token_program,
        )?;

        let (to_normalized_token_account_amount, minted_normalized_token_amount) =
            normalized_token_pool_service.normalize_supported_token(
                &supported_token_mint,
                &supported_token_program,
                &supported_token_reserve_account,
                &mut to_normalized_token_account,
                &from_supported_token_account,
                fund_reserve_account,
                &[&ctx.fund_account.load()?.get_reserve_account_seeds()],
                item.allocated_token_amount,
                &mut pricing_service,
            )?;

        // validation (expected diff <= 1)
        let expected_minted_normalized_token_amount = pricing_service.get_sol_amount_as_token(
            &normalized_token_mint_address,
            pricing_service
                .get_token_amount_as_sol(&item.supported_token_mint, item.allocated_token_amount)?,
        )?;
        require_gte!(
            1,
            expected_minted_normalized_token_amount.abs_diff(minted_normalized_token_amount)
        );

        // update fund account
        let mut fund_account = ctx.fund_account.load_mut()?;

        let supported_token = fund_account.get_supported_token_mut(&item.supported_token_mint)?;
        supported_token.token.operation_reserved_amount -= item.allocated_token_amount;

        let normalized_token = fund_account.get_normalized_token_mut().unwrap();
        normalized_token.operation_reserved_amount += minted_normalized_token_amount;

        require_gte!(
            to_normalized_token_account_amount,
            normalized_token.operation_reserved_amount,
        );

        let result = Some(
            NormalizeSTCommandResult {
                supported_token_mint: item.supported_token_mint,
                normalized_supported_token_amount: item.allocated_token_amount,
                minted_token_amount: minted_normalized_token_amount,
                operation_reserved_token_amount: normalized_token.operation_reserved_amount,
            }
            .into(),
        );

        drop(fund_account);
        FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
            .update_asset_values(&mut pricing_service, true)?;

        let command = Self {
            state: NormalizeSTCommandState::Prepare {
                items: items[1..].to_vec(),
            },
        }
        .without_required_accounts();

        Ok((result, Some(command)))
    }
}
