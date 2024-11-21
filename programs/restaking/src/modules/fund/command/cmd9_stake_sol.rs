use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};
use crate::errors;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::staking;
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct StakeSOLCommand {
    #[max_len(10)]
    pub items: Vec<StakeSOLCommandItem>,
    pub state: StakeSOLCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct StakeSOLCommandItem {
    pub mint: Pubkey,
    pub sol_amount: u64,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum StakeSOLCommandState {
    Init,
    ReadPoolState,
    Stake,
}

impl SelfExecutable for StakeSOLCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        // there are remaining tokens to handle
        if let Some(item) = self.items.get(0) {
            let supported_token = ctx.fund_account.get_supported_token(&item.mint)?;

            match &self.state {
                StakeSOLCommandState::Init => {
                    if item.sol_amount > 0 {
                        // request to read pool account

                        match supported_token.pricing_source {
                            TokenPricingSource::SPLStakePool {
                                address: pool_address,
                            } => {
                                let mut command = self.clone();
                                command.state = StakeSOLCommandState::ReadPoolState;

                                return Ok(Some(
                                    OperationCommand::StakeSOL(command)
                                        .with_required_accounts(vec![(pool_address, false)]),
                                ));
                            }
                            TokenPricingSource::MarinadeStakePool { .. } => {
                                // TODO: support marinade..
                            }
                            _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                        }
                    }
                }
                StakeSOLCommandState::ReadPoolState => {
                    match supported_token.pricing_source {
                        TokenPricingSource::SPLStakePool {
                            address: pool_address,
                        } => {
                            let [pool_account_info, _remaining_accounts @ ..] = accounts else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };
                            require_keys_eq!(pool_address, *pool_account_info.key);

                            let mut command = self.clone();
                            command.state = StakeSOLCommandState::Stake;

                            let mut required_accounts =
                                staking::SPLStakePoolService::find_accounts_to_deposit_sol(
                                    pool_account_info,
                                )?;
                            required_accounts.extend(vec![
                                (ctx.fund_account.find_reserve_account_address().0, true),
                                (
                                    ctx.fund_account
                                        .find_supported_token_account_address(&item.mint)?,
                                    true,
                                ),
                            ]);

                            return Ok(Some(
                                OperationCommand::StakeSOL(command)
                                    .with_required_accounts(required_accounts),
                            ));
                        }
                        TokenPricingSource::MarinadeStakePool { .. } => {
                            // TODO: support marinade..
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    }
                }
                StakeSOLCommandState::Stake => {
                    let [pool_program, pool_account, pool_token_mint, pool_token_program, withdraw_authority, reserve_stake_account, manager_fee_account, fund_reserve_account, fund_supported_token_account, _remaining_accounts @ ..] =
                        accounts
                    else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    let (to_pool_token_account_amount, minted_supported_token_amount) =
                        staking::SPLStakePoolService::new(
                            pool_program,
                            pool_account,
                            pool_token_mint,
                            pool_token_program,
                        )?
                        .deposit_sol(
                            withdraw_authority,
                            reserve_stake_account,
                            manager_fee_account,
                            fund_reserve_account,
                            fund_supported_token_account,
                            &ctx.fund_account.find_reserve_account_seeds(),
                            item.sol_amount,
                        )?;

                    ctx.fund_account.sol_operation_reserved_amount = ctx
                        .fund_account
                        .sol_operation_reserved_amount
                        .checked_sub(item.sol_amount)
                        .ok_or_else(|| {
                            error!(errors::ErrorCode::FundUnexpectedReserveAccountBalanceException)
                        })?;

                    let fund_supported_token_info = ctx
                        .fund_account
                        .get_supported_token_mut(pool_token_mint.key)?;
                    fund_supported_token_info.set_operation_reserved_amount(
                        fund_supported_token_info
                            .get_operation_reserved_amount()
                            .checked_add(minted_supported_token_amount)
                            .unwrap(),
                    );

                    msg!(
                        "staked {} sol to mint {} tokens",
                        item.sol_amount,
                        minted_supported_token_amount
                    );

                    require_gte!(minted_supported_token_amount, item.sol_amount.div_ceil(2));
                    require_eq!(
                        fund_supported_token_info.get_operation_reserved_amount(),
                        to_pool_token_account_amount
                    );
                }
            }

            // proceed to next token
            if self.items.len() > 1 {
                return Ok(Some(
                    OperationCommand::StakeSOL(StakeSOLCommand {
                        items: self.items.iter().skip(1).copied().collect(),
                        state: StakeSOLCommandState::Init,
                    })
                    .with_required_accounts(vec![]),
                ));
            }
        }

        // TODO v0.3/operation: next step after stake sol
        Ok(None)
    }
}
