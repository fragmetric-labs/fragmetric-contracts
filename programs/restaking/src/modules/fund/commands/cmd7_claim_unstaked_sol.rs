use anchor_lang::prelude::*;

use crate::modules::pricing::TokenPricingSource;
use crate::modules::staking::*;
use crate::utils::{AccountInfoExt, PDASeeds};
use crate::{errors::ErrorCode, utils::AsAccountInfo};

use super::{
    FundAccount, FundService, OperationCommand, OperationCommandContext, OperationCommandEntry,
    OperationCommandResult, SelfExecutable, FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ClaimUnstakedSOLCommand {
    state: ClaimUnstakedSOLCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ClaimUnstakedSOLCommandItem {
    token_mint: Pubkey,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum ClaimUnstakedSOLCommandState {
    #[default]
    New,
    Prepare {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        items: Vec<ClaimUnstakedSOLCommandItem>,
    },
    Execute {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        items: Vec<ClaimUnstakedSOLCommandItem>,
    },
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ClaimUnstakedSOLCommandResult {
    pub token_mint: Pubkey,
    pub claimed_sol_amount: u64,
    pub operation_reserved_sol_amount: u64,
    pub operation_receivable_sol_amount: u64,
}

impl SelfExecutable for ClaimUnstakedSOLCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let (result, entry) = match &self.state {
            ClaimUnstakedSOLCommandState::New => self.execute_init(ctx, accounts)?,
            ClaimUnstakedSOLCommandState::Prepare { items } => {
                self.execute_prepare(ctx, accounts, items.clone(), None)?
            }
            ClaimUnstakedSOLCommandState::Execute { items } => {
                self.execute_execute(ctx, accounts, items)?
            }
        };

        // TODO v0.4/operation: next step... unstake lst
        Ok((result, entry))
    }
}

impl ClaimUnstakedSOLCommand {
    fn execute_init<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let items = ctx
            .fund_account
            .load()?
            .get_supported_tokens_iter()
            .map(|supported_token| ClaimUnstakedSOLCommandItem {
                token_mint: supported_token.mint,
            })
            .collect();

        // prepare state does not require additional accounts,
        // so we can execute directly.
        self.execute_prepare(ctx, accounts, items, None)
    }

    fn execute_prepare<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: Vec<ClaimUnstakedSOLCommandItem>,
        previous_execution_result: Option<OperationCommandResult>,
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if items.is_empty() {
            return Ok((previous_execution_result, None));
        }

        // to ensure that `accounts` contains pool account
        FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
            .new_pricing_service(accounts.iter().cloned())?;

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

        let fund_reserve_account = fund_account.get_reserve_account_address()?;
        let fund_treasury_account = fund_account.get_treasury_account_address()?;

        let command = Self {
            state: ClaimUnstakedSOLCommandState::Execute { items },
        };

        let entry = match pricing_source {
            Some(TokenPricingSource::SPLStakePool { .. }) => {
                let accounts_to_claim_sol = <SPLStakePoolService>::find_accounts_to_claim_sol()?;
                let fund_stake_accounts = (0..5).map(|index| {
                    let address = *FundAccount::find_stake_account_address(
                        &ctx.fund_account.key(),
                        pool_account.key,
                        index,
                    );
                    (address, true)
                });

                let required_accounts = [(fund_reserve_account, true)]
                    .into_iter()
                    .chain(accounts_to_claim_sol)
                    .chain(fund_stake_accounts);

                command.with_required_accounts(required_accounts)
            }
            Some(TokenPricingSource::MarinadeStakePool { .. }) => {
                let accounts_to_claim_sol =
                    MarinadeStakePoolService::find_accounts_to_claim_sol(pool_account)?;
                let withdrawal_ticket_accounts = (0..5).map(|index| {
                    let address = *FundAccount::find_unstaking_ticket_account_address(
                        &ctx.fund_account.key(),
                        pool_account.key,
                        index,
                    );
                    (address, true)
                });

                let required_accounts =
                    [(fund_reserve_account, true), (fund_treasury_account, true)]
                        .into_iter()
                        .chain(accounts_to_claim_sol)
                        .chain(withdrawal_ticket_accounts);

                command.with_required_accounts(required_accounts)
            }
            Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. }) => {
                let accounts_to_claim_sol =
                    SanctumSingleValidatorSPLStakePoolService::find_accounts_to_claim_sol()?;
                let fund_stake_accounts = (0..5).map(|index| {
                    let address = *FundAccount::find_stake_account_address(
                        &ctx.fund_account.key(),
                        pool_account.key,
                        index,
                    );
                    (address, true)
                });

                let required_accounts = [(fund_reserve_account, true)]
                    .into_iter()
                    .chain(accounts_to_claim_sol)
                    .chain(fund_stake_accounts);

                command.with_required_accounts(required_accounts)
            }
            _ => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
        };

        Ok((previous_execution_result, Some(entry)))
    }

    fn execute_execute<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: &[ClaimUnstakedSOLCommandItem],
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
            Some(TokenPricingSource::SPLStakePool { address }) => {
                self.spl_stake_pool_claim_sol::<SPLStakePool>(ctx, accounts, item, address)?
            }
            Some(TokenPricingSource::MarinadeStakePool { address }) => {
                self.marinade_stake_pool_claim_sol(ctx, accounts, item, address)?
            }
            Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address }) => {
                self.spl_stake_pool_claim_sol::<SanctumSingleValidatorSPLStakePool>(
                    ctx, accounts, item, address,
                )?
            }
            _ => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
        };

        // prepare state does not require additional accounts,
        // so we can execute directly.
        self.execute_prepare(ctx, accounts, items[1..].to_vec(), result)
    }

    fn spl_stake_pool_claim_sol<'info, T: SPLStakePoolInterface>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        item: &ClaimUnstakedSOLCommandItem,
        pool_account_address: Pubkey,
    ) -> Result<Option<OperationCommandResult>> {
        let [fund_reserve_account, clock, stake_history, stake_program, remaining_accounts @ ..] =
            accounts
        else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };
        let fund_stake_accounts = {
            if remaining_accounts.len() < 5 {
                err!(error::ErrorCode::AccountNotEnoughKeys)?
            }
            &remaining_accounts[..5]
        };

        let mut total_claimed_sol_amount = 0;
        let mut claim_sol_count = 0;

        let fund_account = ctx.fund_account.load()?;

        for (index, fund_stake_account) in fund_stake_accounts.iter().enumerate() {
            let fund_stake_account_address = *FundAccount::find_stake_account_address(
                &ctx.fund_account.key(),
                &pool_account_address,
                index as u8,
            );

            require_keys_eq!(fund_stake_account_address, fund_stake_account.key());

            // Skip uninitialized stake account
            if !fund_stake_account.is_initialized() {
                continue;
            }

            let claimed_sol_amount = SPLStakePoolService::<T>::claim_sol(
                &item.token_mint,
                clock,
                stake_history,
                stake_program,
                fund_reserve_account,
                fund_stake_account,
                ctx.fund_account.as_account_info(),
                &[&fund_account.get_seeds()],
            )?;

            total_claimed_sol_amount += claimed_sol_amount;
            claim_sol_count += 1;
        }

        // claim did not happen
        if claim_sol_count == 0 {
            return Ok(None);
        }

        drop(fund_account);

        // update fund account
        let mut fund_account = ctx.fund_account.load_mut()?;
        fund_account.sol.operation_reserved_amount += total_claimed_sol_amount;
        fund_account.sol.operation_receivable_amount -= total_claimed_sol_amount;

        Ok(Some(
            ClaimUnstakedSOLCommandResult {
                token_mint: item.token_mint,
                claimed_sol_amount: total_claimed_sol_amount,
                operation_reserved_sol_amount: fund_account.sol.operation_reserved_amount,
                operation_receivable_sol_amount: fund_account.sol.operation_receivable_amount,
            }
            .into(),
        ))
    }

    fn marinade_stake_pool_claim_sol<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        item: &ClaimUnstakedSOLCommandItem,
        pool_account_address: Pubkey,
    ) -> Result<Option<OperationCommandResult>> {
        let [fund_reserve_account, fund_treasury_account, pool_program, pool_account, pool_token_mint, pool_token_program, pool_reserve_account, clock, remaining_accounts @ ..] =
            accounts
        else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };
        let withdrawal_ticket_accounts = {
            if remaining_accounts.len() < 5 {
                err!(error::ErrorCode::AccountNotEnoughKeys)?
            }
            &remaining_accounts[..5]
        };

        require_keys_eq!(pool_account_address, pool_account.key());
        require_keys_eq!(item.token_mint, pool_token_mint.key());

        let marinade_stake_pool_service = MarinadeStakePoolService::new(
            pool_program,
            pool_account,
            pool_token_mint,
            pool_token_program,
        )?;

        let mut total_claimed_sol_amount = 0;
        let mut claim_sol_count = 0;

        let fund_account = ctx.fund_account.load()?;

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

            // Skip uninitialized stake account
            if !withdrawal_ticket_account.is_initialized() {
                continue;
            }

            let claimed_sol_amount = marinade_stake_pool_service.claim_sol(
                ctx.system_program,
                pool_reserve_account,
                clock,
                withdrawal_ticket_account,
                fund_treasury_account,
                fund_reserve_account,
                &[&fund_account.get_reserve_account_seeds()],
            )?;

            total_claimed_sol_amount += claimed_sol_amount;
            claim_sol_count += 1;
        }

        // claim did not happen
        if claim_sol_count == 0 {
            return Ok(None);
        }

        drop(fund_account);

        // update fund account
        let mut fund_account = ctx.fund_account.load_mut()?;
        fund_account.sol.operation_reserved_amount += total_claimed_sol_amount;
        fund_account.sol.operation_receivable_amount -= total_claimed_sol_amount;

        Ok(Some(
            ClaimUnstakedSOLCommandResult {
                token_mint: item.token_mint,
                claimed_sol_amount: total_claimed_sol_amount,
                operation_reserved_sol_amount: fund_account.sol.operation_reserved_amount,
                operation_receivable_sol_amount: fund_account.sol.operation_receivable_amount,
            }
            .into(),
        ))
    }
}
