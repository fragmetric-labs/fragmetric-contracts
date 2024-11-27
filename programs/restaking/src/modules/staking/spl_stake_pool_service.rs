use std::num::NonZeroU32;

use crate::errors;
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::solana_program::stake::state::StakeStateV2;
use anchor_spl::token_interface::TokenAccount;
use spl_stake_pool::big_vec::BigVec;
use spl_stake_pool::state::StakePool;

pub struct SPLStakePoolService<'info: 'a, 'a> {
    pub spl_stake_pool_program: &'a AccountInfo<'info>,
    pub pool_account: &'a AccountInfo<'info>,
    pub pool_token_mint: &'a AccountInfo<'info>,
    pub pool_token_program: &'a AccountInfo<'info>,
}

impl<'info, 'a> SPLStakePoolService<'info, 'a> {
    pub fn new(
        spl_stake_pool_program: &'a AccountInfo<'info>,
        pool_account: &'a AccountInfo<'info>,
        pool_token_mint: &'a AccountInfo<'info>,
        pool_token_program: &'a AccountInfo<'info>,
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
        pool_account_info: &'a AccountInfo<'info>,
    ) -> Result<StakePool> {
        let pool_account_info_narrowed =
            unsafe { std::mem::transmute::<_, &'a AccountInfo<'a>>(pool_account_info) };
        let pool_account =
            StakePool::deserialize(&mut &**pool_account_info_narrowed.try_borrow_data()?)
                .map_err(|_| error!(ErrorCode::AccountDidNotDeserialize))?;
        require_eq!(pool_account.is_valid(), true);
        Ok(pool_account)
    }

    pub(super) fn deserialize_reserve_stake_account(
        reserve_stake_account_info: &'a AccountInfo<'info>,
    ) -> Result<StakeStateV2> {
        let reserve_stake_account_info_narrowed =
            unsafe { std::mem::transmute::<_, &'a AccountInfo<'a>>(reserve_stake_account_info) };
        let reserve_stake_account = StakeStateV2::deserialize(
            &mut &**reserve_stake_account_info_narrowed.try_borrow_data()?,
        )
        .map_err(|_| error!(ErrorCode::AccountDidNotDeserialize))?;
        Ok(reserve_stake_account)
    }

    fn find_accounts_for_new(
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

    pub fn find_accounts_to_deposit_sol(
        pool_account_info: &'a AccountInfo<'info>,
    ) -> Result<Vec<(Pubkey, bool)>> {
        let pool_account = Self::deserialize_pool_account(pool_account_info)?;
        let mut accounts = Self::find_accounts_for_new(pool_account_info, &pool_account);
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

    /// gives (to_pool_token_account_amount, minted_pool_token_amount)
    pub fn deposit_sol(
        &self,
        withdraw_authority: &AccountInfo<'info>,
        reserve_stake_account: &AccountInfo<'info>,
        manager_fee_account: &AccountInfo<'info>,

        from_sol_account: &AccountInfo<'info>,
        to_pool_token_account: &AccountInfo<'info>,
        from_sol_account_signer_seeds: &[&[u8]],

        sol_amount: u64,
    ) -> Result<(u64, u64)> {
        let to_pool_token_account_narrowed =
            unsafe { std::mem::transmute::<_, &'a AccountInfo<'a>>(to_pool_token_account) };
        let mut to_pool_token_account_parsed =
            InterfaceAccount::<TokenAccount>::try_from(to_pool_token_account_narrowed)?;
        let to_pool_token_account_amount_before = to_pool_token_account_parsed.amount;

        // TODO: consider using spl_stake_pool::instruction::deposit_sol_with_slippage
        let ix = spl_stake_pool::instruction::deposit_sol(
            self.spl_stake_pool_program.key,
            self.pool_account.key,
            withdraw_authority.key,
            reserve_stake_account.key,
            from_sol_account.key,
            to_pool_token_account.key,
            manager_fee_account.key,
            to_pool_token_account.key, // referer pool token account
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
                to_pool_token_account.clone(),
                manager_fee_account.clone(),
                to_pool_token_account.clone(),
                self.pool_token_mint.clone(),
                self.pool_token_program.clone(),
            ],
            &[from_sol_account_signer_seeds],
        )?;

        to_pool_token_account_parsed.reload()?;
        let to_pool_token_account_amount = to_pool_token_account_parsed.amount;
        let minted_pool_token_amount =
            to_pool_token_account_amount - to_pool_token_account_amount_before;

        Ok((to_pool_token_account_amount, minted_pool_token_amount))
    }

    pub fn find_accounts_to_withdraw_sol_or_stake(
        pool_account_info: &'a AccountInfo<'info>,
    ) -> Result<Vec<(Pubkey, bool)>> {
        let pool_account = Self::deserialize_pool_account(pool_account_info)?;
        let mut accounts = Self::find_accounts_for_new(pool_account_info, &pool_account);
        accounts.extend([
            // for self.withdraw_sol
            (
                spl_stake_pool::find_withdraw_authority_program_address(
                    &spl_stake_pool::id(),
                    &pool_account_info.key(),
                )
                .0,
                false,
            ),
            (pool_account.reserve_stake, true),
            (pool_account.validator_list, true),
            (pool_account.manager_fee_account, true),
            (solana_program::sysvar::clock::ID, false),
            (solana_program::sysvar::stake_history::ID, false),
            (solana_program::stake::program::ID, false),
        ]);
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

    pub fn get_withdrawal_available_from_reserve_or_validator(
        pool_program_info: &'a AccountInfo<'info>,
        pool_account_info: &'a AccountInfo<'info>,
        reserve_stake_account_info: &'a AccountInfo<'info>,
        validator_list_account_info: &'a AccountInfo<'info>,
        pool_tokens: u64,
    ) -> Result<Option<Vec<(Pubkey, u64)>>> {
        // return Vec<(where to withdraw account, how much to withdraw pool tokens)>
        let pool_account = Self::deserialize_pool_account(pool_account_info)?;

        let mut validator_list_account_data = validator_list_account_info.try_borrow_mut_data()?;
        let (_header, validator_list) =
            spl_stake_pool::state::ValidatorListHeader::deserialize_vec(
                &mut validator_list_account_data,
            )?;

        let mut withdraw_lamports =
            Self::calc_withdraw_sol_lamports_by_pool_tokens_amount(&pool_account, pool_tokens)?;
        msg!(
            "pool_tokens {}, withdraw_lamports {}",
            pool_tokens,
            withdraw_lamports
        );

        let mut available_validators_with_available_lamports: Option<Vec<(Pubkey, u64)>> = None;
        let new_reserve_lamports = reserve_stake_account_info
            .lamports()
            .saturating_sub(withdraw_lamports);
        msg!(
            "reserve stake lamports {}",
            reserve_stake_account_info.lamports()
        );
        let reserve_stake_state =
            Self::deserialize_reserve_stake_account(reserve_stake_account_info)?;
        if let StakeStateV2::Initialized(meta) = reserve_stake_state {
            let minimum_reserve_lamports = spl_stake_pool::minimum_reserve_lamports(&meta);
            msg!("minimum_reserve_lamports {}", minimum_reserve_lamports);
            if new_reserve_lamports < minimum_reserve_lamports {
                msg!("Attempting to withdraw {} lamports, maximum possible SOL withdrawal is {} lamports", withdraw_lamports, reserve_stake_account_info.lamports().saturating_sub(minimum_reserve_lamports));
                msg!("You should try to withdraw from validators");

                // now we should find the validator_stake_account to withdraw from
                withdraw_lamports = Self::calc_withdraw_stake_lamports_by_pool_tokens_amount(
                    &pool_account,
                    pool_tokens,
                )?;
                let mut lamports_out_left = withdraw_lamports;

                loop {
                    let (available_validator_stake_info_opt, withdraw_lamports_from_validator) =
                        Self::get_available_validator_stake_info(
                            &validator_list,
                            &lamports_out_left,
                        );
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
                        let max_active_pool_tokens_from_lamports = pool_account
                            .calc_pool_tokens_for_deposit(withdraw_lamports_from_validator)
                            .ok_or(errors::ErrorCode::CalculationArithmeticException)?;
                        available_validators_with_available_lamports
                            .get_or_insert_with(Vec::new)
                            .push((
                                available_validator_stake_account_address,
                                max_active_pool_tokens_from_lamports,
                            ));
                    } else {
                        // then it means there's no available active stakes
                        available_validators_with_available_lamports
                            .get_or_insert_with(Vec::new)
                            .push((Pubkey::default(), 0));
                        break;
                    }
                    lamports_out_left =
                        lamports_out_left.saturating_sub(withdraw_lamports_from_validator);
                    if lamports_out_left == 0 {
                        break;
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
        // withdraw_lamports =
        //     Self::calc_withdraw_stake_lamports_by_pool_tokens_amount(&pool_account, pool_tokens)?;
        // let mut lamports_out_left = withdraw_lamports;

        // loop {
        //     let (available_validator_stake_info_opt, withdraw_lamports_from_validator) =
        //         Self::get_available_validator_stake_info(&validator_list, &lamports_out_left);
        //     if let Some(available_validator_stake_info) = available_validator_stake_info_opt {
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
        //         let max_active_pool_tokens_from_lamports = pool_account
        //             .calc_pool_tokens_for_deposit(withdraw_lamports_from_validator)
        //             .ok_or(errors::ErrorCode::CalculationArithmeticException)?;
        //         available_validators_with_available_lamports
        //             .get_or_insert_with(Vec::new)
        //             .push((
        //                 available_validator_stake_account_address,
        //                 max_active_pool_tokens_from_lamports,
        //             ));
        //     } else {
        //         // then it means there's no available active stakes
        //         available_validators_with_available_lamports
        //             .get_or_insert_with(Vec::new)
        //             .push((Pubkey::default(), 0));
        //         break;
        //     }
        //     lamports_out_left = lamports_out_left.saturating_sub(withdraw_lamports_from_validator);
        //     if lamports_out_left == 0 {
        //         break;
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
    ) -> Result<u64> {
        let pool_tokens_fee = pool_account
            .calc_pool_tokens_stake_withdrawal_fee(pool_tokens)
            .ok_or(errors::ErrorCode::CalculationArithmeticException)?;
        let pool_tokens_burnt = pool_tokens
            .checked_sub(pool_tokens_fee)
            .ok_or(errors::ErrorCode::CalculationArithmeticException)?;
        let withdraw_lamports = pool_account
            .calc_lamports_withdraw_amount(pool_tokens_burnt)
            .ok_or(errors::ErrorCode::CalculationArithmeticException)?;

        Ok(withdraw_lamports)
    }

    // only checks active stakes
    fn get_available_validator_stake_info<'b, 'data>(
        validator_list: &'b BigVec<'data>,
        lamports: &u64,
    ) -> (Option<spl_stake_pool::state::ValidatorStakeInfo>, u64)
    where
        'b: 'data,
    {
        let len = validator_list.len() as usize;
        let mut current = 0;
        let mut current_index = 4;

        let mut max_active_lamports = 0;
        let mut max_active_lamports_validator =
            spl_stake_pool::state::ValidatorStakeInfo::default();

        while current != len {
            let end_index =
                current_index + std::mem::size_of::<spl_stake_pool::state::ValidatorStakeInfo>();
            let current_slice = &validator_list.data[current_index..end_index];
            if spl_stake_pool::state::ValidatorStakeInfo::active_lamports_greater_than(
                current_slice,
                &lamports,
            ) {
                return (
                    Some(
                        bytemuck::from_bytes::<spl_stake_pool::state::ValidatorStakeInfo>(
                            current_slice,
                        )
                        .clone(),
                    ),
                    lamports.clone(),
                );
            }
            let active_lamports_of_current = u64::try_from_slice(&current_slice[0..8]).unwrap();
            if active_lamports_of_current > max_active_lamports {
                max_active_lamports = active_lamports_of_current;
                max_active_lamports_validator = bytemuck::from_bytes::<
                    spl_stake_pool::state::ValidatorStakeInfo,
                >(current_slice)
                .clone();
            }
            current_index = end_index;
            current += 1;
        }

        if max_active_lamports == 0 {
            return (None, 0);
        } else {
            return (Some(max_active_lamports_validator), max_active_lamports);
        }
    }

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
    ) -> Result<()> {
        if stake_account.lamports() == 0 {
            // if given stake_account has lamports, it means it's already initialized by spl_stake_pool, so it's already an active stake account
            let create_account_ix = solana_program::system_instruction::create_account(
                payer.key,
                stake_account.key,
                0,
                StakeStateV2::size_of() as u64,
                &solana_program::stake::program::ID,
            );
            invoke_signed(
                &create_account_ix,
                &[payer.clone(), stake_account.clone()],
                &[payer_signer_seeds, stake_account_signer_seeds],
            )?;
        }

        Ok(())
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

        pool_token_amount: u64,
    ) -> Result<u64> {
        let withdraw_stake_ix = spl_stake_pool::instruction::withdraw_stake(
            self.spl_stake_pool_program.key,
            self.pool_account.key,
            validator_list_account.key,
            withdraw_authority.key,
            validator_stake_account.key,
            to_stake_account.key,
            to_stake_account.key,
            from_pool_token_account_signer.key, // signer
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
                to_stake_account.clone(),
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
}
