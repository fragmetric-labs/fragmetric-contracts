use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::errors;
use crate::modules::normalization::{NormalizedTokenPoolAccount, NormalizedTokenPoolService};
use crate::modules::pricing::TokenPricingSource;
use crate::modules::staking;
use crate::utils::{AccountInfoExt, PDASeeds};

use super::{
    FundService, HarvestRewardCommand, NormalizedToken, OperationCommand, OperationCommandContext,
    OperationCommandEntry, OperationCommandResult, RestakeVSTCommandState, SelfExecutable,
    StakeSOLCommand, StakeSOLCommandItem, StakeSOLCommandResult, StakeSOLCommandState,
    SupportedToken, WeightedAllocationParticipant, WeightedAllocationStrategy,
    FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct NormalizeSTCommand {
    state: NormalizeSTCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct NormalizeSTCommandItem {
    supported_token_mint: Pubkey,
    allocated_token_amount: u64,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
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

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct NormalizeSTCommandResult {
    pub supported_token_mint: Pubkey,
    pub normalized_supported_token_amount: u64,
    pub minted_token_amount: u64,
    pub operation_reserved_token_amount: u64,
}

impl SelfExecutable for NormalizeSTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let mut remaining_items: Option<Vec<NormalizeSTCommandItem>> = None;
        let mut result: Option<OperationCommandResult> = None;

        match &self.state {
            NormalizeSTCommandState::New => {
                let pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                    .new_pricing_service(accounts.into_iter().cloned())?;
                let fund_account = ctx.fund_account.load()?;

                if let Some(normalized_token) = fund_account.get_normalized_token() {
                    let supported_tokens = fund_account.get_supported_tokens_iter();
                    let restaking_vaults =
                        fund_account.get_restaking_vaults_iter().collect::<Vec<_>>();

                    // can ensure given [accounts] have the account as pricing_service successfully created above
                    let normalized_token_pool_account_info = fund_account
                        .get_normalized_token_pool_address()
                        .map(|address| {
                            accounts
                                .iter()
                                .find(|account| account.key() == address)
                                .copied()
                        })
                        .flatten()
                        .ok_or_else(|| {
                            error!(errors::ErrorCode::FundOperationCommandExecutionFailedException)
                        })?;
                    let normalized_token_pool_account =
                        NormalizedTokenPoolService::deserialize_pool_account(
                            normalized_token_pool_account_info,
                        )?;

                    // find supported tokens which is restakable and normalizable. (token_mint, token_amount)
                    let mut target_supported_token_and_amounts =
                        Vec::<(&Pubkey, u64)>::with_capacity(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS);
                    for supported_token in supported_tokens {
                        for restaking_vault in restaking_vaults.iter() {
                            if restaking_vault.supported_token_mint == supported_token.mint
                                || restaking_vault.supported_token_mint == normalized_token.mint
                                    && normalized_token_pool_account
                                        .has_supported_token(&supported_token.mint)
                            {
                                let asset_net_operation_reserved_amount = fund_account
                                    .get_asset_net_operation_reserved_amount(
                                        Some(supported_token.mint),
                                        &pricing_service,
                                    )?;
                                if asset_net_operation_reserved_amount > 0 {
                                    target_supported_token_and_amounts.push((
                                        &supported_token.mint,
                                        u64::try_from(asset_net_operation_reserved_amount)?,
                                    ))
                                }
                                break;
                            }
                        }
                    }

                    // for each found supported tokens, apply allocation strategy to relevant restaking vaults to calculate the share of normalized token allocation.
                    let mut items = Vec::<NormalizeSTCommandItem>::with_capacity(
                        FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
                    );
                    for (supported_token_mint, supported_token_amount) in
                        target_supported_token_and_amounts
                    {
                        let restakable_vaults = restaking_vaults
                            .iter()
                            .filter(|restaking_vault| {
                                restaking_vault.supported_token_mint == *supported_token_mint
                                    || restaking_vault.supported_token_mint == normalized_token.mint
                            })
                            .copied()
                            .collect::<Vec<_>>();
                        if !restakable_vaults.iter().any(|restaking_vault| {
                            restaking_vault.supported_token_mint == normalized_token.mint
                        }) {
                            // no need to calculate
                            continue;
                        }

                        let mut strategy = WeightedAllocationStrategy::<
                            FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
                        >::new(
                            restakable_vaults
                                .iter()
                                .map(|restaking_vault| {
                                    Ok(WeightedAllocationParticipant::new(
                                        restaking_vault.sol_allocation_weight,
                                        pricing_service.get_token_amount_as_sol(
                                            &restaking_vault.receipt_token_mint,
                                            // TODO: from fund asset...? not available
                                            restaking_vault.receipt_token_operation_reserved_amount,
                                        )?,
                                        restaking_vault.sol_allocation_capacity_amount,
                                    ))
                                })
                                .collect::<Result<Vec<_>>>()?,
                        );
                        strategy.put(pricing_service.get_token_amount_as_sol(
                            supported_token_mint,
                            supported_token_amount,
                        )?)?;

                        let mut allocated_sol_amount_for_normalized_token_vaults = 0;
                        for (index, restakable_vault) in restakable_vaults.iter().enumerate() {
                            if restakable_vault.supported_token_mint == normalized_token.mint {
                                allocated_sol_amount_for_normalized_token_vaults +=
                                    strategy.get_participant_last_put_amount_by_index(index)?;
                            }
                        }
                        if allocated_sol_amount_for_normalized_token_vaults > 0 {
                            items.push(NormalizeSTCommandItem {
                                supported_token_mint: *supported_token_mint,
                                allocated_token_amount: pricing_service.get_sol_amount_as_token(
                                    supported_token_mint,
                                    allocated_sol_amount_for_normalized_token_vaults,
                                )?,
                            })
                        }
                    }
                    if items.len() > 0 {
                        remaining_items = Some(items);
                    }
                }
            }
            NormalizeSTCommandState::Prepare { items } => {
                if let Some(item) = items.first() {
                    let fund_account = ctx.fund_account.load()?;
                    let supported_token =
                        fund_account.get_supported_token(&item.supported_token_mint)?;
                    let normalized_token_pool_account_info = fund_account
                        .get_normalized_token_pool_address()
                        .map(|address| {
                            accounts
                                .iter()
                                .find(|account| account.key() == address)
                                .copied()
                        })
                        .flatten()
                        .ok_or_else(|| {
                            error!(errors::ErrorCode::FundOperationCommandExecutionFailedException)
                        })?;

                    let mut required_accounts =
                        NormalizedTokenPoolService::find_accounts_to_normalize_supported_token(
                            normalized_token_pool_account_info,
                            &supported_token.mint,
                            &supported_token.program,
                        )?;
                    required_accounts.extend(vec![
                        (
                            fund_account.find_normalized_token_reserve_account_address()?,
                            true,
                        ),
                        (
                            fund_account.find_supported_token_reserve_account_address(
                                &supported_token.mint,
                            )?,
                            true,
                        ),
                    ]);

                    return Ok((
                        None,
                        Some(
                            NormalizeSTCommand {
                                state: NormalizeSTCommandState::Execute {
                                    items: items.clone(),
                                },
                            }
                            .with_required_accounts(required_accounts),
                        ),
                    ));
                }
            }
            NormalizeSTCommandState::Execute { items } => {
                if let Some(item) = items.first() {
                    remaining_items = Some(items.into_iter().skip(1).copied().collect::<Vec<_>>());

                    let [normalized_token_pool_account, normalized_token_mint, normalized_token_program, supported_token_mint, supported_token_program, pool_supported_token_reserve_account, to_normalized_token_account, from_supported_token_account, remaining_accounts @ ..] =
                        accounts
                    else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    let normalized_token_mint_key = *normalized_token_mint.key;
                    let mut normalized_token_pool_account = normalized_token_pool_account
                        .parse_account_boxed::<NormalizedTokenPoolAccount>(
                    )?;
                    let mut normalized_token_mint =
                        normalized_token_mint.parse_interface_account_boxed::<Mint>()?;
                    let normalized_token_program =
                        normalized_token_program.parse_program_boxed::<Token>()?;
                    let supported_token_mint =
                        supported_token_mint.parse_interface_account_boxed::<Mint>()?;
                    let supported_token_program =
                        supported_token_program.parse_interface_boxed::<TokenInterface>()?;
                    let pool_supported_token_reserve_account = pool_supported_token_reserve_account
                        .parse_interface_account_boxed::<TokenAccount>()?;
                    let mut to_normalized_token_account = to_normalized_token_account
                        .parse_interface_account_boxed::<TokenAccount>(
                    )?;
                    let from_supported_token_account = from_supported_token_account
                        .parse_interface_account_boxed::<TokenAccount>()?;

                    let mut pricing_service =
                        FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                            .new_pricing_service(accounts.into_iter().cloned())?;

                    let mut normalized_token_pool_service = NormalizedTokenPoolService::new(
                        &mut *normalized_token_pool_account,
                        &mut *normalized_token_mint,
                        &normalized_token_program,
                    )?;

                    let expected_minted_normalized_token_amount = pricing_service
                        .get_sol_amount_as_token(
                            &normalized_token_mint_key,
                            pricing_service.get_token_amount_as_sol(
                                &item.supported_token_mint,
                                item.allocated_token_amount,
                            )?,
                        )?;

                    let (to_normalized_token_account_amount, minted_normalized_token_amount) =
                        normalized_token_pool_service.normalize_supported_token(
                            &supported_token_mint,
                            &supported_token_program,
                            &pool_supported_token_reserve_account,
                            &mut to_normalized_token_account,
                            &from_supported_token_account,
                            &ctx.fund_account.to_account_info(),
                            &[ctx.fund_account.load()?.get_seeds().as_ref()],
                            item.allocated_token_amount,
                            &mut pricing_service,
                        )?;

                    let mut fund_account = ctx.fund_account.load_mut()?;

                    let supported_token =
                        fund_account.get_supported_token_mut(&item.supported_token_mint)?;
                    supported_token.token.operation_reserved_amount -= item.allocated_token_amount;

                    let normalized_token = fund_account.get_normalized_token_mut().unwrap();
                    normalized_token.operation_reserved_amount += minted_normalized_token_amount;

                    require_gte!(
                        minted_normalized_token_amount,
                        expected_minted_normalized_token_amount
                    );
                    require_gte!(
                        to_normalized_token_account_amount,
                        normalized_token.get_total_reserved_amount()
                    );

                    result = Some(
                        NormalizeSTCommandResult {
                            supported_token_mint: item.supported_token_mint,
                            normalized_supported_token_amount: item.allocated_token_amount,
                            minted_token_amount: minted_normalized_token_amount,
                            operation_reserved_token_amount: normalized_token
                                .operation_reserved_amount,
                        }
                        .into(),
                    );
                }
            }
        }

        // transition to next command
        Ok((
            result,
            match remaining_items {
                Some(remaining_items) if remaining_items.len() > 0 => {
                    // practically, specific accounts are not required for the prepare state command. so just run it and proceed to the next command.
                    NormalizeSTCommand {
                        state: NormalizeSTCommandState::Prepare {
                            items: remaining_items,
                        },
                    }
                    .execute(ctx, accounts)?
                    .1
                }
                _ => None,
            }
            .or_else(|| Some(HarvestRewardCommand::default().without_required_accounts())),
        ))
    }
}
