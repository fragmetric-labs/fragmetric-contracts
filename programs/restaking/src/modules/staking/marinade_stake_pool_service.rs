use anchor_lang::{prelude::*, solana_program};
use anchor_spl::{
    token::Token,
    token_interface::{Mint, TokenAccount},
};
use marinade_cpi::{program::MarinadeFinance, LiqPool, State, TicketAccountData};

use crate::errors;
use crate::utils::AccountInfoExt;

pub struct MarinadeStakePoolService<'info> {
    marinade_stake_pool_program: Program<'info, MarinadeFinance>,
    pool_account: Account<'info, State>,
    pool_token_mint: InterfaceAccount<'info, Mint>,
    pool_token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

impl<'info> MarinadeStakePoolService<'info> {
    const TICKET_ACCOUNT_SEED: &'static [u8] = b"marinade_ticket_account";

    fn get_ticket_account_seeds<'a>(pool_account: &'a AccountInfo, index: &'a u8) -> [&'a [u8]; 3] {
        [
            Self::TICKET_ACCOUNT_SEED,
            pool_account.key.as_ref(),
            std::slice::from_ref(index),
        ]
    }

    fn find_ticket_account_address<'a>(pool_account: &'a AccountInfo, index: u8) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &Self::get_ticket_account_seeds(pool_account, &index),
            &crate::ID,
        )
    }

    fn get_ticket_account_signer_seeds<'a>(
        pool_account: &'a AccountInfo,
        index: &'a u8,
        bump: &'a u8,
    ) -> [&'a [u8]; 4] {
        let mut signer_seeds = [b"".as_slice(); 4];
        signer_seeds.copy_from_slice(&Self::get_ticket_account_seeds(pool_account, index));
        signer_seeds[3] = std::slice::from_ref(bump);
        signer_seeds
    }

    pub(in crate::modules) fn get_uninitialized_ticket_account<'a>(
        ticket_accounts: impl IntoIterator<Item = &'a AccountInfo<'info>>,
    ) -> Result<Pubkey>
    where
        'info: 'a,
    {
        for ticket_account in ticket_accounts {
            if !ticket_account.is_initialized() {
                return Ok(ticket_account.key());
            }
        }

        err!(errors::ErrorCode::StakingUninitializedWithdrawTicketNotFoundException)?
    }

    #[inline(never)]
    pub(in crate::modules) fn is_claimable_ticket_account(
        &mut self,
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
        system_program: &'info AccountInfo<'info>,
    ) -> Result<Box<Self>> {
        let pool_account = Account::<State>::try_from(pool_account)?;
        require_keys_eq!(pool_token_mint.key(), pool_account.msol_mint);

        Ok(Box::new(Self {
            pool_account,
            marinade_stake_pool_program: Program::try_from(marinade_stake_pool_program)?,
            pool_token_mint: InterfaceAccount::try_from(pool_token_mint)?,
            pool_token_program: Program::try_from(pool_token_program)?,
            system_program: Program::try_from(system_program)?,
        }))
    }

    /// returns (to_pool_token_account_amount, minted_pool_token_amount)
    pub(in crate::modules) fn deposit(
        &mut self,
        liq_pool_sol_leg_pda: &AccountInfo<'info>,
        liq_pool_msol_leg: &AccountInfo<'info>,
        liq_pool_msol_leg_authority: &AccountInfo<'info>,
        reserve_pda: &AccountInfo<'info>,
        msol_mint_authority: &AccountInfo<'info>,

        from_sol_account: &AccountInfo<'info>,
        to_pool_token_account: &'info AccountInfo<'info>,
        from_sol_account_signer_seeds: &[&[u8]],

        sol_amount: u64,
    ) -> Result<(u64, u64)> {
        let mut to_pool_token_account =
            InterfaceAccount::<TokenAccount>::try_from(to_pool_token_account)?;
        let to_pool_token_account_amount_before = to_pool_token_account.amount;

        marinade_cpi::cpi::deposit(
            CpiContext::new_with_signer(
                self.marinade_stake_pool_program.to_account_info(),
                marinade_cpi::cpi::accounts::Deposit {
                    state: self.pool_account.to_account_info(),
                    msol_mint: self.pool_token_mint.to_account_info(),
                    liq_pool_sol_leg_pda: liq_pool_sol_leg_pda.clone(),
                    liq_pool_msol_leg: liq_pool_msol_leg.clone(),
                    liq_pool_msol_leg_authority: liq_pool_msol_leg_authority.clone(),
                    reserve_pda: reserve_pda.clone(),
                    transfer_from: from_sol_account.clone(),
                    mint_to: to_pool_token_account.to_account_info(),
                    msol_mint_authority: msol_mint_authority.clone(),
                    system_program: self.system_program.to_account_info(),
                    token_program: self.pool_token_program.to_account_info(),
                },
                &[from_sol_account_signer_seeds],
            ),
            sol_amount,
        )?;

        to_pool_token_account.reload()?;
        let to_pool_token_account_amount = to_pool_token_account.amount;
        let minted_pool_token_amount =
            to_pool_token_account_amount - to_pool_token_account_amount_before;

        Ok((to_pool_token_account_amount, minted_pool_token_amount))
    }

    /// returns
    pub(in crate::modules) fn order_unstake(
        &mut self,
        new_ticket_account: &AccountInfo<'info>,
        clock: &AccountInfo<'info>,
        rent: &AccountInfo<'info>,

        operator: &Signer<'info>,
        from_pool_token_account: &AccountInfo<'info>,
        from_pool_token_account_authority: &AccountInfo<'info>,
        from_pool_token_account_authority_signer_seeds: &[&[u8]],

        ticket_account_index: u8,
        token_amount: u64,
    ) -> Result<()> {
        self.create_ticket_account(operator, new_ticket_account, rent, ticket_account_index)?;

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
                &[from_pool_token_account_authority_signer_seeds],
            ),
            token_amount,
        )
    }

    fn create_ticket_account(
        &self,
        operator: &Signer<'info>,
        new_ticket_account: &AccountInfo<'info>,
        rent: &AccountInfo<'info>,
        ticket_account_index: u8,
    ) -> Result<()> {
        let (ticket_account_address, ticket_account_bump) =
            Self::find_ticket_account_address(self.pool_account.as_ref(), ticket_account_index);

        require_keys_eq!(new_ticket_account.key(), ticket_account_address);
        require!(
            !new_ticket_account.is_initialized(),
            ErrorCode::AccountOwnedByWrongProgram,
        );

        let space = 8 + std::mem::size_of::<TicketAccountData>();
        let lamports = Rent::from_account_info(rent)?.minimum_balance(space);
        anchor_lang::system_program::create_account(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                anchor_lang::system_program::CreateAccount {
                    from: operator.to_account_info(),
                    to: new_ticket_account.to_account_info(),
                },
                &[Self::get_ticket_account_signer_seeds(
                    self.pool_account.as_ref(),
                    &ticket_account_index,
                    &ticket_account_bump,
                )
                .as_ref()],
            ),
            lamports,
            space as u64,
            self.marinade_stake_pool_program.key,
        )
    }

    pub(in crate::modules) fn claim(
        &mut self,
        reserve_pda: &'info AccountInfo<'info>,
        clock: &AccountInfo<'info>,
        ticket_accounts: impl IntoIterator<Item = &'info AccountInfo<'info>>,

        to_sol_account: &AccountInfo<'info>,
    ) -> Result<()> {
        for ticket_account in ticket_accounts {
            if !self.is_claimable_ticket_account(clock, ticket_account, reserve_pda)? {
                continue;
            }

            marinade_cpi::cpi::claim(CpiContext::new(
                self.marinade_stake_pool_program.to_account_info(),
                marinade_cpi::cpi::accounts::Claim {
                    state: self.pool_account.to_account_info(),
                    reserve_pda: reserve_pda.clone(),
                    ticket_account: ticket_account.clone(),
                    transfer_sol_to: to_sol_account.clone(),
                    clock: clock.clone(),
                    system_program: self.system_program.to_account_info(),
                },
            ))?;
        }

        Ok(())
    }

    fn find_pool_account_related_address(
        pool_account: &AccountInfo,
        seed: &'static [u8],
    ) -> Pubkey {
        Pubkey::find_program_address(&[pool_account.key.as_ref(), seed], &marinade_cpi::ID).0
    }

    pub(in crate::modules) fn find_ticket_accounts(
        pool_account: &'info AccountInfo<'info>,
        is_writable: bool,
    ) -> impl Iterator<Item = (Pubkey, bool)> + 'info {
        (0..5).map(move |index| {
            (
                Self::find_ticket_account_address(pool_account, index).0,
                is_writable,
            )
        })
    }

    #[inline(never)]
    fn find_accounts_for_new(pool_account: &Account<State>) -> Vec<(Pubkey, bool)> {
        vec![
            (marinade_cpi::ID, false),
            (pool_account.key(), true),
            (pool_account.msol_mint, true),
            (Token::id(), false),
            (System::id(), false),
        ]
    }

    pub(in crate::modules) fn find_accounts_to_deposit(
        pool_account: &'info AccountInfo<'info>,
    ) -> Result<Vec<(Pubkey, bool)>> {
        let pool_account = Account::<State>::try_from(pool_account)?;
        let mut accounts = Self::find_accounts_for_new(&pool_account);
        accounts.extend([
            // liq_pool_sol_leg_pda
            (
                Self::find_pool_account_related_address(pool_account.as_ref(), b"liq_sol"),
                true,
            ),
            // liq_pool_msol_leg
            (pool_account.liq_pool.msol_leg, true),
            // liq_pool_msol_leg_authority
            (
                Self::find_pool_account_related_address(
                    pool_account.as_ref(),
                    b"liq_st_sol_authority",
                ),
                false,
            ),
            // reserve_pda
            Self::find_reserve_pda_account_meta(pool_account.as_ref()),
            // msol_mint_authority
            (
                Self::find_pool_account_related_address(pool_account.as_ref(), b"st_mint"),
                false,
            ),
        ]);
        Ok(accounts)
    }

    pub(in crate::modules) fn find_accounts_to_order_unstake<'a>(
        pool_account: &'info AccountInfo<'info>,
        ticket_accounts: impl IntoIterator<Item = &'a AccountInfo<'info>>,
    ) -> Result<Vec<(Pubkey, bool)>>
    where
        'info: 'a,
    {
        let pool_account = Account::<State>::try_from(pool_account)?;
        let mut accounts = Self::find_accounts_for_new(&pool_account);
        accounts.extend([
            // new_ticket_account
            (
                Self::get_uninitialized_ticket_account(ticket_accounts)?,
                true,
            ),
            // clock
            (solana_program::sysvar::clock::ID, false),
            // rent
            (solana_program::sysvar::rent::ID, false),
        ]);
        Ok(accounts)
    }

    pub(in crate::modules) fn find_accounts_to_claim(
        pool_account_info: &'info AccountInfo<'info>,
    ) -> Result<Vec<(Pubkey, bool)>> {
        let pool_account = Account::<State>::try_from(pool_account_info)?;
        let mut accounts = Self::find_accounts_for_new(&pool_account);
        accounts.extend([
            // reserve_pda
            Self::find_reserve_pda_account_meta(pool_account.as_ref()),
            // clock
            (solana_program::sysvar::clock::ID, false),
        ]);
        accounts.extend(Self::find_ticket_accounts(pool_account_info, true));
        Ok(accounts)
    }

    fn find_reserve_pda_account_meta(pool_account: &AccountInfo) -> (Pubkey, bool) {
        (
            Self::find_pool_account_related_address(pool_account, b"reserve"),
            true,
        )
    }
}
