use anchor_lang::{prelude::*, solana_program};
use anchor_spl::{
    token::Token,
    token_interface::{Mint, TokenAccount},
};
use marinade_cpi::{program::MarinadeFinance, LiqPool, State, TicketAccountData};

use crate::constants::{DEVNET_MSOL_MINT_ADDRESS, MAINNET_MSOL_MINT_ADDRESS};
use crate::utils::AccountInfoExt;
use crate::{errors, utils::SystemProgramExt};

pub struct MarinadeStakePoolService<'info> {
    marinade_stake_pool_program: Program<'info, MarinadeFinance>,
    pool_account: Account<'info, State>,
    pool_token_mint: InterfaceAccount<'info, Mint>,
    pool_token_program: Program<'info, Token>,
}

impl<'info> MarinadeStakePoolService<'info> {
    #[inline(never)]
    pub fn is_claimable_ticket_account(
        &self,
        clock: &AccountInfo<'info>,
        ticket_account: &'info AccountInfo<'info>,
        reserve_pda: &'info AccountInfo<'info>,
    ) -> Result<bool> {
        if !ticket_account.is_initialized() {
            return Ok(false);
        }

        let clock = Clock::from_account_info(clock)?;
        let ticket_account = Account::<TicketAccountData>::try_from(ticket_account)?;

        if clock.epoch < ticket_account.created_epoch + 1 {
            return Ok(false);
        } else if clock.epoch == ticket_account.created_epoch + 1
            && clock.unix_timestamp - clock.epoch_start_timestamp < 30 * 60
        {
            return Ok(false);
        }

        if ticket_account.lamports_amount
            > reserve_pda.lamports() - self.pool_account.rent_exempt_for_token_acc
        {
            return Ok(false);
        }

        Ok(true)
    }

    #[inline(never)]
    pub fn new(
        marinade_stake_pool_program: &'info AccountInfo<'info>,
        pool_account: &'info AccountInfo<'info>,
        pool_token_mint: &'info AccountInfo<'info>,
        pool_token_program: &'info AccountInfo<'info>,
    ) -> Result<Self> {
        let pool_account = Self::deserialize_pool_account(pool_account)?;

        Ok(Self {
            pool_account,
            marinade_stake_pool_program: Program::try_from(marinade_stake_pool_program)?,
            pool_token_mint: InterfaceAccount::try_from(pool_token_mint)?,
            pool_token_program: Program::try_from(pool_token_program)?,
        })
    }

    pub(super) fn deserialize_pool_account(
        pool_account: &'info AccountInfo<'info>,
    ) -> Result<Account<State>> {
        let pool_account = Account::<State>::try_from(pool_account)?;
        #[cfg(feature = "devnet")]
        require_eq!(pool_account.msol_mint, DEVNET_MSOL_MINT_ADDRESS);
        #[cfg(not(feature = "devnet"))]
        require_eq!(pool_account.msol_mint, MAINNET_MSOL_MINT_ADDRESS);

        Ok(pool_account)
    }

    fn find_pool_account_derived_address(
        pool_account: &AccountInfo,
        seed: &'static [u8],
    ) -> Pubkey {
        Pubkey::find_program_address(&[pool_account.key.as_ref(), seed], &marinade_cpi::ID).0
    }

    /// returns (pubkey, writable) of [pool_program, pool_account, pool_token_mint, pool_token_program, system_program]
    fn find_accounts_to_new(pool_account: &Account<State>) -> Vec<(Pubkey, bool)> {
        vec![
            (marinade_cpi::ID, false),
            (pool_account.key(), true),
            (pool_account.msol_mint, true),
            (Token::id(), false),
            (System::id(), false),
        ]
    }

    /// returns (pubkey, writable) of [pool_program, pool_account, pool_token_mint, pool_token_program, system_program, liq_pool_sol_leg, liq_pool_token_leg, liq_pool_token_leg_authority, pool_reserve, pool_token_mint_authority]
    #[inline(never)]
    pub fn find_accounts_to_deposit_sol(
        pool_account: &'info AccountInfo<'info>,
    ) -> Result<Vec<(Pubkey, bool)>> {
        let pool_account = Self::deserialize_pool_account(pool_account)?;
        let mut accounts = Self::find_accounts_to_new(&pool_account);
        accounts.extend([
            // liq_pool_sol_leg
            (
                Self::find_pool_account_derived_address(pool_account.as_ref(), b"liq_sol"),
                true,
            ),
            // liq_pool_mint_leg
            (pool_account.liq_pool.msol_leg, true),
            // liq_pool_mint_leg_authority
            (
                Self::find_pool_account_derived_address(
                    pool_account.as_ref(),
                    b"liq_st_sol_authority",
                ),
                false,
            ),
            // pool_reserve
            Self::find_pool_reserve_account_meta(pool_account.as_ref()),
            // pool_mint_authority
            (
                Self::find_pool_account_derived_address(pool_account.as_ref(), b"st_mint"),
                false,
            ),
        ]);
        Ok(accounts)
    }

    pub fn get_min_deposit_sol_amount(pool_account: &'info AccountInfo<'info>) -> Result<u64> {
        let pool_account = Self::deserialize_pool_account(pool_account)?;
        Ok(pool_account.min_deposit)
    }

    /// returns [to_pool_token_account_amount, minted_pool_token_amount, deducted_sol_fee_amount]
    #[inline(never)]
    pub fn deposit_sol(
        &mut self,
        // fixed
        system_program: &Program<'info, System>,
        liq_pool_sol_leg: &AccountInfo<'info>,
        liq_pool_token_leg: &AccountInfo<'info>,
        liq_pool_token_leg_authority: &AccountInfo<'info>,
        pool_reserve: &AccountInfo<'info>,
        pool_token_mint_authority: &AccountInfo<'info>,

        // variant
        from_sol_account: &AccountInfo<'info>,
        to_pool_token_account: &'info AccountInfo<'info>,
        from_sol_account_seeds: &[&[u8]],

        sol_amount: u64,
    ) -> Result<(u64, u64, u64)> {
        let mut to_pool_token_account =
            InterfaceAccount::<TokenAccount>::try_from(to_pool_token_account)?;
        let to_pool_token_account_amount_before = to_pool_token_account.amount;

        marinade_cpi::cpi::deposit(
            CpiContext::new_with_signer(
                self.marinade_stake_pool_program.to_account_info(),
                marinade_cpi::cpi::accounts::Deposit {
                    state: self.pool_account.to_account_info(),
                    msol_mint: self.pool_token_mint.to_account_info(),
                    liq_pool_sol_leg_pda: liq_pool_sol_leg.clone(),
                    liq_pool_msol_leg: liq_pool_token_leg.clone(),
                    liq_pool_msol_leg_authority: liq_pool_token_leg_authority.clone(),
                    reserve_pda: pool_reserve.clone(),
                    transfer_from: from_sol_account.clone(),
                    mint_to: to_pool_token_account.to_account_info(),
                    msol_mint_authority: pool_token_mint_authority.clone(),
                    system_program: system_program.to_account_info(),
                    token_program: self.pool_token_program.to_account_info(),
                },
                &[from_sol_account_seeds],
            ),
            sol_amount,
        )?;

        to_pool_token_account.reload()?;
        let to_pool_token_account_amount = to_pool_token_account.amount;
        let minted_pool_token_amount =
            to_pool_token_account_amount - to_pool_token_account_amount_before;

        let deducted_sol_fee_amount = 0;

        msg!("STAKE#marinade: pool_token_mint={}, staked_sol_amount={}, to_pool_token_account_amount={}, minted_pool_token_amount={}, deducted_sol_fee_amount={}", self.pool_token_mint.key(), sol_amount, to_pool_token_account_amount, minted_pool_token_amount, deducted_sol_fee_amount);

        Ok((
            to_pool_token_account_amount,
            minted_pool_token_amount,
            deducted_sol_fee_amount,
        ))
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

    /// returns unstaking_sol_amount
    #[inline(never)]
    pub fn order_unstake(
        &mut self,
        // fixed
        system_program: &Program<'info, System>,

        new_ticket_account: &'info AccountInfo<'info>,
        new_ticket_account_seeds: &[&[u8]],
        clock: &AccountInfo<'info>,
        rent: &AccountInfo<'info>,

        // variant
        ticket_account_rent_payer: &Signer<'info>,
        from_pool_token_account: &AccountInfo<'info>,
        from_pool_token_account_authority: &AccountInfo<'info>,
        from_pool_token_account_authority_seeds: &[&[u8]],

        token_amount: u64,
    ) -> Result<u64> {
        self.create_ticket_account(
            system_program,
            ticket_account_rent_payer,
            new_ticket_account,
            new_ticket_account_seeds,
        )?;

        marinade_cpi::cpi::order_unstake(
            CpiContext::new_with_signer(
                self.marinade_stake_pool_program.to_account_info(),
                marinade_cpi::cpi::accounts::OrderUnstake {
                    state: self.pool_account.to_account_info(),
                    msol_mint: self.pool_token_mint.to_account_info(),
                    burn_msol_from: from_pool_token_account.to_account_info(),
                    burn_msol_authority: from_pool_token_account_authority.to_account_info(),
                    new_ticket_account: new_ticket_account.to_account_info(),
                    clock: clock.to_account_info(),
                    rent: rent.to_account_info(),
                    token_program: self.pool_token_program.to_account_info(),
                },
                &[from_pool_token_account_authority_seeds],
            ),
            token_amount,
        )?;

        let ticket_account = Account::<TicketAccountData>::try_from(new_ticket_account)?;
        Ok(ticket_account.lamports_amount)
    }

    fn create_ticket_account(
        &self,
        system_program: &Program<'info, System>,

        ticket_account_rent_payer: &Signer<'info>,
        new_ticket_account: &AccountInfo<'info>,
        new_ticket_account_seeds: &[&[u8]],
    ) -> Result<()> {
        let space = 8 + std::mem::size_of::<TicketAccountData>();
        system_program.create_account(
            new_ticket_account,
            new_ticket_account_seeds,
            ticket_account_rent_payer,
            &[],
            space,
            &crate::ID,
        )
    }

    /// returns unstaked_sol_amount
    #[inline(never)]
    pub fn claim(
        &mut self,
        system_program: &Program<'info, System>,

        pool_reserve_account: &'info AccountInfo<'info>,
        clock: &AccountInfo<'info>,
        ticket_account: &'info AccountInfo<'info>,

        to_sol_account: &AccountInfo<'info>,
        to_sol_account_seeds: &[&[u8]],
        rent_refund_account: &AccountInfo<'info>, // receive rent of ticket account
    ) -> Result<u64> {
        let ticket_account = Account::<TicketAccountData>::try_from(ticket_account)?;
        let ticket_account_rent = ticket_account.get_lamports();
        let unstaked_sol_amount = ticket_account.lamports_amount;

        marinade_cpi::cpi::claim(CpiContext::new(
            self.marinade_stake_pool_program.to_account_info(),
            marinade_cpi::cpi::accounts::Claim {
                state: self.pool_account.to_account_info(),
                reserve_pda: pool_reserve_account.clone(),
                ticket_account: ticket_account.to_account_info(),
                transfer_sol_to: to_sol_account.clone(),
                clock: clock.clone(),
                system_program: system_program.to_account_info(),
            },
        ))?;

        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: to_sol_account.to_account_info(),
                    to: rent_refund_account.to_account_info(),
                },
                &[to_sol_account_seeds],
            ),
            ticket_account_rent,
        )?;

        Ok(unstaked_sol_amount)
    }

    #[inline(never)]
    pub fn find_accounts_to_order_unstake(
        pool_account: &'info AccountInfo<'info>,
        ticket_account: &AccountInfo,
    ) -> Result<Vec<(Pubkey, bool)>> {
        let pool_account = Self::deserialize_pool_account(pool_account)?;
        let mut accounts = Self::find_accounts_to_new(&pool_account);
        accounts.extend([
            // new_ticket_account
            (ticket_account.key(), true),
            // clock
            (solana_program::sysvar::clock::ID, false),
            // rent
            (solana_program::sysvar::rent::ID, false),
        ]);
        Ok(accounts)
    }

    #[inline(never)]
    pub fn find_accounts_to_claim<'a>(
        pool_account_info: &'info AccountInfo<'info>,
        ticket_accounts: impl IntoIterator<Item = &'a AccountInfo<'info>>,
    ) -> Result<Vec<(Pubkey, bool)>>
    where
        'info: 'a,
    {
        let pool_account = Account::<State>::try_from(pool_account_info)?;
        let mut accounts = Self::find_accounts_to_new(&pool_account);
        accounts.extend([
            // pool_reserve
            Self::find_pool_reserve_account_meta(pool_account.as_ref()),
            // clock
            (solana_program::sysvar::clock::ID, false),
        ]);
        accounts.extend(
            ticket_accounts
                .into_iter()
                .map(|account| (account.key(), true)),
        );
        Ok(accounts)
    }

    fn find_pool_reserve_account_meta(pool_account: &AccountInfo) -> (Pubkey, bool) {
        (
            Self::find_pool_account_derived_address(pool_account, b"reserve"),
            true,
        )
    }
}
