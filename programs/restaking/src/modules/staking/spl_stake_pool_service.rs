use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::solana_program::stake::state::StakeStateV2;
use anchor_spl::token_interface::TokenAccount;
use spl_stake_pool::big_vec::BigVec;
use spl_stake_pool::state::StakePool;
use std::num::NonZeroU32;

use crate::errors;
use crate::utils::SystemProgramExt;

#[derive(Clone, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum AvailableWithdrawals {
    Reserve,
    Validators(Vec<(Pubkey, u64)>),
}

impl From<AvailableWithdrawals> for Option<Vec<(Pubkey, u64)>> {
    fn from(value: AvailableWithdrawals) -> Self {
        match value {
            AvailableWithdrawals::Reserve => None,
            AvailableWithdrawals::Validators(v) => Some(v),
        }
    }
}

pub struct SPLStakePoolService<'info> {
    pub spl_stake_pool_program: &'info AccountInfo<'info>,
    pub pool_account: &'info AccountInfo<'info>,
    pub pool_token_mint: &'info AccountInfo<'info>,
    pub pool_token_program: &'info AccountInfo<'info>,
}

impl<'info> SPLStakePoolService<'info> {
    pub fn new(
        spl_stake_pool_program: &'info AccountInfo<'info>,
        pool_account: &'info AccountInfo<'info>,
        pool_token_mint: &'info AccountInfo<'info>,
        pool_token_program: &'info AccountInfo<'info>,
    ) -> Result<Self> {
        require_eq!(spl_stake_pool::ID, spl_stake_pool_program.key());

        Ok(Self {
            spl_stake_pool_program,
            pool_account,
            pool_token_mint,
            pool_token_program,
        })
    }

    pub(super) fn deserialize_pool_account(
        pool_account_info: &'info AccountInfo<'info>,
    ) -> Result<StakePool> {
        let pool_account = StakePool::deserialize(&mut &**pool_account_info.try_borrow_data()?)
            .map_err(|_| error!(ErrorCode::AccountDidNotDeserialize))?;
        require_eq!(pool_account.is_valid(), true);
        Ok(pool_account)
    }

    /// returns (pubkey, writable) of [pool_program, pool_account, pool_token_mint, pool_token_program]
    pub(super) fn deserialize_stake_account(
        stake_account_info: &'info AccountInfo<'info>,
    ) -> Result<StakeStateV2> {
        let stake_account =
            StakeStateV2::deserialize(&mut &**stake_account_info.try_borrow_data()?)
                .map_err(|_| error!(ErrorCode::AccountDidNotDeserialize))?;
        Ok(stake_account)
    }

    fn find_accounts_to_new(
        pool_account_info: &AccountInfo,
        pool_account: &StakePool,
    ) -> Vec<(Pubkey, bool)> {
        vec![
            // for Self::new
            (spl_stake_pool::ID, false),
            (pool_account_info.key(), true),
            (pool_account.pool_mint, true),
            (pool_account.token_program_id, false),
        ]
    }

    /// returns (pubkey, writable) of [pool_program, pool_account, pool_token_mint, pool_token_program, withdraw_authority, reserve_stake_account, manager_fee_account]
    pub fn find_accounts_to_deposit_sol(
        pool_account_info: &'info AccountInfo<'info>,
    ) -> Result<Vec<(Pubkey, bool)>> {
        let pool_account = Self::deserialize_pool_account(pool_account_info)?;
        let mut accounts = Self::find_accounts_to_new(pool_account_info, &pool_account);
        accounts.extend([
            // for self.deposit_sol
            (
                spl_stake_pool::find_withdraw_authority_program_address(
                    &spl_stake_pool::ID,
                    &pool_account_info.key(),
                )
                .0,
                false,
            ),
            (pool_account.reserve_stake, true),
            (pool_account.manager_fee_account, true),
        ]);
        Ok(accounts)
    }

    /// returns [to_pool_token_account_amount, minted_pool_token_amount, (deposit_fee_numerator, deposit_fee_denominator)]
    pub fn deposit_sol(
        &self,
        withdraw_authority: &'info AccountInfo<'info>,
        reserve_stake_account: &'info AccountInfo<'info>,
        manager_fee_account: &'info AccountInfo<'info>,

        from_sol_account: &'info AccountInfo<'info>,
        to_pool_token_account: &'info AccountInfo<'info>,
        from_sol_account_signer_seeds: &[&[u8]],

        sol_amount: u64,
    ) -> Result<(u64, u64, (u64, u64))> {
        let mut to_pool_token_account =
            InterfaceAccount::<TokenAccount>::try_from(to_pool_token_account)?;
        let to_pool_token_account_amount_before = to_pool_token_account.amount;

        let ix = spl_stake_pool::instruction::deposit_sol(
            self.spl_stake_pool_program.key,
            self.pool_account.key,
            withdraw_authority.key,
            reserve_stake_account.key,
            from_sol_account.key,
            &to_pool_token_account.key(),
            manager_fee_account.key,
            &to_pool_token_account.key(), // referer pool token account
            self.pool_token_mint.key,
            self.pool_token_program.key,
            sol_amount,
        );

        invoke_signed(
            &ix,
            &[
                self.spl_stake_pool_program.clone(),
                self.pool_account.clone(),
                withdraw_authority.clone(),
                reserve_stake_account.clone(),
                from_sol_account.clone(),
                to_pool_token_account.to_account_info(),
                manager_fee_account.clone(),
                to_pool_token_account.to_account_info(),
                self.pool_token_mint.clone(),
                self.pool_token_program.clone(),
            ],
            &[from_sol_account_signer_seeds],
        )?;

        to_pool_token_account.reload()?;
        let to_pool_token_account_amount = to_pool_token_account.amount;
        let minted_pool_token_amount =
            to_pool_token_account_amount - to_pool_token_account_amount_before;
        let deposit_fee = {
            let pool_account = Self::deserialize_pool_account(self.pool_account)?;
            (
                pool_account.sol_deposit_fee.numerator,
                pool_account.sol_deposit_fee.denominator.max(1),
            )
        };

        msg!("STAKE#spl: pool_token_mint={}, staked_sol_amount={}, to_pool_token_account_amount={}, minted_pool_token_amount={}, deposit_fee={:?}", self.pool_token_mint.key(), sol_amount, to_pool_token_account_amount, minted_pool_token_amount, deposit_fee);

        Ok((
            to_pool_token_account_amount,
            minted_pool_token_amount,
            deposit_fee,
        ))
    }

    pub fn find_accounts_to_get_available_unstake_account(
        pool_account_info: &'info AccountInfo<'info>,
    ) -> Result<Vec<(Pubkey, bool)>> {
        let pool_account = Self::deserialize_pool_account(pool_account_info)?;
        let mut accounts = Self::find_accounts_to_new(pool_account_info, &pool_account);
        accounts.extend([
            (pool_account.reserve_stake, true),
            (pool_account.validator_list, true),
            (solana_program::stake::program::ID, false),
        ]);
        Ok(accounts)
    }

    pub fn find_accounts_to_withdraw_sol_or_stake(
        pool_account_info: &'info AccountInfo<'info>,
    ) -> Result<Vec<(Pubkey, bool)>> {
        let pool_account = Self::deserialize_pool_account(pool_account_info)?;
        let accounts = vec![
            // for self.withdraw_sol
            (
                spl_stake_pool::find_withdraw_authority_program_address(
                    &spl_stake_pool::id(),
                    &pool_account_info.key(),
                )
                .0,
                false,
            ),
            (pool_account.manager_fee_account, true),
            (solana_program::sysvar::clock::ID, false),
            (solana_program::sysvar::stake_history::ID, false),
        ];
        Ok(accounts)
    }

    pub fn find_fund_stake_accounts_for_withdraw_stake(
        stake_account_signer_seeds: &[&[u8]],
    ) -> (Pubkey, bool, u8) {
        // return (pubkey, is_writable, bump)
        let (fund_stake_account, fund_stake_account_bump) =
            Pubkey::find_program_address(stake_account_signer_seeds, &crate::ID);
        return (fund_stake_account, true, fund_stake_account_bump);
    }

    /// gives max fee/expense ratio during a cycle of circulation
    /// returns (numerator, denominator)
    pub(in crate::modules) fn get_max_cycle_fee(
        pool_account_info: &'info AccountInfo<'info>,
    ) -> Result<(u64, u64)> {
        let pool_account = Self::deserialize_pool_account(pool_account_info)?;

        // it costs deposit and withdrawal fee
        let f1 = pool_account.sol_deposit_fee;

        let f2a = pool_account.sol_withdrawal_fee;
        let f2b = pool_account.stake_withdrawal_fee;

        // f1.numerator/f1.denominator > f2.numerator/f2.denominator
        let f2 = if f2b.denominator == 0
            || f2a.numerator * f2b.denominator > f2b.numerator * f2a.denominator
        {
            f2a
        } else {
            f2b
        };

        let fee_rate = 1.0
            - (1.0 - (f1.numerator as f32 / f1.denominator.max(1) as f32))
                * (1.0 - (f2.numerator as f32 / f2.denominator.max(1) as f32));
        let fee_rate_bps = (fee_rate * 10_000.0).ceil();
        if fee_rate_bps > u16::MAX as f32 {
            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?;
        }

        Ok((fee_rate_bps as u64, 10_000))
    }

    pub fn get_withdrawal_available_from_reserve_or_validator(
        pool_program_info: &'info AccountInfo<'info>,
        pool_account_info: &'info AccountInfo<'info>,
        reserve_stake_account_info: &'info AccountInfo<'info>,
        validator_list_account_info: &'info AccountInfo<'info>,
        pool_tokens: u64,
    ) -> Result<AvailableWithdrawals> {
        // return Vec<(where to withdraw account, how much to withdraw pool tokens)>
        let pool_account = Self::deserialize_pool_account(pool_account_info)?;

        let mut validator_list_account_data = validator_list_account_info.try_borrow_mut_data()?;
        let (_header, validator_list) =
            spl_stake_pool::state::ValidatorListHeader::deserialize_vec(
                &mut validator_list_account_data,
            )?;

        let withdraw_lamports =
            Self::calc_withdraw_sol_lamports_by_pool_tokens_amount(&pool_account, pool_tokens)?;
        msg!(
            "pool_tokens {}, withdraw_lamports {}",
            pool_tokens,
            withdraw_lamports
        );

        let mut available_validators_with_available_lamports: AvailableWithdrawals =
            AvailableWithdrawals::Reserve;
        let new_reserve_lamports = reserve_stake_account_info
            .lamports()
            .saturating_sub(withdraw_lamports);
        msg!(
            "reserve stake lamports {}",
            reserve_stake_account_info.lamports()
        );
        let reserve_stake_state = Self::deserialize_stake_account(reserve_stake_account_info)?;
        if let StakeStateV2::Initialized(meta) = reserve_stake_state {
            let minimum_reserve_lamports = spl_stake_pool::minimum_reserve_lamports(&meta);
            msg!("minimum_reserve_lamports {}", minimum_reserve_lamports);
            if new_reserve_lamports < minimum_reserve_lamports {
                msg!("Attempting to withdraw {} lamports, maximum possible SOL withdrawal is {} lamports", withdraw_lamports, reserve_stake_account_info.lamports().saturating_sub(minimum_reserve_lamports));
                msg!("You should try to withdraw from validators");

                // now we should find the validator_stake_account to withdraw from
                let (withdraw_lamports, pool_tokens_fee) =
                    Self::calc_withdraw_stake_lamports_by_pool_tokens_amount(
                        &pool_account,
                        pool_tokens,
                    )?;
                let mut lamports_out_left = withdraw_lamports;

                let stake_minimum_delegation =
                    solana_program::stake::tools::get_minimum_delegation()?;
                let reserve_stake_meta = reserve_stake_state.meta().ok_or(ProgramError::from(
                    spl_stake_pool::error::StakePoolError::WrongStakeStake,
                ))?;
                let minimum_required_lamports_for_stake_account =
                    spl_stake_pool::minimum_stake_lamports(
                        &reserve_stake_meta,
                        stake_minimum_delegation,
                    );

                loop {
                    let (available_validator_stake_info_opt, max_withdraw_lamports_from_validator) =
                        Self::get_available_validator_stake_info(
                            &validator_list,
                            &lamports_out_left,
                            &minimum_required_lamports_for_stake_account,
                        )?;
                    if let Some(available_validator_stake_info) = available_validator_stake_info_opt
                    {
                        let available_validator_vote_account_address =
                            available_validator_stake_info.vote_account_address;
                        let (available_validator_stake_account_address, _) =
                            spl_stake_pool::find_stake_program_address(
                                pool_program_info.key,
                                &available_validator_vote_account_address,
                                pool_account_info.key,
                                NonZeroU32::new(
                                    available_validator_stake_info.validator_seed_suffix.into(),
                                ),
                            );
                        // should add fee lamports back again for real withdrawal request
                        let withdraw_pool_tokens_from_lamports = pool_account
                            .calc_pool_tokens_for_deposit(max_withdraw_lamports_from_validator)
                            .ok_or(errors::ErrorCode::CalculationArithmeticException)?;
                        if let AvailableWithdrawals::Validators(ref mut validators) =
                            available_validators_with_available_lamports
                        {
                            validators.push((
                                available_validator_stake_account_address,
                                withdraw_pool_tokens_from_lamports,
                            ))
                        } else {
                            available_validators_with_available_lamports =
                                AvailableWithdrawals::Validators(vec![(
                                    available_validator_stake_account_address,
                                    withdraw_pool_tokens_from_lamports,
                                )]);
                        }
                        lamports_out_left =
                            lamports_out_left.saturating_sub(max_withdraw_lamports_from_validator);
                    } else {
                        // then it means there's no available active stakes
                        available_validators_with_available_lamports =
                            AvailableWithdrawals::Validators(vec![]);
                        break;
                    }
                    if lamports_out_left == 0 {
                        break;
                    }
                }

                // should add fee lamports back again for real withdrawal request
                if let AvailableWithdrawals::Validators(ref mut validators) =
                    available_validators_with_available_lamports
                {
                    let count = validators.len() as u64;
                    let each_fee = pool_tokens_fee / count;
                    for (_validator_key, token_amount) in validators.iter_mut() {
                        *token_amount += each_fee;
                    }
                }
            }
            // else, then there's enough lamports at reserve stake
        } else {
            msg!("Reserve stake account not in intialized state");
            return Err(ProgramError::from(
                spl_stake_pool::error::StakePoolError::WrongStakeStake,
            ))?;
        }

        // dev) if you want to test withdraw from validator, uncomment this code and run
        // let (withdraw_lamports, pool_tokens_fee) =
        //     Self::calc_withdraw_stake_lamports_by_pool_tokens_amount(&pool_account, pool_tokens)?;
        // let mut lamports_out_left = withdraw_lamports;
        // msg!(
        //     "withdraw_lamports {}, pool_tokens_fee {}",
        //     withdraw_lamports,
        //     pool_tokens_fee
        // );

        // let stake_minimum_delegation = solana_program::stake::tools::get_minimum_delegation()?;
        // let reserve_stake_meta = reserve_stake_state.meta().ok_or(ProgramError::from(
        //     spl_stake_pool::error::StakePoolError::WrongStakeStake,
        // ))?;
        // let minimum_required_lamports_for_stake_account =
        //     spl_stake_pool::minimum_stake_lamports(&reserve_stake_meta, stake_minimum_delegation);
        // msg!(
        //     "minimum_required_lamports_for_stake_account {}",
        //     minimum_required_lamports_for_stake_account
        // );

        // loop {
        //     msg!("lamports_out_left {}", lamports_out_left);
        //     let (available_validator_stake_info_opt, max_withdraw_lamports_from_validator) =
        //         Self::get_available_validator_stake_info(
        //             &validator_list,
        //             &lamports_out_left,
        //             &minimum_required_lamports_for_stake_account,
        //         )?;
        //     if let Some(available_validator_stake_info) = available_validator_stake_info_opt {
        //         msg!(
        //             "max_withdraw_lamports_from_validator {}",
        //             max_withdraw_lamports_from_validator
        //         );
        //         let available_validator_vote_account_address =
        //             available_validator_stake_info.vote_account_address;
        //         let (available_validator_stake_account_address, _) =
        //             spl_stake_pool::find_stake_program_address(
        //                 pool_program_info.key,
        //                 &available_validator_vote_account_address,
        //                 pool_account_info.key,
        //                 NonZeroU32::new(
        //                     available_validator_stake_info.validator_seed_suffix.into(),
        //                 ),
        //             );
        //         // should add fee lamports back again for real withdrawal request
        //         let withdraw_pool_tokens_from_lamports = pool_account
        //             .calc_pool_tokens_for_deposit(max_withdraw_lamports_from_validator)
        //             .ok_or(errors::ErrorCode::CalculationArithmeticException)?;
        //         msg!(
        //             "withdraw_pool_tokens_from_lamports {}",
        //             withdraw_pool_tokens_from_lamports
        //         );
        //         if let AvailableWithdrawals::Validators(ref mut validators) =
        //             available_validators_with_available_lamports
        //         {
        //             validators.push((
        //                 available_validator_stake_account_address,
        //                 withdraw_pool_tokens_from_lamports,
        //             ))
        //         } else {
        //             available_validators_with_available_lamports =
        //                 AvailableWithdrawals::Validators(vec![(
        //                     available_validator_stake_account_address,
        //                     withdraw_pool_tokens_from_lamports,
        //                 )]);
        //         }
        //         lamports_out_left =
        //             lamports_out_left.saturating_sub(max_withdraw_lamports_from_validator);
        //     } else {
        //         // then it means there's no available active stakes
        //         available_validators_with_available_lamports =
        //             AvailableWithdrawals::Validators(vec![]);
        //         break;
        //     }
        //     if lamports_out_left == 0 {
        //         break;
        //     }
        // }

        // // should add fee lamports back again for real withdrawal request
        // if let AvailableWithdrawals::Validators(ref mut validators) =
        //     available_validators_with_available_lamports
        // {
        //     let count = validators.len() as u64;
        //     let each_fee = pool_tokens_fee / count;
        //     msg!("validator count {}, each_fee {}", count, each_fee);
        //     for (validator_key, token_amount) in validators.iter_mut() {
        //         *token_amount += each_fee;
        //         msg!(
        //             "validator key {}, token_amount {}",
        //             validator_key,
        //             token_amount
        //         );
        //     }
        // }

        msg!(
            "available_validators_with_available_lamports {:?}",
            available_validators_with_available_lamports
        );
        Ok(available_validators_with_available_lamports)
    }

    fn calc_withdraw_sol_lamports_by_pool_tokens_amount(
        pool_account: &spl_stake_pool::state::StakePool,
        pool_tokens: u64,
    ) -> Result<u64> {
        let pool_tokens_fee = pool_account
            .calc_pool_tokens_sol_withdrawal_fee(pool_tokens)
            .ok_or(errors::ErrorCode::CalculationArithmeticException)?;
        let pool_tokens_burnt = pool_tokens
            .checked_sub(pool_tokens_fee)
            .ok_or(errors::ErrorCode::CalculationArithmeticException)?;
        let withdraw_lamports = pool_account
            .calc_lamports_withdraw_amount(pool_tokens_burnt)
            .ok_or(errors::ErrorCode::CalculationArithmeticException)?;

        Ok(withdraw_lamports)
    }

    fn calc_withdraw_stake_lamports_by_pool_tokens_amount(
        pool_account: &spl_stake_pool::state::StakePool,
        pool_tokens: u64,
    ) -> Result<(u64, u64)> {
        let pool_tokens_fee = pool_account
            .calc_pool_tokens_stake_withdrawal_fee(pool_tokens)
            .ok_or(errors::ErrorCode::CalculationArithmeticException)?;
        let pool_tokens_burnt = pool_tokens
            .checked_sub(pool_tokens_fee)
            .ok_or(errors::ErrorCode::CalculationArithmeticException)?;
        let withdraw_lamports = pool_account
            .calc_lamports_withdraw_amount(pool_tokens_burnt)
            .ok_or(errors::ErrorCode::CalculationArithmeticException)?;

        Ok((withdraw_lamports, pool_tokens_fee))
    }

    // only checks active stakes
    fn get_available_validator_stake_info<'b, 'data>(
        validator_list: &'b BigVec<'data>,
        lamports: &u64,
        minimum_required_lamports: &u64,
    ) -> Result<(Option<spl_stake_pool::state::ValidatorStakeInfo>, u64)>
    where
        'b: 'data,
    {
        let len = validator_list.len() as usize;
        let mut current = 0;
        let mut current_index = 4;

        let mut max_active_lamports = 0;
        let mut max_active_lamports_validator =
            spl_stake_pool::state::ValidatorStakeInfo::default();

        let withdraw_tolarance_lamports = lamports
            .checked_add(*minimum_required_lamports)
            .ok_or(errors::ErrorCode::CalculationArithmeticException)?;
        msg!(
            "withdraw_tolarance_lamports {}",
            withdraw_tolarance_lamports
        );

        while current != len {
            let end_index =
                current_index + std::mem::size_of::<spl_stake_pool::state::ValidatorStakeInfo>();
            let current_slice = &validator_list.data[current_index..end_index];
            if spl_stake_pool::state::ValidatorStakeInfo::active_lamports_greater_than(
                current_slice,
                &withdraw_tolarance_lamports,
            ) {
                return Ok((
                    Some(
                        bytemuck::from_bytes::<spl_stake_pool::state::ValidatorStakeInfo>(
                            current_slice,
                        )
                        .clone(),
                    ),
                    lamports.clone(),
                ));
            }
            let mut active_lamports_of_current = u64::try_from_slice(&current_slice[0..8]).unwrap();
            if active_lamports_of_current >= *minimum_required_lamports {
                active_lamports_of_current -= *minimum_required_lamports; // safe
                if active_lamports_of_current > max_active_lamports {
                    max_active_lamports = active_lamports_of_current;
                    max_active_lamports_validator = bytemuck::from_bytes::<
                        spl_stake_pool::state::ValidatorStakeInfo,
                    >(current_slice)
                    .clone();
                }
            }
            current_index = end_index;
            current += 1;
        }

        if max_active_lamports == 0 {
            Ok((None, 0))
        } else {
            Ok((Some(max_active_lamports_validator), max_active_lamports))
        }
    }

    // TODO: remove
    pub fn find_stake_account_info_by_address(
        stake_account_infos: &[&'info AccountInfo<'info>],
        stake_account_address: &Pubkey,
    ) -> Result<&'info AccountInfo<'info>> {
        let stake_account_info = stake_account_infos
            .iter()
            .find(|account_info| account_info.key == stake_account_address)
            .ok_or(errors::ErrorCode::StakingAccountNotMatchedException)?;
        Ok(stake_account_info)
    }

    pub fn create_stake_account_if_needed(
        payer: &AccountInfo<'info>,
        stake_account: &AccountInfo<'info>,
        payer_signer_seeds: &[&[u8]],
        stake_account_signer_seeds: &[&[u8]],
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        // if given stake_account has lamports, it means it's already initialized by spl_stake_pool, so it's already an active stake account
        system_program.create_account(
            stake_account,
            stake_account_signer_seeds,
            payer,
            payer_signer_seeds,
            StakeStateV2::size_of(),
            &solana_program::stake::program::ID,
        )
    }

    pub fn find_accounts_to_claim_sol() -> Vec<(Pubkey, bool)> {
        vec![
            (solana_program::sysvar::clock::ID, false),
            (solana_program::sysvar::stake_history::ID, false),
            (solana_program::stake::program::ID, false),
        ]
    }

    /// gives (to_sol_account_amount, returned_sol_amount)
    pub fn withdraw_sol(
        &self,
        withdraw_authority: &AccountInfo<'info>,
        reserve_stake_account: &AccountInfo<'info>,
        manager_fee_account: &AccountInfo<'info>,
        sysvar_clock_program: &AccountInfo<'info>,
        sysvar_stake_history_program: &AccountInfo<'info>,
        stake_program: &AccountInfo<'info>,

        from_pool_token_account: &AccountInfo<'info>,
        to_sol_account: &AccountInfo<'info>,
        from_pool_token_account_signer: &AccountInfo<'info>,
        from_pool_token_account_signer_seeds: &[&[u8]],

        pool_token_amount: u64,
    ) -> Result<(u64, u64)> {
        let to_sol_account_amount_before = to_sol_account.lamports();

        let withdraw_sol_ix = spl_stake_pool::instruction::withdraw_sol(
            self.spl_stake_pool_program.key,
            self.pool_account.key,
            withdraw_authority.key,
            from_pool_token_account_signer.key,
            &from_pool_token_account.key(),
            reserve_stake_account.key,
            to_sol_account.key,
            manager_fee_account.key,
            self.pool_token_mint.key,
            self.pool_token_program.key,
            pool_token_amount,
        );

        invoke_signed(
            &withdraw_sol_ix,
            &[
                self.pool_account.clone(),
                withdraw_authority.clone(),
                from_pool_token_account_signer.clone(),
                from_pool_token_account.to_account_info(),
                reserve_stake_account.clone(),
                to_sol_account.clone(),
                manager_fee_account.clone(),
                self.pool_token_mint.to_account_info(),
                self.pool_token_program.to_account_info(),
                sysvar_clock_program.clone(),
                sysvar_stake_history_program.clone(),
                stake_program.clone(),
            ],
            &[from_pool_token_account_signer_seeds],
        )?;

        let to_sol_account_amount = to_sol_account.lamports();
        let returned_sol_amount = to_sol_account_amount - to_sol_account_amount_before;

        Ok((to_sol_account_amount, returned_sol_amount))
    }

    pub fn withdraw_stake(
        &self,
        withdraw_authority: &AccountInfo<'info>,
        validator_list_account: &AccountInfo<'info>,
        validator_stake_account: &AccountInfo<'info>,
        manager_fee_account: &AccountInfo<'info>,
        sysvar_clock_program: &AccountInfo<'info>,
        stake_program: &AccountInfo<'info>,

        from_pool_token_account: &AccountInfo<'info>,
        to_stake_account: &AccountInfo<'info>,
        from_pool_token_account_signer: &AccountInfo<'info>,
        from_pool_token_account_signer_seeds: &[&[u8]],
        to_stake_account_withdraw_authority: &AccountInfo<'info>,

        pool_token_amount: u64,
    ) -> Result<u64> {
        let withdraw_stake_ix = spl_stake_pool::instruction::withdraw_stake(
            self.spl_stake_pool_program.key,
            self.pool_account.key,
            validator_list_account.key,
            withdraw_authority.key,
            validator_stake_account.key,
            to_stake_account.key,
            to_stake_account_withdraw_authority.key, // User account to set as a new withdraw authority
            from_pool_token_account_signer.key,      // signer
            from_pool_token_account.key,
            manager_fee_account.key,
            self.pool_token_mint.key,
            self.pool_token_program.key,
            pool_token_amount,
        );

        invoke_signed(
            &withdraw_stake_ix,
            &[
                self.pool_account.clone(),
                validator_list_account.clone(),
                withdraw_authority.clone(),
                validator_stake_account.clone(),
                to_stake_account.clone(),
                to_stake_account_withdraw_authority.clone(),
                from_pool_token_account_signer.clone(),
                from_pool_token_account.clone(),
                manager_fee_account.clone(),
                self.pool_token_mint.to_account_info(),
                sysvar_clock_program.clone(),
                self.pool_token_program.to_account_info(),
                stake_program.clone(),
            ],
            &[from_pool_token_account_signer_seeds],
        )?;

        let returned_sol_amount = to_stake_account.lamports();
        Ok(returned_sol_amount)
    }

    pub fn deactivate_stake_account(
        sysvar_clock_program: &AccountInfo<'info>,

        stake_account_info: &AccountInfo<'info>,
        stake_account_withdraw_authority: &AccountInfo<'info>,
        stake_account_withdraw_authority_signer_seeds: &[&[u8]],
    ) -> Result<()> {
        let deactivate_ix = solana_program::stake::instruction::deactivate_stake(
            stake_account_info.key,
            stake_account_withdraw_authority.key,
        );

        invoke_signed(
            &deactivate_ix,
            &[
                stake_account_info.clone(),
                stake_account_withdraw_authority.clone(),
                sysvar_clock_program.clone(),
            ],
            &[stake_account_withdraw_authority_signer_seeds],
        )?;

        Ok(())
    }

    pub fn claim_sol(
        sysvar_clock_program: &AccountInfo<'info>,
        sysvar_stake_history_program: &AccountInfo<'info>,
        stake_program: &AccountInfo<'info>,

        stake_account_info: &'info AccountInfo<'info>,
        fund_reserve_account_info: &AccountInfo<'info>,
        withdraw_authority_signer_seeds: &[&[u8]],
    ) -> Result<()> {
        let stake_account = Self::deserialize_stake_account(stake_account_info);
        if let Ok(_stake_account) = stake_account {
            let withdrawn_lamports = stake_account_info.lamports();

            let withdraw_ix = solana_program::stake::instruction::withdraw(
                stake_account_info.key,
                fund_reserve_account_info.key,
                fund_reserve_account_info.key,
                withdrawn_lamports,
                None,
            );

            invoke_signed(
                &withdraw_ix,
                &[
                    stake_account_info.clone(),
                    fund_reserve_account_info.clone(),
                    sysvar_clock_program.clone(),
                    sysvar_stake_history_program.clone(),
                    stake_program.clone(),
                ],
                &[withdraw_authority_signer_seeds],
            )?;
        } else {
            msg!(
                "stake_account key {}, error {:?}",
                stake_account_info.key,
                stake_account.unwrap_err()
            );
        }

        Ok(())
    }
}
