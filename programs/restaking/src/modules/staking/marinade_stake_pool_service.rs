use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount};
use marinade_cpi::marinade::accounts::{State, TicketAccountData};
use marinade_cpi::marinade::program::MarinadeFinance;

use crate::utils::SystemProgramExt;

pub(in crate::modules) struct MarinadeStakePoolService<'info> {
    marinade_stake_pool_program: Program<'info, MarinadeFinance>,
    pool_account: Account<'info, State>,
    pool_token_mint: InterfaceAccount<'info, Mint>,
    pool_token_program: Program<'info, Token>,
}

impl<'info> MarinadeStakePoolService<'info> {
    #[inline(never)]
    pub fn new(
        marinade_stake_pool_program: &'info AccountInfo<'info>,
        pool_account: &'info AccountInfo<'info>,
        pool_token_mint: &'info AccountInfo<'info>,
        pool_token_program: &'info AccountInfo<'info>,
    ) -> Result<Self> {
        let pool_account = Self::deserialize_pool_account(pool_account)?;

        require_keys_eq!(pool_account.msol_mint, pool_token_mint.key());
        require_keys_eq!(*pool_token_mint.owner, *pool_token_program.key);

        Ok(Self {
            pool_account,
            marinade_stake_pool_program: Program::try_from(marinade_stake_pool_program)?,
            pool_token_mint: InterfaceAccount::try_from(pool_token_mint)?,
            pool_token_program: Program::try_from(pool_token_program)?,
        })
    }

    #[inline(always)]
    pub(super) fn deserialize_pool_account(
        pool_account: &'info AccountInfo<'info>,
    ) -> Result<Account<'info, State>> {
        Account::try_from(pool_account)
    }

    #[inline(always)]
    fn deserialize_withdrawal_ticket_account(
        withdrawal_ticket_account: &'info AccountInfo<'info>,
    ) -> Result<Account<'info, TicketAccountData>> {
        Account::try_from(withdrawal_ticket_account)
    }

    fn find_pool_account_derived_address<'a>(
        pool_account: &impl AsRef<AccountInfo<'a>>,
        seed: &'static [u8],
    ) -> Pubkey {
        Pubkey::find_program_address(
            &[pool_account.as_ref().key.as_ref(), seed],
            &marinade_cpi::marinade::ID,
        )
        .0
    }

    /// * pool_reserve_account(writable)
    fn find_pool_reserve_account_meta<'a>(
        pool_account: &impl AsRef<AccountInfo<'a>>,
    ) -> (Pubkey, bool) {
        (
            Self::find_pool_account_derived_address(pool_account, b"reserve"),
            true,
        )
    }

    /// * pool_program
    /// * pool_account(writable)
    /// * pool_token_mint(writable)
    /// * pool_token_program
    fn find_accounts_to_new(pool_account: &Account<State>) -> [(Pubkey, bool); 4] {
        [
            (marinade_cpi::marinade::ID, false),
            (pool_account.key(), true),
            (pool_account.msol_mint, true),
            (Token::id(), false),
        ]
    }

    /// * (0) pool_program
    /// * (1) pool_account(writable)
    /// * (2) pool_token_mint(writable)
    /// * (3) pool_token_program
    /// * (4) liq_pool_sol_leg(writable)
    /// * (5) liq_pool_mint_leg(wirtable)
    /// * (6) liq_pool_mint_leg_authority
    /// * (7) pool_reserve_account(writable)
    /// * (8) pool_token_mint_authority
    #[inline(never)]
    pub fn find_accounts_to_deposit_sol(
        pool_account: &'info AccountInfo<'info>,
    ) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        let pool_account = &Self::deserialize_pool_account(pool_account)?;

        let accounts = Self::find_accounts_to_new(pool_account).into_iter().chain([
            // liq_pool_sol_leg(writable)
            (
                Self::find_pool_account_derived_address(pool_account, b"liq_sol"),
                true,
            ),
            // liq_pool_mint_leg(writable)
            (pool_account.liq_pool.msol_leg, true),
            // liq_pool_mint_leg_authority
            (
                Self::find_pool_account_derived_address(pool_account, b"liq_st_sol_authority"),
                false,
            ),
            // pool_reserve(writable)
            Self::find_pool_reserve_account_meta(pool_account),
            // pool_token_mint_authority
            (
                Self::find_pool_account_derived_address(pool_account, b"st_mint"),
                false,
            ),
        ]);

        Ok(accounts)
    }

    /// * (0) pool_program
    /// * (1) pool_account(writable)
    /// * (2) pool_token_mint(writable)
    /// * (3) pool_token_program
    /// * (4) sysvar clock
    /// * (5) sysvar rent
    #[inline(never)]
    pub fn find_accounts_to_order_unstake(
        pool_account: &'info AccountInfo<'info>,
    ) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        let pool_account = &Self::deserialize_pool_account(pool_account)?;

        let accounts = Self::find_accounts_to_new(pool_account).into_iter().chain([
            // sysvar clock
            (solana_program::sysvar::clock::ID, false),
            // sysvar rent
            (solana_program::sysvar::rent::ID, false),
        ]);

        Ok(accounts)
    }

    /// * (0) pool_program
    /// * (1) pool_account(writable)
    /// * (2) pool_token_mint(writable)
    /// * (3) pool_token_program
    /// * (4) pool_reserve_account(writable)
    /// * (5) sysvar clock
    #[inline(never)]
    pub fn find_accounts_to_claim_sol(
        pool_account: &'info AccountInfo<'info>,
    ) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        let pool_account = &Self::deserialize_pool_account(pool_account)?;

        let accounts = Self::find_accounts_to_new(pool_account).into_iter().chain([
            // pool_reserve_account(writable)
            Self::find_pool_reserve_account_meta(pool_account),
            // sysvar clock
            (solana_program::sysvar::clock::ID, false),
        ]);

        Ok(accounts)
    }

    pub fn get_min_deposit_sol_amount(&self) -> u64 {
        self.pool_account.min_deposit
    }

    /// returns [to_pool_token_account_amount, minted_pool_token_amount] (no fee)
    #[inline(never)]
    pub fn deposit_sol(
        &self,
        // fixed
        system_program: &Program<'info, System>,
        liq_pool_sol_leg: &AccountInfo<'info>,
        liq_pool_token_leg: &AccountInfo<'info>,
        liq_pool_token_leg_authority: &AccountInfo<'info>,
        pool_reserve_account: &AccountInfo<'info>,
        pool_token_mint_authority: &AccountInfo<'info>,

        // variant
        to_pool_token_account: &'info AccountInfo<'info>,
        from_sol_account: &AccountInfo<'info>,
        from_sol_account_seeds: &[&[&[u8]]],

        sol_amount: u64,
    ) -> Result<(u64, u64)> {
        let mut to_pool_token_account =
            InterfaceAccount::<TokenAccount>::try_from(to_pool_token_account)?;
        let to_pool_token_account_amount_before = to_pool_token_account.amount;

        marinade_cpi::marinade::cpi::deposit(
            CpiContext::new_with_signer(
                self.marinade_stake_pool_program.to_account_info(),
                marinade_cpi::marinade::cpi::accounts::Deposit {
                    state: self.pool_account.to_account_info(),
                    msol_mint: self.pool_token_mint.to_account_info(),
                    liq_pool_sol_leg_pda: liq_pool_sol_leg.to_account_info(),
                    liq_pool_msol_leg: liq_pool_token_leg.to_account_info(),
                    liq_pool_msol_leg_authority: liq_pool_token_leg_authority.to_account_info(),
                    reserve_pda: pool_reserve_account.to_account_info(),
                    transfer_from: from_sol_account.to_account_info(),
                    mint_to: to_pool_token_account.to_account_info(),
                    msol_mint_authority: pool_token_mint_authority.to_account_info(),
                    system_program: system_program.to_account_info(),
                    token_program: self.pool_token_program.to_account_info(),
                },
                from_sol_account_seeds,
            ),
            sol_amount,
        )?;

        to_pool_token_account.reload()?;
        let to_pool_token_account_amount = to_pool_token_account.amount;
        let minted_pool_token_amount =
            to_pool_token_account_amount - to_pool_token_account_amount_before;

        msg!("STAKE#marinade: pool_token_mint={}, staked_sol_amount={}, to_pool_token_account_amount={}, minted_pool_token_amount={}", self.pool_token_mint.key(), sol_amount, to_pool_token_account_amount, minted_pool_token_amount);

        Ok((to_pool_token_account_amount, minted_pool_token_amount))
    }

    /// gives max fee/expense ratio during a cycle of circulation
    /// returns (numerator, denominator)
    #[inline(never)]
    pub fn get_max_cycle_fee(pool_account: &'info AccountInfo<'info>) -> Result<(u64, u64)> {
        let pool_account = Self::deserialize_pool_account(pool_account)?;

        // it only costs withdrawal fee
        Ok((
            // ref: https://github.com/marinade-finance/liquid-staking-program/blob/main/programs/marinade-finance/src/state/fee.rs
            pool_account
                .withdraw_stake_account_fee
                .bp_cents
                .max(pool_account.delayed_unstake_fee.bp_cents) as u64,
            1_000_000,
        ))
    }

    /// returns [unstaking_sol_amount, deducted_sol_fee_amount]
    #[inline(never)]
    pub fn order_unstake(
        &self,
        // fixed
        system_program: &Program<'info, System>,
        clock: &AccountInfo<'info>,
        rent: &AccountInfo<'info>,

        // variant
        new_withdrawal_ticket_account: &'info AccountInfo<'info>,
        new_withdrawal_ticket_account_seeds: &[&[&[u8]]],
        new_withdrawal_ticket_account_rent_payer: &Signer<'info>,
        from_pool_token_account: &AccountInfo<'info>,
        from_pool_token_account_signer: &AccountInfo<'info>,
        from_pool_token_account_signer_seeds: &[&[&[u8]]],

        pool_token_amount: u64,
    ) -> Result<(u64, u64)> {
        system_program.initialize_account(
            new_withdrawal_ticket_account,
            new_withdrawal_ticket_account_rent_payer, // already signer so we don't need signer seeds
            new_withdrawal_ticket_account_seeds,
            8 + std::mem::size_of::<TicketAccountData>(),
            None,
            &marinade_cpi::marinade::ID,
        )?;

        marinade_cpi::marinade::cpi::order_unstake(
            CpiContext::new_with_signer(
                self.marinade_stake_pool_program.to_account_info(),
                marinade_cpi::marinade::cpi::accounts::OrderUnstake {
                    state: self.pool_account.to_account_info(),
                    msol_mint: self.pool_token_mint.to_account_info(),
                    burn_msol_from: from_pool_token_account.to_account_info(),
                    burn_msol_authority: from_pool_token_account_signer.to_account_info(),
                    new_ticket_account: new_withdrawal_ticket_account.to_account_info(),
                    clock: clock.to_account_info(),
                    rent: rent.to_account_info(),
                    token_program: self.pool_token_program.to_account_info(),
                },
                from_pool_token_account_signer_seeds,
            ),
            pool_token_amount,
        )?;

        let withdrawal_ticket_account =
            Self::deserialize_withdrawal_ticket_account(new_withdrawal_ticket_account)?;
        let unstaking_sol_amount = withdrawal_ticket_account.lamports_amount;
        let deducted_sol_fee_amount = {
            let fee_numerator = self.pool_account.delayed_unstake_fee.bp_cents;
            let fee_denominator = 1_000_000 - fee_numerator;
            crate::utils::get_proportional_amount(
                unstaking_sol_amount,
                fee_numerator as u64,
                fee_denominator as u64,
            )?
        };

        msg!("UNSTAKE#marinade: pool_token_mint={}, pool_token_amount={}, deducted_sol_fee_amount={}, unstaked_sol_amount={}", self.pool_token_mint.key(), pool_token_amount, deducted_sol_fee_amount, unstaking_sol_amount);

        Ok((unstaking_sol_amount, deducted_sol_fee_amount))
    }

    /// When ticket beneficiary is a program owned account, you don't need seeds.
    ///
    /// returns [claimed_sol_amount]
    #[inline(never)]
    pub fn claim_sol(
        &self,
        // fixed
        system_program: &Program<'info, System>,
        pool_reserve_account: &AccountInfo<'info>,
        clock: &AccountInfo<'info>,

        // variant
        withdrawal_ticket_account: &'info AccountInfo<'info>,
        withdrawal_ticket_account_rent_refund_account: &AccountInfo<'info>, // receive rent of ticket account
        withdrawal_ticket_account_beneficiary: &AccountInfo<'info>,
        withdrawal_ticket_account_beneficiary_seeds: &[&[&[u8]]],
    ) -> Result<u64> {
        let withdrawal_ticket_account =
            &Self::deserialize_withdrawal_ticket_account(withdrawal_ticket_account)?;

        // Withdrawal ticket is not claimable yet
        if !self.is_withdrawal_ticket_claimable(
            pool_reserve_account,
            &Clock::from_account_info(clock)?,
            withdrawal_ticket_account,
        ) {
            return Ok(0);
        }

        let withdrawal_ticket_account_beneficiary_amount_before =
            withdrawal_ticket_account_beneficiary.lamports();
        let withdrawal_ticket_account_rent = withdrawal_ticket_account.get_lamports();
        let unstaked_sol_amount = withdrawal_ticket_account.lamports_amount;

        marinade_cpi::marinade::cpi::claim(CpiContext::new(
            self.marinade_stake_pool_program.to_account_info(),
            marinade_cpi::marinade::cpi::accounts::Claim {
                state: self.pool_account.to_account_info(),
                reserve_pda: pool_reserve_account.to_account_info(),
                ticket_account: withdrawal_ticket_account.to_account_info(),
                transfer_sol_to: withdrawal_ticket_account_beneficiary.to_account_info(),
                clock: clock.to_account_info(),
                system_program: system_program.to_account_info(),
            },
        ))?;

        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: withdrawal_ticket_account_beneficiary.to_account_info(),
                    to: withdrawal_ticket_account_rent_refund_account.to_account_info(),
                },
                withdrawal_ticket_account_beneficiary_seeds,
            ),
            withdrawal_ticket_account_rent,
        )?;

        let withdrawal_ticket_account_beneficiary_amount =
            withdrawal_ticket_account_beneficiary.lamports();
        let claimed_sol_amount = withdrawal_ticket_account_beneficiary_amount
            - withdrawal_ticket_account_beneficiary_amount_before;

        require_eq!(claimed_sol_amount, unstaked_sol_amount);

        msg!(
            "CLAIM#marinade: pool_token_mint={}, to_sol_account_amount={}, claimed_sol_amount={}",
            self.pool_token_mint.key(),
            withdrawal_ticket_account_beneficiary_amount,
            claimed_sol_amount,
        );

        Ok(claimed_sol_amount)
    }

    fn is_withdrawal_ticket_claimable(
        &self,
        pool_reserve_account: &AccountInfo,
        clock: &Clock,
        withdrawal_ticket_account: &Account<TicketAccountData>,
    ) -> bool {
        // At least one epoch should pass.
        if clock.epoch < withdrawal_ticket_account.created_epoch + 1 {
            return false;
        }

        // Even when one epoch has passed, additional 30 min should pass.
        if clock.epoch == withdrawal_ticket_account.created_epoch + 1
            && clock.unix_timestamp - clock.epoch_start_timestamp < 30 * 60
        {
            return false;
        }

        // There should be enough lamports in pool reserve account.
        if withdrawal_ticket_account.lamports_amount
            < pool_reserve_account.lamports() - self.pool_account.rent_exempt_for_token_acc
        {
            return false;
        }

        true
    }
}
