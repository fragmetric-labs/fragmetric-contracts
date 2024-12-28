use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::stake::state::StakeStateV2;
use spl_stake_pool::state::StakePool;

use crate::MAINNET_SANCTUM_SINGLE_VALIDATOR_SPL_STAKE_POOL_PROGRAM_ADDRESS;

use super::{AvailableWithdrawals, SPLStakePoolInterface, SPLStakePoolService};

pub struct SanctumSPLStakePool;

impl anchor_lang::Id for SanctumSPLStakePool {
    fn id() -> Pubkey {
        MAINNET_SANCTUM_SINGLE_VALIDATOR_SPL_STAKE_POOL_PROGRAM_ADDRESS
    }
}

impl SPLStakePoolInterface for SanctumSPLStakePool {}

pub struct SanctumSingleValidatorSPLStakePoolService<'info> {
    pub inner_spl_stake_pool_service: SPLStakePoolService<'info, SanctumSPLStakePool>,
    _marker: (),
}

impl<'info> SanctumSingleValidatorSPLStakePoolService<'info> {
    pub fn new(
        spl_stake_pool_program: &'info AccountInfo<'info>,
        pool_account: &'info AccountInfo<'info>,
        pool_token_mint: &'info AccountInfo<'info>,
        pool_token_program: &'info AccountInfo<'info>,
    ) -> Result<Self> {
        Ok(Self {
            inner_spl_stake_pool_service: SPLStakePoolService {
                spl_stake_pool_program: Program::try_from(spl_stake_pool_program)?,
                pool_account,
                pool_token_mint,
                pool_token_program,
            },
            _marker: (),
        })
    }

    pub(super) fn deserialize_pool_account(
        pool_account_info: &'info AccountInfo<'info>,
    ) -> Result<StakePool> {
        SPLStakePoolService::<SanctumSPLStakePool>::deserialize_pool_account(pool_account_info)
    }

    /// returns (pubkey, writable) of [pool_program, pool_account, pool_token_mint, pool_token_program]
    pub(super) fn deserialize_stake_account(
        stake_account_info: &'info AccountInfo<'info>,
    ) -> Result<StakeStateV2> {
        SPLStakePoolService::<SanctumSPLStakePool>::deserialize_stake_account(stake_account_info)
    }

    fn find_accounts_to_new(
        pool_account_info: &AccountInfo,
        pool_account: &StakePool,
    ) -> Vec<(Pubkey, bool)> {
        vec![
            // for Self::new
            (SanctumSPLStakePool::id(), false),
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
                    &SanctumSPLStakePool::id(),
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
                    &SanctumSPLStakePool::id(),
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
}
