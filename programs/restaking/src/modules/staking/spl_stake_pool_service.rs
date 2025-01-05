use std::num::NonZeroU32;

use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use solana_program::stake::state::StakeStateV2;
use spl_stake_pool::error::StakePoolError;
use spl_stake_pool::state::{StakePool, ValidatorListHeader, ValidatorStakeInfo};

use crate::errors::ErrorCode;
use crate::utils::SystemProgramExt;

/// There are two ways to withdraw SOL
/// * withdraw sol from reserve account
/// * withdraw stake from validators and how much each
#[derive(Debug)]
pub(in crate::modules) enum AvailableWithdrawals {
    Reserve,
    Validators(Vec<(Pubkey, u64)>),
}

pub(in crate::modules) trait SPLStakePoolInterface: anchor_lang::Id {}
impl<T: anchor_lang::Id> SPLStakePoolInterface for T {}

pub(in crate::modules) struct SPLStakePool;

impl anchor_lang::Id for SPLStakePool {
    fn id() -> Pubkey {
        spl_stake_pool::ID
    }
}

pub(in crate::modules) struct SPLStakePoolService<'info, T = SPLStakePool>
where
    T: SPLStakePoolInterface,
{
    spl_stake_pool_program: Program<'info, T>, // &'info AccountInfo<'info>,
    pool_account: &'info AccountInfo<'info>,
    pool_token_mint: InterfaceAccount<'info, Mint>,
    pool_token_program: Interface<'info, TokenInterface>,
}

impl<'info, T: SPLStakePoolInterface> SPLStakePoolService<'info, T> {
    #[inline(never)]
    pub fn new(
        spl_stake_pool_program: &'info AccountInfo<'info>,
        pool_account: &'info AccountInfo<'info>,
        pool_token_mint: &'info AccountInfo<'info>,
        pool_token_program: &'info AccountInfo<'info>,
    ) -> Result<Self> {
        let pool_account_data = Self::deserialize_pool_account(pool_account)?;

        require_keys_eq!(pool_account_data.pool_mint, pool_token_mint.key());
        require_keys_eq!(*pool_token_mint.owner, *pool_token_program.key);

        Ok(Self {
            pool_account,
            spl_stake_pool_program: Program::try_from(spl_stake_pool_program)?,
            pool_token_mint: InterfaceAccount::try_from(pool_token_mint)?,
            pool_token_program: Interface::try_from(pool_token_program)?,
        })
    }

    #[inline(always)]
    fn get_pool_account_data(&self) -> Result<StakePool> {
        Self::deserialize_pool_account(self.pool_account)
    }

    pub(super) fn deserialize_pool_account(pool_account: &AccountInfo) -> Result<StakePool> {
        let pool_account_data =
            StakePool::deserialize(&mut pool_account.try_borrow_data()?.as_ref())
                .map_err(|_| error!(error::ErrorCode::AccountDidNotDeserialize))?;

        require_eq!(pool_account_data.is_valid(), true);

        Ok(pool_account_data)
    }

    fn deserialize_stake_account(stake_account: &AccountInfo) -> Result<StakeStateV2> {
        StakeStateV2::deserialize(&mut &**stake_account.try_borrow_data()?)
            .map_err(|_| error!(error::ErrorCode::AccountDidNotDeserialize))
    }

    /// * withdraw_authority
    fn find_withdraw_authority_account_meta<'a>(
        pool_account: &impl AsRef<AccountInfo<'a>>,
    ) -> (Pubkey, bool) {
        (
            spl_stake_pool::find_withdraw_authority_program_address(
                &T::id(),
                pool_account.as_ref().key,
            )
            .0,
            false,
        )
    }

    /// * pool_program
    /// * pool_account(writable)
    /// * pool_token_mint(writable)
    /// * pool_token_program
    fn find_accounts_to_new(
        pool_account: &AccountInfo,
        pool_account_data: &StakePool,
    ) -> [(Pubkey, bool); 4] {
        [
            (T::id(), false),
            (pool_account.key(), true),
            (pool_account_data.pool_mint, true),
            (pool_account_data.token_program_id, false),
        ]
    }

    /// * (0) pool_program
    /// * (1) pool_account(writable)
    /// * (2) pool_token_mint(writable)
    /// * (3) pool_token_program
    /// * (4) withdraw_authority
    /// * (5) reserve_stake_account(writable)
    /// * (6) manager_fee_account(writable)
    #[inline(never)]
    pub fn find_accounts_to_deposit_sol(
        pool_account: &AccountInfo,
    ) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        let pool_account_data = &Self::deserialize_pool_account(pool_account)?;

        let accounts = Self::find_accounts_to_new(pool_account, pool_account_data)
            .into_iter()
            .chain([
                Self::find_withdraw_authority_account_meta(pool_account),
                (pool_account_data.reserve_stake, true),
                (pool_account_data.manager_fee_account, true),
            ]);

        Ok(accounts)
    }

    /// * (0) pool_program
    /// * (1) pool_account(writable)
    /// * (2) pool_token_mint(writable)
    /// * (3) pool_token_program
    /// * (4) reserve_stake_account
    /// * (5) validator_list_account
    #[inline(never)]
    pub fn find_accounts_to_get_available_unstake_account(
        pool_account: &AccountInfo,
    ) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        let pool_account_data = &Self::deserialize_pool_account(pool_account)?;

        let accounts = Self::find_accounts_to_new(pool_account, pool_account_data)
            .into_iter()
            .chain([
                (pool_account_data.reserve_stake, false),
                (pool_account_data.validator_list, false),
            ]);

        Ok(accounts)
    }

    /// * (0) pool_program
    /// * (1) pool_account(writable)
    /// * (2) pool_token_mint(writable)
    /// * (3) pool_token_program
    /// * (4) withdraw_authority
    /// * (5) reserve_stake_account(writable)
    /// * (6) manager_fee_account(writable)
    /// * (7) sysvar clock
    /// * (8) sysvar stake_history
    /// * (9) stake_program
    #[inline(never)]
    pub fn find_accounts_to_withdraw_sol(
        pool_account: &AccountInfo,
    ) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        let pool_account_data = &Self::deserialize_pool_account(pool_account)?;

        let accounts = Self::find_accounts_to_new(pool_account, pool_account_data)
            .into_iter()
            .chain([
                Self::find_withdraw_authority_account_meta(pool_account),
                (pool_account_data.reserve_stake, true),
                (pool_account_data.manager_fee_account, true),
                (solana_program::sysvar::clock::ID, false),
                (solana_program::sysvar::stake_history::ID, false),
                (solana_program::stake::program::ID, false),
            ]);

        Ok(accounts)
    }

    /// * (0) pool_program
    /// * (1) pool_account(writable)
    /// * (2) pool_token_mint(writable)
    /// * (3) pool_token_program
    /// * (4) withdraw_authority
    /// * (5) manager_fee_account(writable)
    /// * (6) validator_list_account
    /// * (7) sysvar clock
    /// * (8) stake_program
    #[inline(never)]
    pub fn find_accounts_to_withdraw_stake(
        pool_account: &'info AccountInfo<'info>,
    ) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        let pool_account_data = &Self::deserialize_pool_account(pool_account)?;

        let accounts = Self::find_accounts_to_new(pool_account, pool_account_data)
            .into_iter()
            .chain([
                Self::find_withdraw_authority_account_meta(pool_account),
                (pool_account_data.manager_fee_account, true),
                (pool_account_data.validator_list, false),
                (solana_program::sysvar::clock::ID, false),
                (solana_program::stake::program::ID, false),
            ]);

        Ok(accounts)
    }

    /// * (0) sysvar clock
    /// * (1) sysvar stake_history
    /// * (2) stake_program
    pub fn find_accounts_to_claim_sol() -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        Ok([
            (solana_program::sysvar::clock::ID, false),
            (solana_program::sysvar::stake_history::ID, false),
            (solana_program::stake::program::ID, false),
        ]
        .into_iter())
    }

    /// returns [to_pool_token_account_amount, minted_pool_token_amount, deducted_pool_token_fee_amount]
    #[inline(never)]
    pub fn deposit_sol(
        &self,
        // fixed
        withdraw_authority: &AccountInfo<'info>,
        reserve_stake_account: &AccountInfo<'info>,
        manager_fee_account: &AccountInfo<'info>,

        // variant
        to_pool_token_account: &'info AccountInfo<'info>,
        from_sol_account: &AccountInfo<'info>,
        from_sol_account_seeds: &[&[&[u8]]],

        sol_amount: u64,
    ) -> Result<(u64, u64, u64)> {
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
            &self.pool_token_mint.key(),
            self.pool_token_program.key,
            sol_amount,
        );

        solana_program::program::invoke_signed(
            &ix,
            &[
                self.spl_stake_pool_program.to_account_info(),
                self.pool_account.to_account_info(),
                withdraw_authority.to_account_info(),
                reserve_stake_account.to_account_info(),
                from_sol_account.to_account_info(),
                to_pool_token_account.to_account_info(),
                manager_fee_account.to_account_info(),
                to_pool_token_account.to_account_info(),
                self.pool_token_mint.to_account_info(),
                self.pool_token_program.to_account_info(),
            ],
            from_sol_account_seeds,
        )?;

        to_pool_token_account.reload()?;
        let to_pool_token_account_amount = to_pool_token_account.amount;
        let minted_pool_token_amount =
            to_pool_token_account_amount - to_pool_token_account_amount_before;

        let deducted_pool_token_fee_amount = {
            let pool_account_data = self.get_pool_account_data()?;
            pool_account_data
                .calc_pool_tokens_for_deposit(sol_amount)
                .and_then(|pool_token_amount| {
                    pool_account_data.calc_pool_tokens_sol_deposit_fee(pool_token_amount)
                })
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?
        };

        msg!("STAKE#spl: pool_token_mint={}, staked_sol_amount={}, deducted_pool_token_fee_amount={}, to_pool_token_account_amount={}, minted_pool_token_amount={}", self.pool_token_mint.key(), sol_amount, deducted_pool_token_fee_amount, to_pool_token_account_amount, minted_pool_token_amount);

        Ok((
            to_pool_token_account_amount,
            minted_pool_token_amount,
            deducted_pool_token_fee_amount,
        ))
    }

    /// gives max fee/expense ratio during a cycle of circulation
    /// returns (numerator, denominator)
    #[inline(never)]
    pub fn get_max_cycle_fee(pool_account: &AccountInfo) -> Result<(u64, u64)> {
        let pool_account_data = Self::deserialize_pool_account(pool_account)?;

        // it costs deposit and withdrawal fee
        let f1 = pool_account_data.sol_deposit_fee;

        let f2a = pool_account_data.sol_withdrawal_fee;
        let f2b = pool_account_data.stake_withdrawal_fee;

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
            err!(ErrorCode::FundOperationCommandExecutionFailedException)?;
        }

        Ok((fee_rate_bps as u64, 10_000))
    }

    #[inline(never)]
    pub fn get_available_withdrawals(
        &self,
        // fixed
        reserve_stake_account: &AccountInfo,
        validator_list_account: &AccountInfo,

        pool_token_amount: u64,
    ) -> Result<AvailableWithdrawals> {
        let pool_account_data = &self.get_pool_account_data()?;

        // First check reserve stake account state
        let StakeStateV2::Initialized(reserve_stake_account_meta) =
            Self::deserialize_stake_account(reserve_stake_account)?
        else {
            return Err(ProgramError::from(StakePoolError::WrongStakeStake))?;
        };

        let withdraw_sol_amount =
            Self::get_withdraw_sol_amount(pool_account_data, pool_token_amount)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        let minimum_reserved_sol_amount =
            spl_stake_pool::minimum_reserve_lamports(&reserve_stake_account_meta);

        msg!(
            "Attempting to withdraw {} lamports from reserve stake account",
            withdraw_sol_amount,
        );

        // When there's enough lamports at reserve stake account.
        // Exclude equality to prevent error from sub-decimal values
        if reserve_stake_account.lamports() > minimum_reserved_sol_amount + withdraw_sol_amount {
            return Ok(AvailableWithdrawals::Reserve);
        }

        msg!(
            "Maximum possible SOL withdrawal is {} lamports. You should try to withdraw stake from validators.",
            reserve_stake_account.lamports().saturating_sub(minimum_reserved_sol_amount + 1),
        );

        // Now we should find available validator to withdraw stake from.
        let mut validator_list_account_data = validator_list_account.try_borrow_mut_data()?;
        let (_, validator_list) =
            ValidatorListHeader::deserialize_vec(&mut validator_list_account_data)?;
        let num_validator_stake_infos = validator_list.len() as usize;
        let validator_stake_infos =
            validator_list.deserialize_slice::<ValidatorStakeInfo>(0, num_validator_stake_infos)?;

        // To minimize the number of accounts to withdraw stake,
        // we prefer accounts with more active staked sol amount.
        let mut indices: Vec<_> = (0..num_validator_stake_infos).collect();
        indices.sort_by(|left_index, right_index| {
            // descending order
            u64::from(validator_stake_infos[*right_index].active_stake_lamports).cmp(&u64::from(
                validator_stake_infos[*left_index].active_stake_lamports,
            ))
        });

        // Each stake account must have at least this much active staked sol amount.
        let minimum_staked_sol_amount = spl_stake_pool::minimum_stake_lamports(
            &reserve_stake_account_meta,
            solana_program::stake::tools::get_minimum_delegation()?,
        );

        // Iterate validator stake infos until we fully fill pool_token_amount.
        let mut remaining_pool_token_amount = pool_token_amount;
        let mut validator_stake_items = vec![];

        for index in indices {
            if remaining_pool_token_amount == 0 {
                break;
            }

            let validator_stake_info = &validator_stake_infos[index];

            let Some(available_sol_amount) = u64::from(validator_stake_info.active_stake_lamports)
                .checked_sub(minimum_staked_sol_amount)
            else {
                break; // already sorted by descending order
            };
            let available_pool_token_amount = Self::get_withdraw_stake_pool_token_amount(
                pool_account_data,
                available_sol_amount,
            )?;

            if available_pool_token_amount == 0 {
                continue;
            }

            let (stake_account_address, _) = spl_stake_pool::find_stake_program_address(
                self.spl_stake_pool_program.key,
                &validator_stake_info.vote_account_address,
                self.pool_account.key,
                NonZeroU32::new(validator_stake_info.validator_seed_suffix.into()),
            );
            let pool_token_amount = available_pool_token_amount.min(remaining_pool_token_amount);

            remaining_pool_token_amount -= pool_token_amount;
            validator_stake_items.push((stake_account_address, pool_token_amount));
        }

        Ok(AvailableWithdrawals::Validators(validator_stake_items))
    }

    /// How much SOL would be withdrawn from validator reserve account?
    fn get_withdraw_sol_amount(
        pool_account_data: &StakePool,
        pool_token_amount: u64,
    ) -> Option<u64> {
        let pool_tokens_fee =
            pool_account_data.calc_pool_tokens_sol_withdrawal_fee(pool_token_amount)?;
        let pool_tokens_burnt = pool_token_amount.checked_sub(pool_tokens_fee)?;

        pool_account_data.calc_lamports_withdraw_amount(pool_tokens_burnt)
    }

    /// How much pool tokens will be burnt for withdraw stake?
    fn get_withdraw_stake_pool_token_amount(
        pool_account_data: &StakePool,
        sol_amount: u64,
    ) -> Result<u64> {
        // pool_token_burnt = sol_amount / price
        let pool_token_burnt = crate::utils::get_proportional_amount(
            sol_amount,
            pool_account_data.pool_token_supply,
            pool_account_data.total_lamports,
        )?;
        // pool_token_amount = pool_token_burnt * 1/(1-fee)
        // if denominator is zero then fee is zero
        crate::utils::get_proportional_amount(
            pool_token_burnt,
            pool_account_data.stake_withdrawal_fee.denominator,
            pool_account_data
                .stake_withdrawal_fee
                .denominator
                .saturating_sub(pool_account_data.stake_withdrawal_fee.numerator),
        )
    }

    /// returns [unstaked_sol_amount, deducted_pool_token_fee_amount]
    #[inline(never)]
    pub fn withdraw_sol(
        &self,
        // fixed
        withdraw_authority: &AccountInfo<'info>,
        reserve_stake_account: &AccountInfo<'info>,
        manager_fee_account: &AccountInfo<'info>,
        clock: &AccountInfo<'info>,
        stake_history: &AccountInfo<'info>,
        stake_program: &AccountInfo<'info>,

        // variant
        to_sol_account: &AccountInfo<'info>,
        from_pool_token_account: &AccountInfo<'info>,
        from_pool_token_account_signer: &AccountInfo<'info>,
        from_pool_token_account_signer_seeds: &[&[&[u8]]],

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
            &self.pool_token_mint.key(),
            self.pool_token_program.key,
            pool_token_amount,
        );

        solana_program::program::invoke_signed(
            &withdraw_sol_ix,
            &[
                self.pool_account.to_account_info(),
                withdraw_authority.to_account_info(),
                from_pool_token_account_signer.to_account_info(),
                from_pool_token_account.to_account_info(),
                reserve_stake_account.to_account_info(),
                to_sol_account.to_account_info(),
                manager_fee_account.to_account_info(),
                self.pool_token_mint.to_account_info(),
                self.pool_token_program.to_account_info(),
                clock.to_account_info(),
                stake_history.to_account_info(),
                stake_program.to_account_info(),
            ],
            from_pool_token_account_signer_seeds,
        )?;

        let to_sol_account_amount = to_sol_account.lamports();
        let unstaked_sol_amount = to_sol_account_amount - to_sol_account_amount_before;

        let deducted_pool_token_fee_amount = self
            .get_pool_account_data()?
            .calc_pool_tokens_sol_withdrawal_fee(pool_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        msg!("UNSTAKE#spl: pool_token_mint={}, pool_token_amount={}, deducted_pool_token_fee_amount={}, to_sol_account_amount={}, unstaked_sol_amount={}", self.pool_token_mint.key(), pool_token_amount, deducted_pool_token_fee_amount, to_sol_account_amount, unstaked_sol_amount);

        Ok((unstaked_sol_amount, deducted_pool_token_fee_amount))
    }

    /// rent for `to_stake_account` is 0 so rent payer actually does not pay rent.
    ///
    /// returns [unstaking_sol_amount, deducted_pool_token_fee_amount]
    #[inline(never)]
    pub fn withdraw_stake(
        &self,
        // fixed
        system_program: &Program<'info, System>,
        withdraw_authority: &AccountInfo<'info>,
        manager_fee_account: &AccountInfo<'info>,
        validator_list_account: &AccountInfo<'info>,
        clock: &AccountInfo<'info>,
        stake_program: &AccountInfo<'info>,

        // variant
        validator_stake_account: &AccountInfo<'info>,

        to_stake_account: &AccountInfo<'info>,
        to_stake_account_seeds: &[&[&[u8]]],
        to_stake_account_rent_payer: &Signer<'info>,

        to_stake_account_withdraw_authority: &AccountInfo<'info>,
        to_stake_account_withdraw_authority_seeds: &[&[&[u8]]],

        from_pool_token_account: &AccountInfo<'info>,
        from_pool_token_account_signer: &AccountInfo<'info>,
        from_pool_token_account_signer_seeds: &[&[&[u8]]],

        pool_token_amount: u64,
    ) -> Result<(u64, u64)> {
        system_program.initialize_account(
            to_stake_account,
            to_stake_account_rent_payer, // already signer so we don't need signer seeds
            to_stake_account_seeds,
            StakeStateV2::size_of(),
            Some(0),
            &solana_program::stake::program::ID,
        )?;

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
            &self.pool_token_mint.key(),
            self.pool_token_program.key,
            pool_token_amount,
        );

        solana_program::program::invoke_signed(
            &withdraw_stake_ix,
            &[
                self.pool_account.to_account_info(),
                validator_list_account.to_account_info(),
                withdraw_authority.to_account_info(),
                validator_stake_account.to_account_info(),
                to_stake_account.to_account_info(),
                to_stake_account_withdraw_authority.to_account_info(),
                from_pool_token_account_signer.to_account_info(),
                from_pool_token_account.to_account_info(),
                manager_fee_account.to_account_info(),
                self.pool_token_mint.to_account_info(),
                clock.to_account_info(),
                self.pool_token_program.to_account_info(),
                stake_program.to_account_info(),
            ],
            from_pool_token_account_signer_seeds,
        )?;

        let unstaking_sol_amount = to_stake_account.lamports();

        let deactivate_ix = solana_program::stake::instruction::deactivate_stake(
            to_stake_account.key,
            to_stake_account_withdraw_authority.key,
        );

        solana_program::program::invoke_signed(
            &deactivate_ix,
            &[
                to_stake_account.to_account_info(),
                to_stake_account_withdraw_authority.to_account_info(),
                clock.to_account_info(),
            ],
            to_stake_account_withdraw_authority_seeds,
        )?;

        let deducted_pool_token_fee_amount = self
            .get_pool_account_data()?
            .calc_pool_tokens_stake_withdrawal_fee(pool_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        msg!("UNSTAKE#spl: pool_token_mint={}, pool_token_amount={}, deducted_pool_token_fee_amount={}, unstaked_sol_amount={}", self.pool_token_mint.key(), pool_token_amount, deducted_pool_token_fee_amount, unstaking_sol_amount);

        Ok((unstaking_sol_amount, deducted_pool_token_fee_amount))
    }

    /// returns [claimed_sol_amount]
    #[inline(never)]
    pub fn claim_sol(
        // just for logging
        pool_token_mint: &Pubkey,

        // fixed
        clock: &AccountInfo<'info>,
        stake_history: &AccountInfo<'info>,
        stake_program: &AccountInfo<'info>,

        // variant
        to_sol_account: &AccountInfo<'info>,
        from_stake_account: &AccountInfo<'info>,
        from_stake_account_withdraw_authority: &AccountInfo<'info>,
        from_stake_account_withdraw_authority_seeds: &[&[&[u8]]],
    ) -> Result<u64> {
        // Stake account is not withdrawable yet
        if Self::is_stake_account_withdrawable(&Self::deserialize_stake_account(
            from_stake_account,
        )?) {
            return Ok(0);
        }

        let to_sol_account_amount_before = to_sol_account.lamports();
        let unstaked_sol_amount = from_stake_account.lamports();

        let withdraw_ix = solana_program::stake::instruction::withdraw(
            from_stake_account.key,
            from_stake_account_withdraw_authority.key,
            to_sol_account.key,
            unstaked_sol_amount,
            None,
        );

        solana_program::program::invoke_signed(
            &withdraw_ix,
            &[
                from_stake_account.to_account_info(),
                from_stake_account_withdraw_authority.to_account_info(),
                clock.to_account_info(),
                stake_history.to_account_info(),
                stake_program.to_account_info(),
            ],
            from_stake_account_withdraw_authority_seeds,
        )?;

        let to_sol_account_amount = to_sol_account.lamports();
        let claimed_sol_amount = to_sol_account_amount - to_sol_account_amount_before;

        require_eq!(claimed_sol_amount, unstaked_sol_amount);

        msg!(
            "CLAIM#spl: pool_token_mint={}, to_sol_account_amount={}, claimed_sol_amount={}",
            pool_token_mint,
            to_sol_account_amount,
            claimed_sol_amount
        );

        Ok(claimed_sol_amount)
    }

    fn is_stake_account_withdrawable(stake_account_data: &StakeStateV2) -> bool {
        // TODO v0.4/operation
        true
    }
}
