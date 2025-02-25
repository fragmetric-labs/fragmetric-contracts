use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token_2022;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use whirlpool_cpi::whirlpool::accounts::{Whirlpool, WhirlpoolsConfig};

pub(in crate::modules) struct OrcaDEXLiquidityPoolService<'info> {
    whirlpool_program: Program<'info, whirlpool_cpi::whirlpool::program::Whirlpool>,
    pool_account: Account<'info, Whirlpool>,
    token_mint_a: &'info AccountInfo<'info>,
    token_vault_a: &'info AccountInfo<'info>,
    token_program_a: &'info AccountInfo<'info>,
    token_mint_b: &'info AccountInfo<'info>,
    token_vault_b: &'info AccountInfo<'info>,
    token_program_b: &'info AccountInfo<'info>,
}

impl<'info> OrcaDEXLiquidityPoolService<'info> {
    #[inline(never)]
    pub fn new(
        whirlpool_program: &'info AccountInfo<'info>,
        pool_account: &'info AccountInfo<'info>,
        token_mint_a: &'info AccountInfo<'info>,
        token_vault_a: &'info AccountInfo<'info>,
        token_mint_b: &'info AccountInfo<'info>,
        token_vault_b: &'info AccountInfo<'info>,
        token_program: &'info AccountInfo<'info>,
        token_2022_program: &'info AccountInfo<'info>,
    ) -> Result<Self> {
        let pool_account = Self::deserialize_pool_account(pool_account)?;
        let _ = Program::<token::Token>::try_from(token_program)?;
        let _ = Program::<token_2022::Token2022>::try_from(token_2022_program)?;

        require_keys_eq!(pool_account.token_mint_a, token_mint_a.key());
        require_keys_eq!(pool_account.token_vault_a, token_vault_a.key());
        require_keys_eq!(pool_account.token_mint_b, token_mint_b.key());
        require_keys_eq!(pool_account.token_vault_b, token_vault_b.key());

        let token_program_a = match token_mint_a.owner {
            &token::ID => token_program,
            &token_2022::ID => token_2022_program,
            _ => err!(error::ErrorCode::InvalidProgramId)?,
        };
        let token_program_b = match token_mint_b.owner {
            &token::ID => token_program,
            &token_2022::ID => token_2022_program,
            _ => err!(error::ErrorCode::InvalidProgramId)?,
        };

        Ok(Self {
            whirlpool_program: Program::try_from(whirlpool_program)?,
            pool_account,
            token_mint_a,
            token_vault_a,
            token_program_a,
            token_mint_b,
            token_vault_b,
            token_program_b,
        })
    }

    #[inline(always)]
    pub(super) fn deserialize_pool_account(
        pool_account: &'info AccountInfo<'info>,
    ) -> Result<Account<'info, Whirlpool>> {
        Account::try_from(pool_account)
    }

    fn find_tick_array_address<'a>(
        pool_account: &impl AsRef<AccountInfo<'a>>,
        tick_array_start_index: i32,
    ) -> Pubkey {
        Pubkey::find_program_address(
            &[
                b"tick_array",
                pool_account.as_ref().key.as_ref(),
                tick_array_start_index.to_string().as_bytes(),
            ],
            &whirlpool_cpi::whirlpool::ID,
        )
        .0
    }

    /// * pool_program
    /// * pool_account(writable)
    /// * token_mint_a
    /// * token_vault_a(writable)
    /// * token_mint_b
    /// * token_vault_b(writable)
    /// * token_program
    /// * token_2022_program
    fn find_accounts_to_new(pool_account: &Account<Whirlpool>) -> [(Pubkey, bool); 8] {
        [
            (whirlpool_cpi::whirlpool::ID, false),
            (pool_account.key(), true),
            (pool_account.token_mint_a, false),
            (pool_account.token_vault_a, true),
            (pool_account.token_mint_b, false),
            (pool_account.token_vault_b, true),
            (token::ID, false),
            (token_2022::ID, false),
        ]
    }

    /// ref: https://github.com/orca-so/whirlpools/blob/main/programs/whirlpool/src/util/sparse_swap.rs#L315
    /// * tick_array0(writable)
    /// * tick_array1(writable)
    /// * tick_array2(writable)
    fn find_tick_array_accounts_to_swap(
        pool_account: &Account<Whirlpool>,
        a_to_b: bool,
    ) -> [(Pubkey, bool); 3] {
        const TICK_ARRAY_SIZE: i32 = 88;
        let current_tick_index = pool_account.tick_current_index;
        let tick_spacing = pool_account.tick_spacing;
        let ticks_in_array = TICK_ARRAY_SIZE * tick_spacing as i32;

        let base_tick_array_start_index =
            if current_tick_index % ticks_in_array == 0 || current_tick_index >= 0 {
                current_tick_index / ticks_in_array
            } else {
                current_tick_index / ticks_in_array - 1
            };
        let offset = if a_to_b {
            [0, -1, -2]
        } else if current_tick_index + tick_spacing as i32
            >= base_tick_array_start_index + ticks_in_array
        {
            [1, 2, 3]
        } else {
            [0, 1, 2]
        };

        offset.map(|o| {
            let tick_array_start_index = base_tick_array_start_index + o * ticks_in_array;
            (
                Self::find_tick_array_address(pool_account, tick_array_start_index),
                true,
            )
        })
    }

    /// Transfer hook tokens are not supported.
    ///
    /// * (0) pool_program
    /// * (1) pool_account(writable)
    /// * (2) token_mint_a
    /// * (3) token_vault_a(writable)
    /// * (4) token_mint_b
    /// * (5) token_vault_b(writable)
    /// * (6) token_program
    /// * (7) token_2022_program
    /// * (8) memo_program
    /// * (9) oracle(writable)
    /// * (10) tick_array0(writable)
    /// * (11) tick_array1(writable)
    /// * (12) tick_array2(writable)
    #[inline(never)]
    pub fn find_accounts_to_swap(
        pool_account: &'info AccountInfo<'info>,
        a_to_b: bool,
    ) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        let pool_account = &Self::deserialize_pool_account(pool_account)?;

        let accounts = Self::find_accounts_to_new(pool_account)
            .into_iter()
            .chain([
                // memo_program
                (anchor_spl::memo::spl_memo::ID, false),
                // oracle(writable)
                (
                    Pubkey::find_program_address(
                        &[b"oracle", pool_account.key().as_ref()],
                        &whirlpool_cpi::whirlpool::ID,
                    )
                    .0,
                    true,
                ),
            ])
            .chain(Self::find_tick_array_accounts_to_swap(pool_account, a_to_b));

        Ok(accounts)
    }

    /// returns [amount_in, amount_out]
    #[inline(never)]
    pub fn swap(
        &self,
        // fixed
        memo_program: &AccountInfo<'info>,
        oracle: &AccountInfo<'info>,
        tick_array0: &AccountInfo<'info>,
        tick_array1: &AccountInfo<'info>,
        tick_array2: &AccountInfo<'info>,

        // variant
        token_owner_account_a: &'info AccountInfo<'info>,
        token_owner_account_b: &'info AccountInfo<'info>,
        token_owner: &AccountInfo<'info>,
        token_owner_seeds: &[&[&[u8]]],

        amount_in: u64,
        a_to_b: bool, // if a_to_b then input = token_a
    ) -> Result<(u64, u64)> {
        let (mut input_token_account, mut output_token_account) = {
            let token_owner_account_a =
                InterfaceAccount::<TokenAccount>::try_from(token_owner_account_a)?;
            let token_owner_account_b =
                InterfaceAccount::<TokenAccount>::try_from(token_owner_account_b)?;
            if a_to_b {
                (token_owner_account_a, token_owner_account_b)
            } else {
                (token_owner_account_b, token_owner_account_a)
            }
        };

        let input_token_amount_before = input_token_account.amount;
        let output_token_amount_before = output_token_account.amount;

        whirlpool_cpi::whirlpool::cpi::swap_v2(
            CpiContext::new_with_signer(
                self.whirlpool_program.to_account_info(),
                whirlpool_cpi::whirlpool::cpi::accounts::SwapV2 {
                    token_program_a: self.token_program_a.to_account_info(),
                    token_program_b: self.token_program_b.to_account_info(),
                    memo_program: memo_program.to_account_info(),
                    token_authority: token_owner.to_account_info(),
                    whirlpool: self.pool_account.to_account_info(),
                    token_mint_a: self.token_mint_a.to_account_info(),
                    token_mint_b: self.token_mint_b.to_account_info(),
                    token_owner_account_a: token_owner_account_a.to_account_info(),
                    token_vault_a: self.token_vault_a.to_account_info(),
                    token_owner_account_b: token_owner_account_b.to_account_info(),
                    token_vault_b: self.token_vault_b.to_account_info(),
                    tick_array0: tick_array0.to_account_info(),
                    tick_array1: tick_array1.to_account_info(),
                    tick_array2: tick_array2.to_account_info(),
                    oracle: oracle.to_account_info(),
                },
                token_owner_seeds,
            ),
            amount_in,
            0,
            0,
            true,
            a_to_b,
            None,
        )?;

        input_token_account.reload()?;
        output_token_account.reload()?;
        let input_token_amount = input_token_account.amount;
        let output_token_amount = output_token_account.amount;
        let amount_in = input_token_amount_before - input_token_amount;
        let amount_out = output_token_amount - output_token_amount_before;

        msg!(
            "SWAP#orca: input_token_mint={}, output_token_mint={}, amount_in={}, amount_out={}",
            input_token_account.mint,
            output_token_account.mint,
            amount_in,
            amount_out
        );

        Ok((amount_in, amount_out))
    }
}
