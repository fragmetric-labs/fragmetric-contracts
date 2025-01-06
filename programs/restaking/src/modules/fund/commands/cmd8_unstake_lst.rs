use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::staking::*;
use crate::utils::{AccountInfoExt, AsAccountInfo, PDASeeds};

use super::{
    EnqueueWithdrawalBatchCommand, FundAccount, FundService, OperationCommandContext,
    OperationCommandEntry, OperationCommandResult, SelfExecutable,
    FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct UnstakeLSTCommand {
    #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
    items: Vec<UnstakeLSTCommandItem>,
    state: UnstakeLSTCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct UnstakeLSTCommandItem {
    token_mint: Pubkey,
    allocated_token_amount: u64,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum UnstakeLSTCommandState {
    Init, // TODO v0.4/operation: change to `New` and allocate token amount
    Prepare,
    GetAvailableWithdrawals,
    Execute {
        #[max_len(5)]
        withdraw_stake_items: Vec<WithdrawStakeItem>,
    },
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct WithdrawStakeItem {
    validator_stake_account: Pubkey,
    fund_stake_account: Pubkey,
    fund_stake_account_index: u8,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct UnstakeLSTCommandResult {
    pub token_mint: Pubkey,
    pub burnt_token_amount: u64,
    pub deducted_sol_fee_amount: u64,
    pub unstaking_sol_amount: u64,
    pub unstaked_sol_amount: u64,
    pub operation_reserved_sol_amount: u64,
    pub operation_receivable_sol_amount: u64,
}

impl SelfExecutable for UnstakeLSTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let (result, entry) = match &self.state {
            UnstakeLSTCommandState::Init => self.execute_init(ctx, accounts, &self.items)?,
            UnstakeLSTCommandState::Prepare => {
                self.execute_prepare(ctx, accounts, self.items.clone(), None)?
            }
            UnstakeLSTCommandState::GetAvailableWithdrawals => {
                self.execute_get_available_withdrawals(ctx, accounts, &self.items)?
            }
            UnstakeLSTCommandState::Execute {
                withdraw_stake_items,
            } => self.execute_execute(ctx, accounts, &self.items, withdraw_stake_items)?,
        };

        // TODO v0.4/operation: next step ... stake sol
        Ok((
            result,
            entry.or_else(|| {
                Some(EnqueueWithdrawalBatchCommand::default().without_required_accounts())
            }),
        ))
    }
}

impl UnstakeLSTCommand {
    #[inline(never)]
    fn execute_init<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: &[UnstakeLSTCommandItem],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        // TODO v0.4/operation: allocate token amounts

        // prepare state does not require additional accounts,
        // so we can execute directly.
        self.execute_prepare(ctx, accounts, items.to_vec(), None)
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
        let fund_account = ctx.fund_account.load()?;
        let pricing_source = fund_account
            .get_supported_token(&item.token_mint)?
            .pricing_source
            .try_deserialize()?;
        let pool_account = match pricing_source {
            Some(TokenPricingSource::SPLStakePool { address })
            | Some(TokenPricingSource::MarinadeStakePool { address })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address }) => *accounts
                .iter()
                .find(|account| account.key() == address)
                .ok_or_else(|| error!(ErrorCode::FundOperationCommandExecutionFailedException))?,
            _ => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
        };

        let entry = match pricing_source {
            Some(TokenPricingSource::SPLStakePool { .. }) => {
                let accounts_to_get_validator_stake_accounts =
                    <SPLStakePoolService>::find_accounts_to_get_validator_stake_accounts(
                        pool_account,
                    )?;
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

                let required_accounts =
                    accounts_to_get_validator_stake_accounts.chain(fund_stake_accounts);

                Self {
                    items,
                    state: UnstakeLSTCommandState::GetAvailableWithdrawals,
                }
                .with_required_accounts(required_accounts)
            }
            Some(TokenPricingSource::MarinadeStakePool { .. }) => {
                let fund_supported_token_reserve_account =
                    fund_account.find_supported_token_reserve_account_address(&item.token_mint)?;
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

                let required_accounts = [(fund_supported_token_reserve_account, true)]
                    .into_iter()
                    .chain(accounts_to_order_unstake)
                    .chain(withdrawal_ticket_accounts);

                Self {
                    state: UnstakeLSTCommandState::Execute {
                        withdraw_stake_items: vec![],
                    },
                    items,
                }
                .with_required_accounts(required_accounts)
            }
            Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. }) => {
                let accounts_to_get_validator_stake_accounts = SanctumSingleValidatorSPLStakePoolService::find_accounts_to_get_validator_stake_accounts(pool_account)?;
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

                let required_accounts =
                    accounts_to_get_validator_stake_accounts.chain(fund_stake_accounts);

                Self {
                    items,
                    state: UnstakeLSTCommandState::GetAvailableWithdrawals,
                }
                .with_required_accounts(required_accounts)
            }
            _ => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
        };

        Ok((previous_execution_result, Some(entry)))
    }

    #[inline(never)]
    fn execute_get_available_withdrawals<'info>(
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
                .spl_stake_pool_get_validator_stake_accounts::<SPLStakePool>(
                    ctx, accounts, items, item, address,
                ),
            Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address }) => {
                self.spl_stake_pool_get_validator_stake_accounts::<SanctumSingleValidatorSPLStakePool>(
                    ctx, accounts, items, item, address,
                )
            }
            _ => err!(ErrorCode::FundOperationCommandExecutionFailedException),
        }?;

        Ok((None, Some(entry)))
    }

    fn spl_stake_pool_get_validator_stake_accounts<'info, T: SPLStakePoolInterface>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: &[UnstakeLSTCommandItem],
        current_item: &UnstakeLSTCommandItem,
        pool_account_address: Pubkey,
    ) -> Result<OperationCommandEntry> {
        let [pool_program, pool_account, pool_token_mint, pool_token_program, reserve_stake_account, validator_list_account, remaining_accounts @ ..] =
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

        let available_fund_stake_accounts = fund_stake_accounts
            .iter()
            .enumerate()
            .filter_map(|(index, fund_stake_account)| {
                (!fund_stake_account.is_initialized()).then_some(index)
            })
            .collect::<Vec<_>>();

        // Maximum number of validators = # of available(uninitialized) fund stake accounts
        let num_validators = available_fund_stake_accounts.len();
        let validator_stake_accounts = spl_stake_pool_service
            .get_validator_stake_accounts(validator_list_account, num_validators)?;
        // Actual # of validators
        let num_validators = validator_stake_accounts.len();

        let withdraw_stake_items = available_fund_stake_accounts
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
        let fund_stake_accounts = available_fund_stake_accounts
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
            items: items.to_vec(),
            state: UnstakeLSTCommandState::Execute {
                withdraw_stake_items,
            },
        }
        .with_required_accounts(required_accounts))
    }
}

// #[inline(never)]
// fn execute_unstake<'info>(
//     &self,
//     ctx: &mut OperationCommandContext<'info, '_>,
//     accounts: &[&'info AccountInfo<'info>],
//     items: &[UnstakeLSTCommandItem],
// ) -> Result<(
//     Option<OperationCommandResult>,
//     Option<OperationCommandEntry>,
// )> {
//     if items.is_empty() {
//         return Ok((None, None));
//     }
//     let item = &items[0];

//     let token_pricing_source = ctx
//         .fund_account
//         .load()?
//         .get_supported_token(&item.token_mint)?
//         .pricing_source
//         .try_deserialize()?;

//     let result = match token_pricing_source {
//         Some(TokenPricingSource::SPLStakePool { address }) => {
//             self.spl_stake_pool_withdraw_sol::<SPLStakePool>(ctx, accounts, item, address)?
//         }
//         Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address }) => {
//             self.spl_stake_pool_withdraw_sol::<SanctumSingleValidatorSPLStakePool>(
//                 ctx, accounts, item, address,
//             )?
//         }
//         _ => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
//     };

//     // prepare state does not require additional accounts,
//     // so we can execute directly.
//     self.execute_prepare(ctx, accounts, items[1..].to_vec(), Some(result))
// }

// fn spl_stake_pool_withdraw_sol<'info, T: SPLStakePoolInterface>(
//     &self,
//     ctx: &mut OperationCommandContext<'info, '_>,
//     accounts: &[&'info AccountInfo<'info>],
//     item: &UnstakeLSTCommandItem,
//     pool_account_address: Pubkey,
// ) -> Result<OperationCommandResult> {
//     let &[fund_reserve_account, fund_supported_token_reserve_account, pool_program, pool_account, pool_token_mint, pool_token_program, withdraw_authority, reserve_stake_account, manager_fee_account, clock, stake_history, stake_program, ..] =
//         accounts
//     else {
//         err!(error::ErrorCode::AccountNotEnoughKeys)?
//     };
//     require_keys_eq!(pool_account_address, pool_account.key());
//     require_keys_eq!(item.token_mint, pool_token_mint.key());

//     let spl_stake_pool_service = SPLStakePoolService::<T>::new(
//         pool_program,
//         pool_account,
//         pool_token_mint,
//         pool_token_program,
//     )?;

//     let (unstaked_sol_amount, deducted_pool_token_fee_amount) = {
//         let fund_account = ctx.fund_account.load()?;
//         spl_stake_pool_service.withdraw_sol(
//             withdraw_authority,
//             reserve_stake_account,
//             manager_fee_account,
//             clock,
//             stake_history,
//             stake_program,
//             fund_reserve_account,
//             fund_supported_token_reserve_account,
//             ctx.fund_account.as_account_info(),
//             &[&fund_account.get_seeds()],
//             item.allocated_token_amount,
//         )?
//     };

//     // pricing service with updated token values
//     let pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
//         .new_pricing_service(accounts.iter().cloned())?;

//     // validation (expects diff <= 1)
//     let expected_pool_token_fee_amount = item.allocated_token_amount.saturating_sub(
//         pricing_service.get_sol_amount_as_token(pool_token_mint.key, unstaked_sol_amount)?,
//     );
//     require_gte!(
//         1,
//         expected_pool_token_fee_amount.abs_diff(deducted_pool_token_fee_amount),
//     );

//     // calculate deducted fee as SOL (will be added to SOL receivable)
//     let deducted_sol_fee_amount = pricing_service
//         .get_token_amount_as_sol(pool_token_mint.key, deducted_pool_token_fee_amount)?;

//     // Update fund account
//     let mut fund_account = ctx.fund_account.load_mut()?;
//     fund_account.sol.operation_reserved_amount += unstaked_sol_amount;
//     fund_account.sol.operation_receivable_amount += deducted_sol_fee_amount;

//     let supported_token = fund_account.get_supported_token_mut(pool_token_mint.key)?;
//     supported_token.token.operation_reserved_amount -= item.allocated_token_amount;

//     // validation
//     require_eq!(
//         fund_reserve_account.lamports(),
//         fund_account.sol.get_total_reserved_amount(),
//     );

//     Ok(UnstakeLSTCommandResult {
//         token_mint: item.token_mint,
//         burnt_token_amount: item.allocated_token_amount,
//         deducted_sol_fee_amount,
//         unstaking_sol_amount: 0,
//         unstaked_sol_amount,
//         operation_reserved_sol_amount: fund_account.sol.operation_reserved_amount,
//         operation_receivable_sol_amount: fund_account.sol.operation_receivable_amount,
//     }
//     .into())
// }

// #[inline(never)]
// fn execute_execute<'info>(
//     &self,
//     ctx: &mut OperationCommandContext<'info, '_>,
//     accounts: &[&'info AccountInfo<'info>],
//     items: &[UnstakeLSTCommandItem],
//     withdraw_stake_items: &[WithdrawStakeItem],
// ) -> Result<(
//     Option<OperationCommandResult>,
//     Option<OperationCommandEntry>,
// )> {
//     if items.is_empty() {
//         return Ok((None, None));
//     }
//     let item = &items[0];

//     let token_pricing_source = ctx
//         .fund_account
//         .load()?
//         .get_supported_token(&item.token_mint)?
//         .pricing_source
//         .try_deserialize()?;

//     let result = match token_pricing_source {
//         Some(TokenPricingSource::SPLStakePool { address }) => self
//             .spl_stake_pool_withdraw_stake::<SPLStakePool>(
//                 ctx,
//                 accounts,
//                 item,
//                 withdraw_stake_items,
//                 address,
//             )?,
//         Some(TokenPricingSource::MarinadeStakePool { address }) => {
//             self.marinade_stake_pool_order_unstake(ctx, accounts, item, address)?
//         }
//         Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address }) => {
//             self.spl_stake_pool_withdraw_stake::<SanctumSingleValidatorSPLStakePool>(
//                 ctx,
//                 accounts,
//                 item,
//                 withdraw_stake_items,
//                 address,
//             )?
//         }
//         _ => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
//     }
//     .map(
//         |(burnt_token_amount, unstaking_sol_amount, deducted_sol_fee_amount)| {
//             let mut fund_account = ctx.fund_account.load_mut()?;
//             fund_account.sol.operation_receivable_amount +=
//                 unstaking_sol_amount + deducted_sol_fee_amount;

//             let supported_token = fund_account.get_supported_token_mut(&item.token_mint)?;
//             supported_token.token.operation_reserved_amount -= burnt_token_amount;

//             Ok::<_, Error>(
//                 UnstakeLSTCommandResult {
//                     token_mint: item.token_mint,
//                     burnt_token_amount: item.allocated_token_amount,
//                     deducted_sol_fee_amount,
//                     unstaking_sol_amount,
//                     unstaked_sol_amount: 0,
//                     operation_reserved_sol_amount: fund_account.sol.operation_reserved_amount,
//                     operation_receivable_sol_amount: fund_account
//                         .sol
//                         .operation_receivable_amount,
//                 }
//                 .into(),
//             )
//         },
//     )
//     .transpose()?;

//     // prepare state does not require additional accounts,
//     // so we can execute directly.
//     self.execute_prepare(ctx, accounts, items[1..].to_vec(), result)
// }

// /// returns [burnt_token_amount, unstaking_sol_amount, deducted_sol_fee_amount]
// fn spl_stake_pool_withdraw_stake<'info, T: SPLStakePoolInterface>(
//     &self,
//     ctx: &mut OperationCommandContext<'info, '_>,
//     accounts: &[&'info AccountInfo<'info>],
//     command_item: &UnstakeLSTCommandItem,
//     withdraw_stake_items: &[WithdrawStakeItem],
//     pool_account_address: Pubkey,
// ) -> Result<Option<(u64, u64, u64)>> {
//     let [fund_supported_token_reserve_account, pool_program, pool_account, pool_token_mint, pool_token_program, withdraw_authority, manager_fee_account, validator_list_account, clock, stake_program, remaining_accounts @ ..] =
//         accounts
//     else {
//         err!(error::ErrorCode::AccountNotEnoughKeys)?
//     };
//     let (fund_stake_accounts, remaining_accounts) = {
//         if remaining_accounts.len() < 5 {
//             err!(error::ErrorCode::AccountNotEnoughKeys)?;
//         }
//         remaining_accounts.split_at(5)
//     };
//     let validator_stake_accounts = {
//         if remaining_accounts.len() < withdraw_stake_items.len() {
//             err!(error::ErrorCode::AccountNotEnoughKeys)?;
//         }
//         &remaining_accounts[..withdraw_stake_items.len()]
//     };

//     require_keys_eq!(pool_account_address, pool_account.key());
//     require_keys_eq!(command_item.token_mint, pool_token_mint.key());
//     for (index, fund_stake_account) in fund_stake_accounts.iter().enumerate() {
//         let fund_stake_account_address = *FundAccount::find_stake_account_address(
//             &ctx.fund_account.key(),
//             pool_account.key,
//             index as u8,
//         );
//         require_keys_eq!(fund_stake_account_address, fund_stake_account.key());
//     }
//     for (withdraw_stake_item, validator_stake_account) in
//         withdraw_stake_items.iter().zip(validator_stake_accounts)
//     {
//         require_keys_eq!(
//             withdraw_stake_item.validator_stake_account,
//             validator_stake_account.key(),
//         );
//     }

//     let spl_stake_pool_service = SPLStakePoolService::<T>::new(
//         pool_program,
//         pool_account,
//         pool_token_mint,
//         pool_token_program,
//     )?;

//     let mut total_burnt_token_amount = 0;
//     let mut total_unstaking_sol_amount = 0;
//     let mut total_deducted_pool_token_fee_amount = 0;
//     let mut withdraw_stake_count = 0;

//     let fund_stake_accounts_iter = fund_stake_accounts
//         .iter()
//         .enumerate()
//         // we can only use uninitialized stake account
//         .filter(|(_, fund_stake_account)| !fund_stake_account.is_initialized());
//     let validator_stake_accounts_iter = withdraw_stake_items
//         .iter()
//         .map(|item| item.token_amount)
//         .zip(validator_stake_accounts);

//     let fund_account = ctx.fund_account.load()?;

//     for ((index, &fund_stake_account), (unstaking_token_amount, &validator_stake_account)) in
//         fund_stake_accounts_iter.zip(validator_stake_accounts_iter)
//     {
//         let (unstaking_sol_amount, deducted_pool_token_fee_amount) = spl_stake_pool_service
//             .withdraw_stake(
//                 ctx.system_program,
//                 withdraw_authority,
//                 manager_fee_account,
//                 validator_list_account,
//                 clock,
//                 stake_program,
//                 validator_stake_account,
//                 fund_stake_account,
//                 &[&FundAccount::find_stake_account_address(
//                     &ctx.fund_account.key(),
//                     pool_account.key,
//                     index as u8,
//                 )
//                 .get_signer_seeds()],
//                 ctx.operator,
//                 ctx.fund_account.as_account_info(),
//                 &[&fund_account.get_seeds()],
//                 fund_supported_token_reserve_account,
//                 ctx.fund_account.as_account_info(),
//                 &[&fund_account.get_seeds()],
//                 unstaking_token_amount,
//             )?;

//         total_burnt_token_amount += unstaking_token_amount;
//         total_unstaking_sol_amount += unstaking_sol_amount;
//         total_deducted_pool_token_fee_amount += deducted_pool_token_fee_amount;
//         withdraw_stake_count += 1;
//     }

//     // withdraw stake did not happen
//     if total_burnt_token_amount == 0 {
//         return Ok(None);
//     }

//     drop(fund_account);

//     // pricing service with updated token values
//     let pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
//         .new_pricing_service(accounts.iter().cloned())?;

//     // validation (expects diff <= withdraw_stake_count)
//     let expected_pool_token_fee_amount = total_burnt_token_amount.saturating_sub(
//         pricing_service
//             .get_sol_amount_as_token(pool_token_mint.key, total_unstaking_sol_amount)?,
//     );
//     require_gte!(
//         withdraw_stake_count,
//         expected_pool_token_fee_amount.abs_diff(total_deducted_pool_token_fee_amount)
//     );

//     // calculate deducted fee as SOL (will be added to SOL receivable)
//     let total_deducted_sol_fee_amount = pricing_service
//         .get_token_amount_as_sol(pool_token_mint.key, total_deducted_pool_token_fee_amount)?;

//     Ok(Some((
//         total_burnt_token_amount,
//         total_unstaking_sol_amount,
//         total_deducted_sol_fee_amount,
//     )))
// }

// /// returns [burnt_token_amount, unstaking_sol_amount, deducted_sol_fee_amount]
// fn marinade_stake_pool_order_unstake<'info>(
//     &self,
//     ctx: &mut OperationCommandContext<'info, '_>,
//     accounts: &[&'info AccountInfo<'info>],
//     item: &UnstakeLSTCommandItem,
//     pool_account_address: Pubkey,
// ) -> Result<Option<(u64, u64, u64)>> {
//     let [fund_supported_token_reserve_account, pool_program, pool_account, pool_token_mint, pool_token_program, clock, rent, remaining_accounts @ ..] =
//         accounts
//     else {
//         err!(error::ErrorCode::AccountNotEnoughKeys)?
//     };
//     let withdrawal_ticket_accounts = {
//         if remaining_accounts.len() < 5 {
//             err!(error::ErrorCode::AccountNotEnoughKeys)?;
//         }
//         &remaining_accounts[..5]
//     };

//     require_keys_eq!(pool_account_address, pool_account.key());
//     require_keys_eq!(item.token_mint, pool_token_mint.key());
//     for (index, withdrawal_ticket_account) in withdrawal_ticket_accounts.iter().enumerate() {
//         let withdrawal_ticket_account_address =
//             *FundAccount::find_unstaking_ticket_account_address(
//                 &ctx.fund_account.key(),
//                 pool_account.key,
//                 index as u8,
//             );
//         require_keys_eq!(
//             withdrawal_ticket_account_address,
//             withdrawal_ticket_account.key()
//         );
//     }

//     let Some((withdrawal_ticket_account_index, withdrawal_ticket_account)) =
//         withdrawal_ticket_accounts
//             .iter()
//             .enumerate()
//             .find(|(_, account)| !account.is_initialized())
//     else {
//         // there is no available(uninitialized) withdrawal ticket account
//         return Ok(None);
//     };

//     let marinade_stake_pool_service = MarinadeStakePoolService::new(
//         pool_program,
//         pool_account,
//         pool_token_mint,
//         pool_token_program,
//     )?;

//     let (unstaking_sol_amount, deducted_sol_fee_amount) = {
//         let fund_account = ctx.fund_account.load()?;
//         marinade_stake_pool_service.order_unstake(
//             ctx.system_program,
//             clock,
//             rent,
//             withdrawal_ticket_account,
//             &[&FundAccount::find_unstaking_ticket_account_address(
//                 &ctx.fund_account.key(),
//                 pool_account.key,
//                 withdrawal_ticket_account_index as u8,
//             )
//             .get_signer_seeds()],
//             ctx.operator, // here, operator pays rent
//             fund_supported_token_reserve_account,
//             ctx.fund_account.as_account_info(),
//             &[&fund_account.get_seeds()],
//             item.allocated_token_amount,
//         )?
//     };

//     // pricing service with updated token values
//     let pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
//         .new_pricing_service(accounts.iter().cloned())?;

//     // validation (expects diff <= 1)
//     let expected_sol_fee_amount = pricing_service
//         .get_token_amount_as_sol(pool_token_mint.key, item.allocated_token_amount)?
//         .saturating_sub(unstaking_sol_amount);
//     require_gte!(1, expected_sol_fee_amount.abs_diff(deducted_sol_fee_amount));

//     Ok(Some((
//         item.allocated_token_amount,
//         unstaking_sol_amount,
//         deducted_sol_fee_amount,
//     )))
// }
