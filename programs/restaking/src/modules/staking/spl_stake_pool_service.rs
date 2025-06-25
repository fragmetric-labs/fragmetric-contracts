use std::num::NonZeroU32;

use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token_interface::TokenAccount;
use solana_program::stake::state::StakeStateV2;
use spl_stake_pool::error::StakePoolError;
use spl_stake_pool::state::{StakePool, ValidatorListHeader, ValidatorStakeInfo};

use crate::errors::ErrorCode;
use crate::utils::SystemProgramExt;

use super::ValidateStakePool;

pub trait SPLStakePoolInterface: anchor_lang::Id {}
impl<T: anchor_lang::Id> SPLStakePoolInterface for T {}

pub struct SPLStakePool;

impl anchor_lang::Id for SPLStakePool {
    fn id() -> Pubkey {
        spl_stake_pool::ID
    }
}

pub(in crate::modules) struct SPLStakePoolService<'info, T = SPLStakePool>
where
    T: SPLStakePoolInterface,
{
    spl_stake_pool_program: &'info AccountInfo<'info>,
    pool_account: &'info AccountInfo<'info>, // deserialize on-demand
    pool_token_mint: &'info AccountInfo<'info>,
    pool_token_program: &'info AccountInfo<'info>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: SPLStakePoolInterface> ValidateStakePool for SPLStakePoolService<'_, T> {
    #[inline(never)]
    fn validate_stake_pool<'info>(
        pool_account: &'info AccountInfo<'info>,
        pool_token_mint: &Pubkey,
    ) -> Result<()> {
        let pool_account_data = Self::deserialize_pool_account(pool_account)?;

        require_keys_eq!(pool_account_data.pool_mint, pool_token_mint.key());

        Ok(())
    }
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

        require_keys_eq!(T::id(), spl_stake_pool_program.key());
        require_keys_eq!(pool_account_data.pool_mint, pool_token_mint.key());
        require_keys_eq!(pool_account_data.token_program_id, pool_token_program.key());

        Ok(Self {
            spl_stake_pool_program,
            pool_account,
            pool_token_mint,
            pool_token_program,
            _marker: Default::default(),
        })
    }

    pub(super) fn deserialize_pool_account(pool_account: &AccountInfo) -> Result<StakePool> {
        use borsh1::BorshDeserialize;

        let pool_account_data =
            StakePool::deserialize(&mut pool_account.try_borrow_data()?.as_ref())
                .map_err(|_| error!(error::ErrorCode::AccountDidNotDeserialize))?;

        require_keys_eq!(*pool_account.owner, T::id());
        require_eq!(pool_account_data.is_valid(), true);

        Ok(pool_account_data)
    }

    fn get_pool_account_data(&self) -> Result<StakePool> {
        Self::deserialize_pool_account(self.pool_account)
    }

    pub(super) fn deserialize_stake_account(stake_account: &AccountInfo) -> Result<StakeStateV2> {
        require_keys_eq!(*stake_account.owner, solana_program::stake::program::ID);
        StakeStateV2::deserialize(&mut stake_account.try_borrow_data()?.as_ref())
            .map_err(|_| error!(error::ErrorCode::AccountDidNotDeserialize))
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

    /// * (0) pool_program
    /// * (1) pool_account(writable)
    /// * (2) pool_token_mint(writable)
    /// * (3) pool_token_program
    /// * (4) withdraw_authority
    /// * (5) reserve_stake_account(writable)
    /// * (6) manager_fee_account(writable)
    /// * (7) validator_list_account(writable)
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
                (pool_account_data.validator_list, true),
            ]);

        Ok(accounts)
    }

    /// * (0) pool_program
    /// * (1) pool_account(writable)
    /// * (2) pool_token_mint(writable)
    /// * (3) pool_token_program
    /// * (4) validator_list_account
    #[inline(never)]
    pub fn find_accounts_to_get_validator_stake_accounts(
        pool_account: &AccountInfo,
    ) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        let pool_account_data = &Self::deserialize_pool_account(pool_account)?;

        let accounts = Self::find_accounts_to_new(pool_account, pool_account_data)
            .into_iter()
            .chain([(pool_account_data.validator_list, false)]);

        Ok(accounts)
    }

    /// * (0) pool_program
    /// * (1) pool_account(writable)
    /// * (2) pool_token_mint(writable)
    /// * (3) pool_token_program
    /// * (4) withdraw_authority
    /// * (5) reserve_stake_account(writable)
    /// * (6) manager_fee_account(writable)
    /// * (7) validator_list_account(writable)
    /// * (8) sysvar clock
    /// * (9) sysvar stake_history
    /// * (10) stake_program
    #[inline(never)]
    pub fn find_accounts_to_withdraw(
        pool_account: &AccountInfo,
    ) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        let pool_account_data = &Self::deserialize_pool_account(pool_account)?;

        let accounts = Self::find_accounts_to_new(pool_account, pool_account_data)
            .into_iter()
            .chain([
                Self::find_withdraw_authority_account_meta(pool_account),
                (pool_account_data.reserve_stake, true),
                (pool_account_data.manager_fee_account, true),
                (pool_account_data.validator_list, true),
                (solana_program::sysvar::clock::ID, false),
                (solana_program::sysvar::stake_history::ID, false),
                (solana_program::stake::program::ID, false),
            ]);

        Ok(accounts)
    }

    /// * (0) sysvar clock
    /// * (1) sysvar stake_history
    pub fn find_accounts_to_get_claimable_stake_accounts(
    ) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        Ok([
            (solana_program::sysvar::clock::ID, false),
            (solana_program::sysvar::stake_history::ID, false),
        ]
        .into_iter())
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
        validator_list_account: &AccountInfo<'info>,

        // variant
        to_pool_token_account: &'info AccountInfo<'info>,
        from_sol_account: &AccountInfo<'info>,
        from_sol_account_seeds: &[&[&[u8]]],

        sol_amount: u64,
    ) -> Result<(u64, u64, u64)> {
        let pool_account_data = &self.get_pool_account_data()?;

        // first update stake pool balance
        self.update_stake_pool_balance_if_needed(
            pool_account_data,
            withdraw_authority,
            reserve_stake_account,
            manager_fee_account,
            validator_list_account,
        )?;

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
            let minted_amount = pool_account_data
                .calc_pool_tokens_for_deposit(sol_amount)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
            let total_fee = pool_account_data
                .calc_pool_tokens_sol_deposit_fee(minted_amount)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
            let referral_fee = pool_account_data
                .calc_pool_tokens_sol_referral_fee(total_fee)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
            total_fee.saturating_sub(referral_fee) // manager_fee
        };

        msg!(
            "STAKE#spl: pool_token_mint={}, staked_sol_amount={}, deducted_pool_token_fee_amount={}, to_pool_token_account_amount={}, minted_pool_token_amount={}",
            self.pool_token_mint.key(),
            sol_amount,
            deducted_pool_token_fee_amount,
            to_pool_token_account_amount,
            minted_pool_token_amount
        );

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
            err!(ErrorCode::CalculationArithmeticException)?;
        }

        Ok((fee_rate_bps as u64, 10_000))
    }

    /// Find possible active stake accounts, up to `max_num_validators`.
    #[inline(never)]
    pub fn get_validator_stake_accounts(
        &self,
        // fixed
        validator_list_account: &AccountInfo,

        max_num_validators: usize,
    ) -> Result<Vec<Pubkey>> {
        let mut validator_list_account_data = validator_list_account.try_borrow_mut_data()?;
        let (_, validator_list) =
            ValidatorListHeader::deserialize_vec(&mut validator_list_account_data)?;
        let num_validator_stake_infos = validator_list.len();
        let validator_stake_infos = validator_list
            .deserialize_slice::<ValidatorStakeInfo>(0, num_validator_stake_infos as usize)?;

        let num_validators = max_num_validators.min(num_validator_stake_infos as usize);
        let mut validator_stake_accounts = Vec::with_capacity(num_validators);

        // To maximize available lamports to withdraw from active stake account,
        // we prefer accounts with more active staked sol amount.
        let mut indices = (0..num_validator_stake_infos).collect::<Vec<_>>();
        indices.sort_by_key(|index| {
            let active_stake_lamports =
                u64::from(validator_stake_infos[*index as usize].active_stake_lamports);
            // descending order
            u64::MAX - active_stake_lamports
        });
        for i in 0..num_validators {
            let validator_stake_info = &validator_stake_infos[indices[i] as usize];
            let (stake_account_address, _) = spl_stake_pool::find_stake_program_address(
                self.spl_stake_pool_program.key,
                &validator_stake_info.vote_account_address,
                self.pool_account.key,
                NonZeroU32::new(validator_stake_info.validator_seed_suffix.into()),
            );

            validator_stake_accounts.push(stake_account_address);

            // // How to find and update mock validator stake accounts...
            // // 1. Clone recent validator list account from mainnet
            // // 2. Uncomment the log below. It'll show top 5 stake accounts that we need.
            // // 3. Clone those stake accounts from mainnet

            // msg!(
            //     "Validator#{} {} active stake = {}",
            //     i,
            //     stake_account_address,
            //     u64::from(validator_stake_info.active_stake_lamports),
            // );
        }

        Ok(validator_stake_accounts)
    }

    fn update_stake_pool_balance_if_needed(
        &self,
        pool_account_data: &StakePool,
        // fixed
        withdraw_authority: &AccountInfo<'info>,
        reserve_stake_account: &AccountInfo<'info>,
        manager_fee_account: &AccountInfo<'info>,
        validator_list_account: &AccountInfo<'info>,
    ) -> Result<()> {
        if pool_account_data.last_update_epoch >= Clock::get()?.epoch {
            return Ok(());
        }

        let update_stake_pool_balance_ix = spl_stake_pool::instruction::update_stake_pool_balance(
            self.spl_stake_pool_program.key,
            self.pool_account.key,
            withdraw_authority.key,
            validator_list_account.key,
            reserve_stake_account.key,
            manager_fee_account.key,
            &self.pool_token_mint.key(),
            self.pool_token_program.key,
        );

        solana_program::program::invoke(
            &update_stake_pool_balance_ix,
            &[
                self.pool_account.to_account_info(),
                withdraw_authority.to_account_info(),
                validator_list_account.to_account_info(),
                reserve_stake_account.to_account_info(),
                manager_fee_account.to_account_info(),
                self.pool_token_mint.to_account_info(),
                self.pool_token_program.to_account_info(),
            ],
        )?;

        Ok(())
    }

    /// This method only takes available amount of pool token,
    /// so burnt pool token amount may be less than requested pool token amount.
    ///
    /// returns [burnt_pool_token_amount, unstaked_sol_amount, deducted_pool_token_fee_amount]
    #[inline(never)]
    pub fn withdraw_sol(
        &self,
        // fixed
        withdraw_authority: &AccountInfo<'info>,
        reserve_stake_account: &AccountInfo<'info>,
        manager_fee_account: &AccountInfo<'info>,
        validator_list_account: &AccountInfo<'info>,
        clock: &AccountInfo<'info>,
        stake_history: &AccountInfo<'info>,
        stake_program: &AccountInfo<'info>,

        // variant
        to_sol_account: &AccountInfo<'info>,
        from_pool_token_account: &AccountInfo<'info>,
        from_pool_token_account_signer: &AccountInfo<'info>,
        from_pool_token_account_signer_seeds: &[&[&[u8]]],

        pool_token_amount: u64,
    ) -> Result<(u64, u64, u64)> {
        let pool_account_data = &self.get_pool_account_data()?;

        // first update stake pool balance
        self.update_stake_pool_balance_if_needed(
            pool_account_data,
            withdraw_authority,
            reserve_stake_account,
            manager_fee_account,
            validator_list_account,
        )?;

        let reserve_stake_account_data = &Self::deserialize_stake_account(reserve_stake_account)?;

        let to_sol_account_amount_before = to_sol_account.lamports();
        let pool_token_amount = Self::get_available_pool_token_amount_to_withdraw_sol(
            pool_account_data,
            reserve_stake_account,
            reserve_stake_account_data,
        )?
        .min(pool_token_amount);

        // Withdraw amount too small
        if pool_token_amount == 0 {
            return Ok((0, 0, 0));
        }

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

        let deducted_pool_token_fee_amount = pool_account_data
            .calc_pool_tokens_sol_withdrawal_fee(pool_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        msg!("UNSTAKE#spl: pool_token_mint={}, burnt_pool_token_amount={}, deducted_pool_token_fee_amount={}, to_sol_account_amount={}, unstaked_sol_amount={}", self.pool_token_mint.key(), pool_token_amount, deducted_pool_token_fee_amount, to_sol_account_amount, unstaked_sol_amount);

        Ok((
            pool_token_amount,
            unstaked_sol_amount,
            deducted_pool_token_fee_amount,
        ))
    }

    fn get_available_pool_token_amount_to_withdraw_sol(
        pool_account_data: &StakePool,
        reserve_stake_account: &AccountInfo,
        reserve_stake_account_data: &StakeStateV2,
    ) -> Result<u64> {
        if pool_account_data.sol_withdraw_authority.is_some() {
            return Ok(0);
        }
        let StakeStateV2::Initialized(meta) = reserve_stake_account_data else {
            return Err(ProgramError::from(StakePoolError::WrongStakeStake))?;
        };

        let reserved_sol_amount = reserve_stake_account.lamports();
        let minimum_reserved_sol_amount = spl_stake_pool::minimum_reserve_lamports(meta);
        let available_sol_amount = reserved_sol_amount.saturating_sub(minimum_reserved_sol_amount);

        let available_pool_token_amount_to_burn = crate::utils::get_proportional_amount(
            available_sol_amount,
            pool_account_data.pool_token_supply,
            pool_account_data.total_lamports,
        )?;

        // pool_token_amount = pool_token_burnt * (1 / 1 - f) = pool_token_burnt * (d / (d - n))
        let numerator = pool_account_data.sol_withdrawal_fee.numerator;
        let denominator = pool_account_data.sol_withdrawal_fee.denominator;
        crate::utils::get_proportional_amount(
            available_pool_token_amount_to_burn,
            denominator,
            denominator.saturating_sub(numerator),
        )
    }

    /// This method only takes available amount of pool token,
    /// so burnt pool token amount may be less than requested pool token amount.
    ///
    /// returns [burnt_pool_token_amount, unstaking_sol_amount, deducted_pool_token_fee_amount]
    #[inline(never)]
    pub fn withdraw_stake(
        &self,
        // fixed
        system_program: &Program<'info, System>,
        withdraw_authority: &AccountInfo<'info>,
        reserve_stake_account: &AccountInfo<'info>,
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
    ) -> Result<(u64, u64, u64)> {
        let pool_account_data = &self.get_pool_account_data()?;

        // first update stake pool balance
        self.update_stake_pool_balance_if_needed(
            pool_account_data,
            withdraw_authority,
            reserve_stake_account,
            manager_fee_account,
            validator_list_account,
        )?;

        let validator_stake_account_data =
            &Self::deserialize_stake_account(validator_stake_account)?;

        let pool_token_amount = Self::get_available_pool_token_amount_to_withdraw_stake(
            pool_account_data,
            validator_stake_account,
            validator_stake_account_data,
        )?
        .min(pool_token_amount);

        // withdraw amount too small
        if pool_token_amount == 0 {
            return Ok((0, 0, 0));
        }

        let payer_lamports_before = to_stake_account_rent_payer.lamports();

        // initialize `to_stake_account` first - will be used for split stake
        system_program.initialize_account(
            to_stake_account,
            to_stake_account_rent_payer, // payer is already signer so we don't need signer seeds
            to_stake_account_seeds,
            StakeStateV2::size_of(),
            None,
            &solana_program::stake::program::ID,
        )?;

        let rent_payed = payer_lamports_before - to_stake_account_rent_payer.lamports();

        let withdraw_stake_ix = spl_stake_pool::instruction::withdraw_stake(
            self.spl_stake_pool_program.key,
            self.pool_account.key,
            validator_list_account.key,
            withdraw_authority.key,
            validator_stake_account.key,
            to_stake_account.key,
            to_stake_account_withdraw_authority.key,
            from_pool_token_account_signer.key,
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

        let unstaking_sol_amount = to_stake_account.lamports().saturating_sub(rent_payed);

        // deactivate `to_stake_account` - since it's state is active now as
        // it has been splitted from active stake account

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

        let deducted_pool_token_fee_amount = pool_account_data
            .calc_pool_tokens_stake_withdrawal_fee(pool_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        msg!("UNSTAKE#spl: pool_token_mint={}, pool_token_amount={}, deducted_pool_token_fee_amount={}, unstaked_sol_amount=0, unstaking_sol_amount={}", self.pool_token_mint.key(), pool_token_amount, deducted_pool_token_fee_amount, unstaking_sol_amount);

        Ok((
            pool_token_amount,
            unstaking_sol_amount,
            deducted_pool_token_fee_amount,
        ))
    }

    /// https://github.com/solana-labs/solana-program-library/blob/master/stake-pool/program/src/processor.rs#L2792
    fn get_available_pool_token_amount_to_withdraw_stake(
        pool_account_data: &StakePool,
        validator_stake_account: &AccountInfo,
        validator_stake_account_data: &StakeStateV2,
    ) -> Result<u64> {
        let StakeStateV2::Stake(meta, stake, _) = validator_stake_account_data else {
            return Err(ProgramError::from(StakePoolError::WrongStakeStake))?;
        };

        let stake_minimum_delegation = solana_program::stake::tools::get_minimum_delegation()?;
        // minimum staked sol amount = minimum delegation + rent exempt fee
        let minimum_staked_sol_amount =
            spl_stake_pool::minimum_stake_lamports(meta, stake_minimum_delegation);
        let tolerance = pool_account_data
            .get_lamports_per_pool_token()
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        let minimum_staked_sol_amount_with_tolerance =
            minimum_staked_sol_amount.saturating_add(tolerance);

        let active_stake_sol_amount = validator_stake_account.lamports();
        let available_delegation_amount = stake
            .delegation
            .stake
            .saturating_sub(stake_minimum_delegation);

        // Here we just added tolerance for safety.
        // In fact, maximum available sol amount to withdraw = active stake amount - mimimum staked sol amount
        // https://github.com/solana-labs/solana-program-library/blob/master/stake-pool/program/src/processor.rs#L2903
        let available_sol_amount = active_stake_sol_amount
            .saturating_sub(minimum_staked_sol_amount_with_tolerance)
            .min(available_delegation_amount);

        // New stake account must also ensure minimum delegation condition.
        if available_sol_amount < stake_minimum_delegation {
            return Ok(0);
        }

        let available_pool_token_amount_to_burn = crate::utils::get_proportional_amount(
            available_sol_amount,
            pool_account_data.pool_token_supply,
            pool_account_data.total_lamports,
        )?;

        // pool_token_amount = pool_token_burnt * (1 / 1 - f) = pool_token_burnt * (d / (d - n))
        let numerator = pool_account_data.stake_withdrawal_fee.numerator;
        let denominator = pool_account_data.stake_withdrawal_fee.denominator;
        crate::utils::get_proportional_amount(
            available_pool_token_amount_to_burn,
            denominator,
            denominator.saturating_sub(numerator),
        )
    }

    #[inline(never)]
    pub fn get_claimable_stake_accounts(
        // fixed
        clock: &AccountInfo,
        stake_history: &AccountInfo,

        // variant
        stake_accounts: impl Iterator<Item = &'info AccountInfo<'info>>,
    ) -> Result<Vec<&'info Pubkey>> {
        let clock = Clock::from_account_info(clock)?;
        let stake_history = StakeHistory::from_account_info(stake_history)?;

        stake_accounts
            .map(move |stake_account| {
                let stake_account_data = &Self::deserialize_stake_account(stake_account)?;
                Ok(
                    Self::is_stake_account_withdrawable(
                        stake_account_data,
                        &clock,
                        &stake_history,
                    )?
                    .then_some(stake_account.key),
                )
            })
            .filter_map(Result::transpose)
            .collect()
    }

    /// returns [claimed_sol_amount]
    #[inline(never)]
    pub fn claim_sol(
        // just for logging
        pool_token_mint: &Pubkey,

        // fixed
        system_program: &Program<'info, System>,
        clock: &AccountInfo<'info>,
        stake_history: &AccountInfo<'info>,
        stake_program: &AccountInfo<'info>,

        // variant
        to_sol_account: &AccountInfo<'info>,
        to_sol_account_seeds: &[&[&[u8]]],
        from_stake_account: &AccountInfo<'info>,
        from_stake_account_rent_refund_account: &AccountInfo<'info>,
        from_stake_account_withdraw_authority: &AccountInfo<'info>,
        from_stake_account_withdraw_authority_seeds: &[&[&[u8]]],
    ) -> Result<u64> {
        let stake_account_data = &Self::deserialize_stake_account(from_stake_account)?;

        let to_sol_account_amount_before = to_sol_account.lamports();
        #[allow(clippy::unwrap_used)] // withdrawable stake account always have meta
        let from_stake_account_rent = stake_account_data.meta().unwrap().rent_exempt_reserve;
        let unstaked_sol_amount = from_stake_account.lamports() - from_stake_account_rent;

        let withdraw_ix = solana_program::stake::instruction::withdraw(
            from_stake_account.key,
            from_stake_account_withdraw_authority.key,
            to_sol_account.key,
            from_stake_account.lamports(),
            None,
        );

        solana_program::program::invoke_signed(
            &withdraw_ix,
            &[
                from_stake_account.to_account_info(),
                from_stake_account_withdraw_authority.to_account_info(),
                to_sol_account.to_account_info(),
                clock.to_account_info(),
                stake_history.to_account_info(),
                stake_program.to_account_info(),
            ],
            from_stake_account_withdraw_authority_seeds,
        )?;

        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: to_sol_account.to_account_info(),
                    to: from_stake_account_rent_refund_account.to_account_info(),
                },
                to_sol_account_seeds,
            ),
            from_stake_account_rent,
        )?;

        let to_sol_account_amount = to_sol_account.lamports();
        let claimed_sol_amount = to_sol_account_amount - to_sol_account_amount_before;

        require_eq!(claimed_sol_amount, unstaked_sol_amount);

        msg!(
            "CLAIM_UNSTAKED#spl: pool_token_mint={}, to_sol_account_amount={}, claimed_sol_amount={}",
            pool_token_mint,
            to_sol_account_amount,
            claimed_sol_amount
        );

        Ok(claimed_sol_amount)
    }

    /// ref: https://github.com/anza-xyz/agave/blob/master/programs/stake/src/stake_state.rs#L822
    fn is_stake_account_withdrawable(
        stake_account_data: &StakeStateV2,
        clock: &Clock,
        stake_history: &StakeHistory,
    ) -> Result<bool> {
        let StakeStateV2::Stake(_, stake, _) = stake_account_data else {
            return Err(ProgramError::from(StakePoolError::WrongStakeStake))?;
        };

        // Runtime feature GwtDQBghCTBgmX2cpEGNPxTEBUTQRaDMGTr5qychdGMj has been activated since
        // epoch 565 (mainnet), epoch 584 (devnet), and 0 (local).
        // Since we know that this feature is activated in every cluster,
        // we just use epoch 0 as activated epoch.
        let new_rate_activation_epoch = Some(0);
        // if we have a deactivation epoch and we're in cooldown
        let staked = if clock.epoch >= stake.delegation.deactivation_epoch {
            stake
                .delegation
                .stake(clock.epoch, stake_history, new_rate_activation_epoch)
        } else {
            // Assume full stake if the stake account hasn't been
            // de-activated, because in the future the exposed stake
            // might be higher than stake.stake() due to warmup
            stake.delegation.stake
        };

        Ok(staked == 0)
    }
}
