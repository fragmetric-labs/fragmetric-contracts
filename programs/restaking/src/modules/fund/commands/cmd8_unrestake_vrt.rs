use std::cell::Ref;
use std::iter::Peekable;
use std::ops::Neg;

use anchor_lang::prelude::*;
use anchor_spl::associated_token;

use crate::modules::normalization::NormalizedTokenPoolAccount;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::JitoRestakingVaultService;
use crate::utils::{AccountInfoExt, PDASeeds};
use crate::{errors, modules::restaking::SolvBTCVaultService};

use super::{
    FundAccount, FundService, OperationCommandContext, OperationCommandEntry,
    OperationCommandResult, SelfExecutable, UndelegateVSTCommand, WeightedAllocationParticipant,
    WeightedAllocationStrategy, FUND_ACCOUNT_MAX_RESTAKING_VAULTS,
    FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct UnrestakeVRTCommand {
    state: UnrestakeVRTCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct UnrestakeVSTCommandItem {
    vault: Pubkey,
    receipt_token_mint: Pubkey,
    supported_token_mint: Pubkey,
    allocated_receipt_token_amount: u64,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum UnrestakeVRTCommandState {
    #[default]
    New,
    Prepare {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        items: Vec<UnrestakeVSTCommandItem>,
    },
    Execute {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        items: Vec<UnrestakeVSTCommandItem>,
    },
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct UnrestakeVRTCommandResult {
    pub vault: Pubkey,
    pub token_mint: Pubkey,
    pub unrestaking_token_amount: u64,
    pub total_unrestaking_token_amount: u64,
    pub operation_reserved_token_amount: u64,
}

const RESTAKING_MINIMUM_WITHDRAWAL_LAMPORTS: u64 = 1_000_000_000;

#[deny(clippy::wildcard_enum_match_arm)]
impl SelfExecutable for UnrestakeVRTCommand {
    fn execute<'a, 'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let (result, entry) = match &self.state {
            UnrestakeVRTCommandState::New => self.execute_new(ctx, accounts)?,
            UnrestakeVRTCommandState::Prepare { items } => {
                self.execute_prepare(ctx, accounts, items)?
            }
            UnrestakeVRTCommandState::Execute { items } => {
                self.execute_execute(ctx, accounts, items)?
            }
        };

        Ok((
            result,
            entry.or_else(|| Some(UndelegateVSTCommand::default().without_required_accounts())),
        ))
    }
}

#[deny(clippy::wildcard_enum_match_arm)]
impl UnrestakeVRTCommand {
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

        let normalized_token_pool_account = fund_account
            .get_normalized_token_pool_address()
            .and_then(|address| {
                accounts
                    .iter()
                    .find(|account| account.key() == address)
                    .copied()
            })
            .map(Account::<NormalizedTokenPoolAccount>::try_from)
            .transpose()?;
        let normalized_token_pool_account = normalized_token_pool_account.as_ref();

        // a strategy with supported tokens
        let mut extra_unrestaking_strategy =
            WeightedAllocationStrategy::<FUND_ACCOUNT_MAX_SUPPORTED_TOKENS>::new(
                fund_account
                    .get_supported_tokens_iter()
                    .map(|supported_token| {
                        WeightedAllocationParticipant::new(
                            supported_token.sol_allocation_weight,
                            0,
                            supported_token.sol_allocation_capacity_amount,
                        )
                    }),
            );

        // calculate additionally required unrestaking amount for each tokens to meet SOL withdrawal obligation
        let mut extra_unrestaking_obligated_amount_as_sol =
            fund_account.get_total_unstaking_obligated_amount_as_sol(&pricing_service)?;

        // and will calculate mandatory unrestaking amount for each tokens to meet token withdrawal obligation
        let mut unrestaking_obligated_amounts_as_sol: [u64; FUND_ACCOUNT_MAX_SUPPORTED_TOKENS] =
            [0; FUND_ACCOUNT_MAX_SUPPORTED_TOKENS];

        // reflect ready to unstake amount of normalized token
        if let Some(pool) = normalized_token_pool_account {
            extra_unrestaking_obligated_amount_as_sol = extra_unrestaking_obligated_amount_as_sol
                .saturating_sub(
                    pricing_service.get_token_amount_as_sol(
                        &pool.normalized_token_mint,
                        fund_account
                            .get_normalized_token()
                            .unwrap()
                            .operation_reserved_amount,
                    )?,
                );
        }

        for (supported_token_index, supported_token) in
            fund_account.get_supported_tokens_iter().enumerate()
        {
            // reflect already unstaked, unstaking amount of this supported token
            let unstaking_reserved_amount_as_sol = pricing_service.get_token_amount_as_sol(
                &supported_token.mint,
                u64::try_from(
                    fund_account
                        .get_asset_net_operation_reserved_amount(
                            Some(supported_token.mint),
                            false,
                            &pricing_service,
                        )?
                        .saturating_add(supported_token.pending_unstaking_amount_as_sol as i128)
                        .max(0),
                )?,
            )?;
            extra_unrestaking_obligated_amount_as_sol = extra_unrestaking_obligated_amount_as_sol
                .saturating_sub(unstaking_reserved_amount_as_sol);

            // the amount to unrestake for withdrawal obligation of this token itself
            unrestaking_obligated_amounts_as_sol[supported_token_index] = pricing_service
                .get_token_amount_as_sol(
                    &supported_token.mint,
                    u64::try_from(
                        fund_account
                            .get_asset_net_operation_reserved_amount(
                                Some(supported_token.mint),
                                true,
                                &pricing_service,
                            )?
                            .min(0)
                            .neg(),
                    )?,
                )?;

            // iterator for (restaking_vault, is_normalized_token_vault)
            let unrestakable_vaults_iter =
                fund_account
                    .get_restaking_vaults_iter()
                    .filter_map(|restaking_vault| {
                        if restaking_vault.supported_token_mint == supported_token.mint {
                            Some((restaking_vault, false))
                        } else if normalized_token_pool_account.is_some_and(|pool| {
                            pool.normalized_token_mint == restaking_vault.supported_token_mint
                                && pool.has_supported_token(&supported_token.mint)
                        }) {
                            Some((restaking_vault, true))
                        } else {
                            None
                        }
                    });

            // sum remaining unrestakable amount of this supported token
            let extra_unrestaking_strategy_participant =
                extra_unrestaking_strategy.get_participant_by_index_mut(supported_token_index)?;
            for (restaking_vault, is_normalized_token_vault) in unrestakable_vaults_iter {
                extra_unrestaking_strategy_participant.allocated_amount += {
                    if is_normalized_token_vault {
                        // calculate supported token amount in normalized token pool proportionally
                        let pool = normalized_token_pool_account.unwrap();
                        pricing_service.get_token_amount_as_sol(
                            &supported_token.mint,
                            crate::utils::get_proportional_amount(
                                pool.get_supported_token(&supported_token.mint)
                                    .map(|t| t.locked_amount)
                                    .unwrap(),
                                pricing_service.get_token_amount_as_sol(
                                    &restaking_vault.receipt_token_mint,
                                    restaking_vault.receipt_token_operation_reserved_amount
                                        + restaking_vault.receipt_token_operation_receivable_amount,
                                )?,
                                pricing_service.get_token_amount_as_sol(
                                    &pool.normalized_token_mint,
                                    pool.normalized_token_supply_amount,
                                )?,
                            )?,
                        )?
                    } else {
                        pricing_service.get_token_amount_as_sol(
                            &restaking_vault.receipt_token_mint,
                            restaking_vault.receipt_token_operation_reserved_amount
                                + restaking_vault.receipt_token_operation_receivable_amount,
                        )?
                    }
                };
            }

            extra_unrestaking_strategy_participant.allocated_amount =
                extra_unrestaking_strategy_participant
                    .allocated_amount
                    .saturating_sub(unrestaking_obligated_amounts_as_sol[supported_token_index]);
        }

        // first, allocate extra unrestaking amount for each tokens
        extra_unrestaking_strategy.cut_greedy(extra_unrestaking_obligated_amount_as_sol)?;

        // now allocate extra unrestaking + own unrestaking amount to related restaking vaults for each tokens
        let mut items =
            Vec::<UnrestakeVSTCommandItem>::with_capacity(FUND_ACCOUNT_MAX_RESTAKING_VAULTS);
        for restaking_vault in fund_account.get_restaking_vaults_iter() {
            items.push(UnrestakeVSTCommandItem {
                vault: restaking_vault.vault,
                receipt_token_mint: restaking_vault.receipt_token_mint,
                supported_token_mint: restaking_vault.supported_token_mint,
                allocated_receipt_token_amount: 0,
            });
        }

        for (supported_token_index, supported_token) in
            fund_account.get_supported_tokens_iter().enumerate()
        {
            let extra_unrestaking_obligated_amount_as_sol = extra_unrestaking_strategy
                .get_participant_last_cut_amount_by_index(supported_token_index)?;
            let unrestaking_obligated_amounts_as_sol =
                unrestaking_obligated_amounts_as_sol[supported_token_index];

            // iterator for (restaking_vault_index, restaking_vault, is_normalized_token_vault)
            let unrestakable_vaults_iter = fund_account
                .get_restaking_vaults_iter()
                .enumerate()
                .filter_map(|(index, restaking_vault)| {
                    if restaking_vault.supported_token_mint == supported_token.mint {
                        Some((index, restaking_vault, false))
                    } else if normalized_token_pool_account.is_some_and(|pool| {
                        pool.normalized_token_mint == restaking_vault.supported_token_mint
                            && pool.has_supported_token(&supported_token.mint)
                    }) {
                        Some((index, restaking_vault, true))
                    } else {
                        None
                    }
                });

            // here, it allocates unrestaking amount for each vault (of same supported token)
            // it assumes there won't be no more than two duplicate vaults using the same supported token.
            let mut unrestaking_strategy_vault_indexes: [usize; 4] = [0; 4];
            let mut unrestaking_strategy = WeightedAllocationStrategy::<4>::new(
                unrestakable_vaults_iter
                    .take(4)
                    .enumerate()
                    .map(
                        |(
                            index,
                            (restaking_vault_index, restaking_vault, is_normalized_token_vault),
                        )| {
                            // create strategy participant
                            unrestaking_strategy_vault_indexes[index] = restaking_vault_index;
                            Ok(WeightedAllocationParticipant::new(
                                restaking_vault.sol_allocation_weight,
                                if is_normalized_token_vault {
                                    // calculate supported token amount in normalized token pool proportionally
                                    let pool = normalized_token_pool_account.unwrap();
                                    pricing_service.get_token_amount_as_sol(
                                        &supported_token.mint,
                                        crate::utils::get_proportional_amount(
                                            pool.get_supported_token(&supported_token.mint)
                                                .map(|t| t.locked_amount)
                                                .unwrap(),
                                            pricing_service.get_token_amount_as_sol(
                                                &restaking_vault.receipt_token_mint,
                                                restaking_vault
                                                    .receipt_token_operation_reserved_amount
                                                    + restaking_vault
                                                        .receipt_token_operation_receivable_amount,
                                            )?,
                                            pricing_service.get_token_amount_as_sol(
                                                &pool.normalized_token_mint,
                                                pool.normalized_token_supply_amount,
                                            )?,
                                        )?,
                                    )?
                                } else {
                                    pricing_service.get_token_amount_as_sol(
                                        &restaking_vault.receipt_token_mint,
                                        restaking_vault.receipt_token_operation_reserved_amount
                                            + restaking_vault
                                                .receipt_token_operation_receivable_amount,
                                    )?
                                },
                                restaking_vault.sol_allocation_capacity_amount,
                            ))
                        },
                    )
                    .collect::<Result<Vec<_>>>()?,
            );

            unrestaking_strategy.cut_greedy(
                (extra_unrestaking_obligated_amount_as_sol + unrestaking_obligated_amounts_as_sol)
                    .saturating_sub(fund_account.sol.operation_receivable_amount),
            )?;

            for (p_index, p) in unrestaking_strategy.get_participants_iter().enumerate() {
                let item = &mut items[unrestaking_strategy_vault_indexes[p_index]];
                item.allocated_receipt_token_amount += pricing_service.get_sol_amount_as_token(
                    &item.receipt_token_mint,
                    // try to withdraw extra lamports to compensate for flooring errors for each vault & token
                    p.get_last_cut_amount()? + 1,
                )?;
            }
        }

        // reflect already unrestaking amounts
        for (restaking_vault_index, restaking_vault) in
            fund_account.get_restaking_vaults_iter().enumerate()
        {
            let item = &mut items[restaking_vault_index];
            item.allocated_receipt_token_amount = item
                .allocated_receipt_token_amount
                .saturating_sub(restaking_vault.receipt_token_operation_receivable_amount)
                .min(restaking_vault.receipt_token_operation_reserved_amount);
            if pricing_service.get_token_amount_as_sol(
                &item.receipt_token_mint,
                item.allocated_receipt_token_amount,
            )? < RESTAKING_MINIMUM_WITHDRAWAL_LAMPORTS
            {
                item.allocated_receipt_token_amount = 0;
            }
        }
        drop(fund_account);

        FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
            .update_asset_values(&mut pricing_service, true)?;

        let items = items
            .iter()
            .filter(|item| item.allocated_receipt_token_amount > 0)
            .cloned()
            .peekable();

        Ok((None, self.create_prepare_command_with_items(ctx, items)?))
    }

    fn create_prepare_command_with_items<'info>(
        &self,
        ctx: &OperationCommandContext,
        mut items: Peekable<impl Iterator<Item = UnrestakeVSTCommandItem>>,
    ) -> Result<Option<OperationCommandEntry>> {
        Ok(if let Some(item) = items.peek() {
            Some(
                match ctx
                    .fund_account
                    .load()?
                    .get_restaking_vault(&item.vault)?
                    .receipt_token_pricing_source
                    .try_deserialize()?
                {
                    Some(TokenPricingSource::JitoRestakingVault { address }) => {
                        UnrestakeVRTCommand {
                            state: UnrestakeVRTCommandState::Prepare {
                                items: items.collect(),
                            },
                        }
                        .with_required_accounts(
                            JitoRestakingVaultService::find_accounts_to_new(address)?,
                        )
                    }
                    Some(TokenPricingSource::SolvBTCVault { address }) => UnrestakeVRTCommand {
                        state: UnrestakeVRTCommandState::Prepare {
                            items: items.collect(),
                        },
                    }
                    .with_required_accounts(SolvBTCVaultService::find_accounts_to_new(address)?),
                    Some(TokenPricingSource::VirtualVault { .. }) => {
                        // no unrestaking on virtual vault
                        let _ = items.next();
                        return self.create_prepare_command_with_items(ctx, items);
                    }
                    // otherwise fails
                    Some(TokenPricingSource::SPLStakePool { .. })
                    | Some(TokenPricingSource::MarinadeStakePool { .. })
                    | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
                    | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. })
                    | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                    | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                    | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                    | Some(TokenPricingSource::PeggedToken { .. })
                    | None => {
                        err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                    }
                    #[cfg(all(test, not(feature = "idl-build")))]
                    Some(TokenPricingSource::Mock { .. }) => {
                        err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                    }
                },
            )
        } else {
            None
        })
    }

    fn execute_prepare<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: &[UnrestakeVSTCommandItem],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if items.is_empty() {
            return Ok((None, None));
        }
        let item = &items[0];
        let fund_account = ctx.fund_account.load()?;
        let restaking_vault = fund_account.get_restaking_vault(&item.vault)?;

        match restaking_vault
            .receipt_token_pricing_source
            .try_deserialize()?
        {
            Some(TokenPricingSource::JitoRestakingVault { address }) => {
                let [vault_program, vault_config, vault_account, ..] = accounts else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(address, vault_account.key());

                let vault_service =
                    JitoRestakingVaultService::new(vault_program, vault_config, vault_account)?;
                let required_accounts = vault_service
                    .find_accounts_to_request_withdraw()?
                    .chain([
                        (
                            fund_account.find_vault_receipt_token_reserve_account_address(
                                vault_account.key,
                            )?,
                            true,
                        ),
                        (fund_account.get_reserve_account_address()?, true),
                    ])
                    .chain(
                        (0..5)
                            .map(|index| {
                                let ticket_base_account =
                                    *FundAccount::find_unrestaking_ticket_account_address(
                                        &ctx.fund_account.key(),
                                        &item.vault,
                                        index,
                                    );
                                let ticket_account = vault_service
                                    .find_withdrawal_ticket_account(&ticket_base_account);
                                let ticket_receipt_token_account =
                                    associated_token::get_associated_token_address_with_program_id(
                                        &ticket_account,
                                        &item.receipt_token_mint,
                                        &anchor_spl::token::ID,
                                    );
                                [
                                    (ticket_account, true),
                                    (ticket_receipt_token_account, true),
                                    (ticket_base_account, false),
                                ]
                            })
                            .flatten(),
                    );

                Ok((
                    None,
                    Some(
                        UnrestakeVRTCommand {
                            state: UnrestakeVRTCommandState::Execute {
                                items: items.to_vec(),
                            },
                        }
                        .with_required_accounts(required_accounts),
                    ),
                ))
            }
            Some(TokenPricingSource::VirtualVault { .. }) => Ok((
                None,
                self.create_prepare_command_with_items(ctx, items[1..].iter().cloned().peekable())?,
            )),
            Some(TokenPricingSource::SolvBTCVault { address }) => {
                let [vault_program, vault_account, ..] = accounts else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(address, vault_account.key());

                let vault_service = SolvBTCVaultService::new(vault_program, vault_account)?;
                let required_accounts =
                    vault_service.find_accounts_to_request_withdrawal()?.chain([
                        (
                            fund_account
                                .find_vault_supported_token_reserve_account_address(&address)?,
                            true,
                        ),
                        (
                            fund_account.find_vault_receipt_token_reserve_account_address(
                                &restaking_vault.vault,
                            )?,
                            true,
                        ),
                        (fund_account.get_reserve_account_address()?, false),
                    ]);

                Ok((
                    None,
                    Some(
                        UnrestakeVRTCommand {
                            state: UnrestakeVRTCommandState::Execute {
                                items: items.to_vec(),
                            },
                        }
                        .with_required_accounts(required_accounts),
                    ),
                ))
            }
            // invalid configuration
            Some(TokenPricingSource::SPLStakePool { .. })
            | Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
            | None => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        }
    }

    fn execute_execute<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: &[UnrestakeVSTCommandItem],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if items.is_empty() {
            return Ok((None, None));
        }

        let pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
            .new_pricing_service(accounts.iter().copied(), false)?;

        let item = &items[0];
        let fund_account = ctx.fund_account.load()?;
        let restaking_vault = fund_account.get_restaking_vault(&item.vault)?;
        let receipt_token_pricing_source = restaking_vault
            .receipt_token_pricing_source
            .try_deserialize()?;

        drop(fund_account);

        let result = (|| match receipt_token_pricing_source {
            Some(TokenPricingSource::JitoRestakingVault { address }) => {
                let [vault_program, vault_config, vault_account, token_program, associated_token, system_program, vault_receipt_token_mint, fund_vault_receipt_token_reserve_account, fund_reserve_account, remaining_accounts @ ..] =
                    accounts
                else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(address, vault_account.key());
                let withdrawal_ticket_candidate_accounts = {
                    if remaining_accounts.len() < 15 {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    }
                    &remaining_accounts[..15]
                };
                let withdrawal_ticket_accounts = (0..5).find_map(|i| {
                    let ticket_account = withdrawal_ticket_candidate_accounts[i * 3];
                    let ticket_receipt_token_account =
                        withdrawal_ticket_candidate_accounts[i * 3 + 1];
                    let ticket_base_account = withdrawal_ticket_candidate_accounts[i * 3 + 2];
                    if !ticket_account.is_initialized() {
                        Some((
                            i,
                            ticket_account,
                            ticket_receipt_token_account,
                            ticket_base_account,
                        ))
                    } else {
                        None
                    }
                });

                if let Some((
                    withdrawal_ticket_index,
                    withdrawal_ticket_account,
                    withdrawal_ticket_receipt_token_account,
                    withdrawal_ticket_base_account,
                )) = withdrawal_ticket_accounts
                {
                    let vault_service =
                        JitoRestakingVaultService::new(vault_program, vault_config, vault_account)?;

                    let mut fund_account = ctx.fund_account.load_mut()?;
                    let restaking_vault = fund_account.get_restaking_vault_mut(&item.vault)?;

                    let (supported_token_amount_numerator, receipt_token_amount_denominator) =
                        vault_service.get_supported_token_to_receipt_token_exchange_ratio()?;
                    restaking_vault.update_supported_token_to_receipt_token_exchange_ratio(
                        supported_token_amount_numerator,
                        receipt_token_amount_denominator,
                    )?;

                    drop(fund_account);

                    let fund_account = ctx.fund_account.load()?;

                    let (
                        from_vault_receipt_token_account_amount,
                        enqueued_vault_receipt_token_amount,
                    ) = vault_service.request_withdraw(
                        token_program,
                        associated_token,
                        system_program,
                        vault_receipt_token_mint,
                        fund_vault_receipt_token_reserve_account,
                        withdrawal_ticket_account,
                        withdrawal_ticket_receipt_token_account,
                        withdrawal_ticket_base_account,
                        ctx.operator,
                        &[],
                        fund_reserve_account,
                        &[
                            &fund_account.get_reserve_account_seeds(),
                            &FundAccount::find_unrestaking_ticket_account_address(
                                &ctx.fund_account.key(),
                                &item.vault,
                                withdrawal_ticket_index as u8,
                            )
                            .get_seeds(),
                        ],
                        item.allocated_receipt_token_amount,
                    )?;

                    require_gte!(
                        fund_reserve_account.lamports(),
                        fund_account.sol.get_total_reserved_amount()
                    );
                    drop(fund_account);

                    let mut fund_account = ctx.fund_account.load_mut()?;
                    let restaking_vault = fund_account.get_restaking_vault_mut(&item.vault)?;
                    restaking_vault.receipt_token_operation_reserved_amount -=
                        enqueued_vault_receipt_token_amount;
                    restaking_vault.receipt_token_operation_receivable_amount +=
                        enqueued_vault_receipt_token_amount;
                    require_gte!(
                        from_vault_receipt_token_account_amount,
                        restaking_vault.receipt_token_operation_reserved_amount
                    );

                    Ok(Some(
                        UnrestakeVRTCommandResult {
                            vault: item.vault,
                            token_mint: item.receipt_token_mint,
                            unrestaking_token_amount: enqueued_vault_receipt_token_amount,
                            total_unrestaking_token_amount: restaking_vault
                                .receipt_token_operation_receivable_amount,
                            operation_reserved_token_amount: restaking_vault
                                .receipt_token_operation_reserved_amount,
                        }
                        .into(),
                    ))
                } else {
                    Ok(None)
                }
            }
            Some(TokenPricingSource::VirtualVault { .. }) => Ok(None),
            Some(TokenPricingSource::SolvBTCVault { address }) => {
                let [vault_program, vault_account, vault_receipt_token_mint, vault_supported_token_mint, vault_vault_supported_token_account, token_program, event_authority, fund_vault_supported_token_account, fund_vault_receipt_token_account, fund_reserve, ..] =
                    accounts
                else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(address, vault_account.key());

                let vault_service = SolvBTCVaultService::new(vault_program, vault_account)?;

                let mut fund_account = ctx.fund_account.load_mut()?;
                let restaking_vault = fund_account.get_restaking_vault_mut(&item.vault)?;

                let (supported_token_amount_numerator, receipt_token_amount_denominator) =
                    vault_service.get_supported_token_to_receipt_token_exchange_ratio()?;
                restaking_vault.update_supported_token_to_receipt_token_exchange_ratio(
                    supported_token_amount_numerator,
                    receipt_token_amount_denominator,
                )?;

                drop(fund_account);

                let fund_account = ctx.fund_account.load()?;

                let (
                    fund_vault_receipt_token_account_amount,
                    enqueued_vault_receipt_token_amount,
                    expected_supported_token_account_amount,
                    total_unrestaking_vault_receipt_token_amount,
                ) = vault_service.request_withdrawal(
                    vault_receipt_token_mint,
                    vault_supported_token_mint,
                    vault_vault_supported_token_account,
                    token_program,
                    event_authority,
                    ctx.fund_account.as_ref(),
                    &[&fund_account.get_seeds()],
                    fund_vault_receipt_token_account,
                    fund_vault_supported_token_account,
                    fund_reserve,
                    &[&fund_account.get_reserve_account_seeds()],
                    item.allocated_receipt_token_amount,
                )?;

                if enqueued_vault_receipt_token_amount > 0 {
                    drop(fund_account);
                    let mut fund_account = ctx.fund_account.load_mut()?;

                    let supported_token =
                        fund_account.get_supported_token_mut(&item.supported_token_mint)?;
                    supported_token.token.operation_receivable_amount +=
                        expected_supported_token_account_amount;

                    require_gte!(
                        expected_supported_token_account_amount,
                        pricing_service.get_token_amount_as_token(
                            vault_receipt_token_mint.key,
                            enqueued_vault_receipt_token_amount,
                            vault_supported_token_mint.key
                        )?,
                    );

                    let restaking_vault = fund_account.get_restaking_vault_mut(&item.vault)?;
                    restaking_vault.receipt_token_operation_reserved_amount -=
                        enqueued_vault_receipt_token_amount;

                    require_gte!(
                        fund_vault_receipt_token_account_amount,
                        restaking_vault.receipt_token_operation_reserved_amount
                    );

                    Ok(Some(
                        UnrestakeVRTCommandResult {
                            vault: item.vault,
                            token_mint: item.receipt_token_mint,
                            unrestaking_token_amount: enqueued_vault_receipt_token_amount,
                            total_unrestaking_token_amount:
                                total_unrestaking_vault_receipt_token_amount,
                            operation_reserved_token_amount: restaking_vault
                                .receipt_token_operation_reserved_amount,
                        }
                        .into(),
                    ))
                } else {
                    Ok(None)
                }
            }
            // invalid configuration
            Some(TokenPricingSource::SPLStakePool { .. })
            | Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
            | None => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        })()?;

        Ok((
            result,
            self.create_prepare_command_with_items(ctx, items[1..].iter().cloned().peekable())?,
        ))
    }
}
