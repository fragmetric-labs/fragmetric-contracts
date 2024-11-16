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
use spl_stake_pool::state::StakePool as SPLStakePoolAccount;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct StakeSOLCommand {
    #[max_len(10)]
    pub remaining_lst_mints: Vec<Pubkey>,
    pub state: StakeSOLCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) enum StakeSOLCommandState {
    Init,
    ReadPoolState,
    DoStake,
}

impl SelfExecutable for StakeSOLCommand {
    fn execute(
        &self,
        ctx: &mut OperationCommandContext,
        accounts: &[AccountInfo],
    ) -> Result<Vec<OperationCommandEntry>> {
        // there are remaining tokens to handle
        if let Some(lst_mint) = self.remaining_lst_mints.get(0) {
            let supported_token = ctx.fund_account.get_supported_token(lst_mint)?;

            // TODO v0.3/staking: put business logic into the "StakingService" with abstraction to focus on flow control here
            match supported_token.get_pricing_source() {
                TokenPricingSource::SPLStakePool {
                    address: pool_address,
                } => {
                    match &self.state {
                        StakeSOLCommandState::Init => {
                            // request to read pool account
                            return Ok(vec![OperationCommand::StakeSOL(StakeSOLCommand {
                                remaining_lst_mints: self.remaining_lst_mints.clone(),
                                state: StakeSOLCommandState::ReadPoolState,
                            })
                            .with_required_accounts(vec![pool_address])]);
                        }
                        StakeSOLCommandState::ReadPoolState => {
                            // accounts
                            let [pool_account_info, _remaining_accounts @ ..] = accounts else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };
                            let pool_account = SPLStakePoolAccount::deserialize(
                                &mut &**pool_account_info.try_borrow_data()?,
                            )
                            .map_err(|_| error!(ErrorCode::AccountDidNotDeserialize))?;
                            require_eq!(pool_account.is_valid(), true);

                            return Ok(vec![OperationCommand::StakeSOL(StakeSOLCommand {
                                remaining_lst_mints: self.remaining_lst_mints.clone(),
                                state: StakeSOLCommandState::DoStake,
                            })
                            .with_required_accounts(vec![
                                ctx.fund_account.find_reserve_account_address().0,
                                spl_stake_pool::id(),
                                pool_account_info.key(),
                                spl_stake_pool::find_withdraw_authority_program_address(
                                    &spl_stake_pool::id(),
                                    &pool_address,
                                )
                                .0,
                                pool_account.reserve_stake,
                                pool_account.manager_fee_account,
                                pool_account.pool_mint,
                                pool_account.token_program_id,
                                ctx.fund_account
                                    .find_supported_token_account_address(lst_mint)?
                                    .0,
                            ])]);
                        }
                        StakeSOLCommandState::DoStake => {
                            let [fund_reserve_account, stake_pool_program, stake_pool, stake_pool_withdraw_authority, reserve_stake_account, manager_fee_account, pool_mint, token_program, fund_supported_token_account_to_stake, remaining_accounts @ ..] =
                                accounts
                            else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };

                            let mut fund_supported_token_account_to_stake_parsed =
                                parse_interface_account_boxed::<token_interface::TokenAccount>(
                                    fund_supported_token_account_to_stake,
                                )?;

                            // TODO: BELOW CALCULATION IS FOR THE PURPOSE OF TESTING, should have proper sol amounts for each lsts based on strategy config
                            let staking_lamports =
                                ctx.fund_account.sol_operation_reserved_amount.div_ceil(2);
                            if staking_lamports > 0 {
                                let before_fund_supported_token_amount =
                                    fund_supported_token_account_to_stake_parsed.amount;
                                staking::deposit_sol_to_spl_stake_pool(
                                    &staking::SPLStakePoolContext {
                                        program: stake_pool_program.clone(),
                                        stake_pool: stake_pool.clone(),
                                        sol_deposit_authority: None,
                                        stake_pool_withdraw_authority:
                                            stake_pool_withdraw_authority.clone(),
                                        reserve_stake_account: reserve_stake_account.clone(),
                                        manager_fee_account: manager_fee_account.clone(),
                                        pool_mint: pool_mint.clone(),
                                        token_program: token_program.clone(),
                                    },
                                    staking_lamports,
                                    fund_reserve_account,
                                    fund_supported_token_account_to_stake,
                                    &[&ctx.fund_account.find_reserve_account_seeds()],
                                )?;
                                fund_supported_token_account_to_stake_parsed.reload()?;
                                ctx.fund_account.sol_operation_reserved_amount = ctx
                                    .fund_account
                                    .sol_operation_reserved_amount
                                    .checked_sub(staking_lamports)
                                    .ok_or_else(|| {
                                        error!(errors::ErrorCode::FundUnexpectedReserveAccountBalanceException)
                                    })?;

                                let minted_supported_token_amount =
                                    fund_supported_token_account_to_stake_parsed.amount
                                        - before_fund_supported_token_amount;
                                let fund_supported_token_info =
                                    ctx.fund_account.get_supported_token_mut(
                                        fund_supported_token_account_to_stake_parsed.mint,
                                    )?;
                                fund_supported_token_info.set_operation_reserved_amount(
                                    fund_supported_token_info
                                        .get_operation_reserved_amount()
                                        .checked_add(minted_supported_token_amount)
                                        .unwrap(),
                                );
                                msg!(
                                    "staked {} sol to mint {} tokens",
                                    staking_lamports,
                                    minted_supported_token_amount
                                );

                                require_gte!(
                                    minted_supported_token_amount,
                                    staking_lamports.div_ceil(2)
                                );
                                require_eq!(
                                    fund_supported_token_info.get_operation_reserved_amount(),
                                    fund_supported_token_account_to_stake_parsed.amount
                                );
                            }
                        }
                    }
                }
                TokenPricingSource::MarinadeStakePool { .. } => {
                    // TODO: support marinade..
                }
                _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
            }

            // proceed to next token
            if self.remaining_lst_mints.len() > 1 {
                return Ok(vec![OperationCommand::StakeSOL(StakeSOLCommand {
                    remaining_lst_mints: self.remaining_lst_mints.iter().skip(1).copied().collect(),
                    state: StakeSOLCommandState::Init,
                })
                .with_required_accounts(vec![])]);
            }
        }

        // TODO: proceed to normalize lst...
        Ok(vec![])
    }
}
