use crate::events;
use crate::modules::fund::FundAccount;
use crate::modules::reward::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token_interface;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
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

    fn find_accounts_for_new(
        pool_account_info: &AccountInfo,
        pool_account: &StakePool,
    ) -> [Pubkey; 4] {
        [
            // for Self::new
            spl_stake_pool::ID,
            pool_account_info.key(),
            pool_account.pool_mint,
            pool_account.token_program_id,
        ]
    }

    pub fn find_accounts_to_deposit_sol(
        pool_account_info: &'a AccountInfo<'info>,
    ) -> Result<([Pubkey; 4], [Pubkey; 3])> {
        let pool_account = Self::deserialize_pool_account(pool_account_info)?;
        Ok((
            Self::find_accounts_for_new(pool_account_info, &pool_account),
            [
                // for self.deposit_sol
                spl_stake_pool::find_withdraw_authority_program_address(
                    &spl_stake_pool::ID,
                    &pool_account_info.key(),
                )
                .0,
                pool_account.reserve_stake,
                pool_account.manager_fee_account,
            ],
        ))
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

    pub fn find_accounts_to_withdraw_sol(
        pool_account_info: &'a AccountInfo<'info>,
    ) -> Result<([Pubkey; 4], [Pubkey; 6])> {
        let pool_account = Self::deserialize_pool_account(pool_account_info)?;
        Ok((
            Self::find_accounts_for_new(pool_account_info, &pool_account),
            [
                // for self.deposit_sol
                spl_stake_pool::find_withdraw_authority_program_address(
                    &spl_stake_pool::id(),
                    &pool_account_info.key(),
                )
                .0,
                pool_account.reserve_stake,
                pool_account.manager_fee_account,
                solana_program::sysvar::clock::ID,
                solana_program::sysvar::stake_history::ID,
                solana_program::stake::program::ID,
            ],
        ))
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
}
