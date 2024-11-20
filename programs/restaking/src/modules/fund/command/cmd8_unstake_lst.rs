use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};
use crate::errors;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::staking;
use crate::utils::PDASeeds;
use anchor_lang::prelude::*;
use anchor_spl::token::accessor::mint;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct UnstakeLSTCommand {
    #[max_len(10)]
    pub items: Vec<UnstakeLSTCommandItem>,
    pub state: UnstakeLSTCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct UnstakeLSTCommandItem {
    pub mint: Pubkey,
    pub token_amount: u64,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum UnstakeLSTCommandState {
    Init,
    ReadPoolState,
    Unstake,
}

impl SelfExecutable for UnstakeLSTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        // there are remaining tokens to handle
        if let Some(item) = self.items.get(0) {
            let supported_token = ctx.fund_account.get_supported_token(&item.mint)?;

            match &self.state {
                UnstakeLSTCommandState::Init => {
                    if item.token_amount > 0 {
                        // request to read pool account

                        match supported_token.get_pricing_source() {
                            TokenPricingSource::SPLStakePool {
                                address: pool_address,
                            } => {
                                let mut command = self.clone();
                                command.state = UnstakeLSTCommandState::ReadPoolState;

                                return Ok(Some(
                                    OperationCommand::UnstakeLST(command)
                                        .with_required_accounts(vec![pool_address]),
                                ));
                            }
                            TokenPricingSource::MarinadeStakePool { .. } => {
                                // TODO: support marinade..
                            }
                            _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                        }
                    }
                }
                UnstakeLSTCommandState::ReadPoolState => {
                    match supported_token.get_pricing_source() {
                        TokenPricingSource::SPLStakePool {
                            address: pool_address,
                        } => {
                            let [pool_account_info, _remaining_accounts @ ..] = accounts else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };
                            require_keys_eq!(pool_address, *pool_account_info.key);

                            let mut command = self.clone();
                            command.state = UnstakeLSTCommandState::Unstake;

                            let required_accounts_from_service =
                                staking::SPLStakePoolService::find_accounts_to_withdraw_sol(
                                    pool_account_info,
                                )?;
                            let mut required_accounts = Vec::new();
                            required_accounts
                                .extend_from_slice(required_accounts_from_service.0.as_slice());
                            required_accounts
                                .extend_from_slice(required_accounts_from_service.1.as_slice());
                            required_accounts
                                .push(ctx.fund_account.find_reserve_account_address().0);
                            required_accounts.push(
                                ctx.fund_account
                                    .find_supported_token_account_address(&item.mint)?
                                    .0,
                            );
                            required_accounts.push(ctx.fund_account.key());

                            return Ok(Some(
                                OperationCommand::UnstakeLST(command)
                                    .with_required_accounts(required_accounts),
                            ));
                        }
                        TokenPricingSource::MarinadeStakePool { .. } => {
                            // TODO: support marinade..
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    }
                }
                UnstakeLSTCommandState::Unstake => {
                    let [pool_program, pool_account, pool_token_mint, pool_token_program, withdraw_authority, reserve_stake_account, manager_fee_account, sysvar_clock_program, sysvar_stake_history_program, stake_program, fund_reserve_account, fund_supported_token_account, fund_account, _remaining_accounts @ ..] =
                        accounts
                    else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    let (to_sol_account_amount, returned_sol_amount) =
                        staking::SPLStakePoolService::new(
                            pool_program,
                            pool_account,
                            pool_token_mint,
                            pool_token_program,
                        )?
                        .withdraw_sol(
                            withdraw_authority,
                            reserve_stake_account,
                            manager_fee_account,
                            sysvar_clock_program,
                            sysvar_stake_history_program,
                            stake_program,
                            fund_supported_token_account,
                            fund_reserve_account,
                            fund_account,
                            &ctx.fund_account.get_signer_seeds(),
                            item.token_amount,
                        )?;

                    ctx.fund_account.sol_operation_reserved_amount = ctx
                        .fund_account
                        .sol_operation_reserved_amount
                        .checked_add(returned_sol_amount)
                        .ok_or_else(|| {
                            error!(errors::ErrorCode::FundUnexpectedReserveAccountBalanceException)
                        })?;

                    let fund_supported_token_info = ctx
                        .fund_account
                        .get_supported_token_mut(pool_token_mint.key)?;
                    fund_supported_token_info.set_operation_reserved_amount(
                        fund_supported_token_info
                            .get_operation_reserved_amount()
                            .checked_sub(item.token_amount)
                            .unwrap(),
                    );

                    msg!(
                        "unstaked {} tokens to get {} sol",
                        item.token_amount,
                        returned_sol_amount
                    );

                    require_gte!(returned_sol_amount, item.token_amount);
                    require_eq!(
                        ctx.fund_account.sol_operation_reserved_amount,
                        to_sol_account_amount
                    );
                }
            }

            // proceed to next token
            if self.items.len() > 1 {
                return Ok(Some(
                    OperationCommand::UnstakeLST(UnstakeLSTCommand {
                        items: self.items.iter().skip(1).copied().collect(),
                        state: UnstakeLSTCommandState::Init,
                    })
                    .with_required_accounts(vec![]),
                ));
            }
        }

        // TODO v0.3/operation: next step ... stake sol
        Ok(None)
    }
}
