use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};
use crate::errors;
use crate::modules::fund::SupportedTokenInfo;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::{fund, staking};
use crate::utils::parse_interface_account_boxed;
use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::accessor::mint;
use anchor_spl::token_interface;
use spl_stake_pool::instruction::deposit_sol;
use spl_stake_pool::state::StakePool as SPLStakePoolAccount;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct StakeSOLCommand {
    #[max_len(10)]
    pub lst_mints: Vec<Pubkey>,
    #[max_len(10)]
    pub staking_sol_amounts: Vec<u64>,
    pub state: StakeSOLCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum StakeSOLCommandState {
    Init,
    ReadPoolState,
    Stake,
}

impl SelfExecutable for StakeSOLCommand {
    fn execute(
        &self,
        ctx: &mut OperationCommandContext,
        accounts: &[AccountInfo],
    ) -> Result<Option<OperationCommandEntry>> {
        require_eq!(
            self.lst_mints.len(),
            self.staking_sol_amounts.len()
        );

        // there are remaining tokens to handle
        if let Some(lst_mint) = self.lst_mints.get(0) {
            let supported_token = ctx.fund_account.get_supported_token(lst_mint)?;

            match &self.state {
                StakeSOLCommandState::Init => {
                    let staking_sol_amount = *self.staking_sol_amounts.get(0).unwrap();
                    if staking_sol_amount > 0 {
                        // request to read pool account

                        match supported_token.get_pricing_source() {
                            TokenPricingSource::SPLStakePool {
                                address: pool_address,
                            } => {
                                let mut command = self.clone();
                                command.state = StakeSOLCommandState::ReadPoolState;

                                return Ok(Some(
                                    OperationCommand::StakeSOL(command)
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
                StakeSOLCommandState::ReadPoolState => {
                    match supported_token.get_pricing_source() {
                        TokenPricingSource::SPLStakePool {
                            address: pool_address,
                        } => {
                            let [pool_account_info, _remaining_accounts @ ..] = accounts else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };
                            require_keys_eq!(pool_address, *pool_account_info.key);

                            let mut command = self.clone();
                            command.state = StakeSOLCommandState::Stake;

                            let required_accounts_from_service =
                                staking::SPLStakePoolService::find_accounts_to_deposit_sol(
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
                                    .find_supported_token_account_address(lst_mint)?
                                    .0,
                            );

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

                    let staking_sol_amount = *self.staking_sol_amounts.get(0).unwrap();
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
                            staking_sol_amount,
                        )?;

                    ctx.fund_account.sol_operation_reserved_amount = ctx
                        .fund_account
                        .sol_operation_reserved_amount
                        .checked_sub(staking_sol_amount)
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
                        staking_sol_amount,
                        minted_supported_token_amount
                    );

                    require_gte!(
                        minted_supported_token_amount,
                        staking_sol_amount.div_ceil(2)
                    );
                    require_eq!(
                        fund_supported_token_info.get_operation_reserved_amount(),
                        to_pool_token_account_amount
                    );
                }
            }

            // proceed to next token
            if self.lst_mints.len() > 1 {
                return Ok(Some(
                    OperationCommand::StakeSOL(StakeSOLCommand {
                        lst_mints: self
                            .lst_mints
                            .iter()
                            .skip(1)
                            .copied()
                            .collect(),
                        staking_sol_amounts: self
                            .staking_sol_amounts
                            .iter()
                            .skip(1)
                            .copied()
                            .collect(),
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
