use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;
use whirlpool_cpi::whirlpool::accounts::Whirlpool;

use super::ValidateLiquidityPool;

pub(in crate::modules) struct OrcaDEXLiquidityPoolService<'info> {
    whirlpool_program: &'info AccountInfo<'info>,
    pool_account: &'info AccountInfo<'info>,
    token_mint_a: &'info AccountInfo<'info>,
    token_vault_a: &'info AccountInfo<'info>,
    token_program_a: &'info AccountInfo<'info>,
    token_mint_b: &'info AccountInfo<'info>,
    token_vault_b: &'info AccountInfo<'info>,
    token_program_b: &'info AccountInfo<'info>,
}

impl ValidateLiquidityPool for OrcaDEXLiquidityPoolService<'_> {
    #[inline(never)]
    fn validate_liquidity_pool<'info>(
        pool_account: &'info AccountInfo<'info>,
        from_token_mint: &Pubkey,
        to_token_mint: &Pubkey,
    ) -> Result<()> {
        let pool_account = Self::deserialize_pool_account(pool_account)?;

        // This validates pool account by checking that input from_token_mint and to_token_mint match the pool's tokenA, B mint.
        Self::a_to_b(&pool_account, &from_token_mint.key(), &to_token_mint.key())?;

        Ok(())
    }
}

impl<'info> OrcaDEXLiquidityPoolService<'info> {
    #[inline(never)]
    pub fn new(
        whirlpool_program: &'info AccountInfo<'info>,
        pool_account: &'info AccountInfo<'info>,
        token_mint_a: &'info AccountInfo<'info>,
        token_vault_a: &'info AccountInfo<'info>,
        token_program_a: &'info AccountInfo<'info>,
        token_mint_b: &'info AccountInfo<'info>,
        token_vault_b: &'info AccountInfo<'info>,
        token_program_b: &'info AccountInfo<'info>,
    ) -> Result<Self> {
        let pool = &*Self::deserialize_pool_account(pool_account)?;

        require_keys_eq!(whirlpool_cpi::whirlpool::ID, whirlpool_program.key());
        require_keys_eq!(pool.token_mint_a, token_mint_a.key());
        require_keys_eq!(pool.token_vault_a, token_vault_a.key());
        require_keys_eq!(*token_mint_a.owner, token_program_a.key());
        require_keys_eq!(pool.token_mint_b, token_mint_b.key());
        require_keys_eq!(pool.token_vault_b, token_vault_b.key());
        require_keys_eq!(*token_mint_b.owner, token_program_b.key());

        Ok(Self {
            whirlpool_program,
            pool_account,
            token_mint_a,
            token_vault_a,
            token_program_a,
            token_mint_b,
            token_vault_b,
            token_program_b,
        })
    }

    pub(super) fn deserialize_pool_account<'a>(
        pool_account: &'a AccountInfo<'a>,
    ) -> Result<Account<'a, Whirlpool>> {
        Account::try_from(pool_account)
    }

    fn a_to_b(
        pool_account: &Account<Whirlpool>,
        from_token_mint: &Pubkey,
        to_token_mint: &Pubkey,
    ) -> Result<bool> {
        let a_to_b = pool_account.token_mint_a == *from_token_mint;
        if a_to_b {
            require_keys_eq!(pool_account.token_mint_b, *to_token_mint);
        } else {
            require_keys_eq!(pool_account.token_mint_a, *to_token_mint);
            require_keys_eq!(pool_account.token_mint_b, *from_token_mint);
        }

        Ok(a_to_b)
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
    /// * token_program_a
    /// * token_mint_b
    /// * token_vault_b(writable)
    /// * token_program_b
    fn find_accounts_to_new(
        pool_account: &Account<Whirlpool>,
        token_program_a: &Pubkey,
        token_program_b: &Pubkey,
    ) -> [(Pubkey, bool); 8] {
        [
            (whirlpool_cpi::whirlpool::ID, false),
            (pool_account.key(), true),
            (pool_account.token_mint_a, false),
            (pool_account.token_vault_a, true),
            (*token_program_a, false),
            (pool_account.token_mint_b, false),
            (pool_account.token_vault_b, true),
            (*token_program_b, false),
        ]
    }

    /// ref: https://github.com/orca-so/whirlpools/blob/main/programs/whirlpool/src/util/sparse_swap.rs#L315
    /// * tick_array_0(writable)
    /// * tick_array_1(writable)
    /// * tick_array_2(writable)
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
            } * ticks_in_array;
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
    /// * (4) token_program_a
    /// * (5) token_mint_b
    /// * (6) token_vault_b(writable)
    /// * (7) token_program_b
    /// * (8) memo_program
    /// * (9) oracle(writable)
    /// * (10) tick_array_0(writable)
    /// * (11) tick_array_1(writable)
    /// * (12) tick_array_2(writable)
    #[inline(never)]
    pub fn find_accounts_to_swap(
        pool_account: &'info AccountInfo<'info>,
        from_token_mint: &AccountInfo,
        to_token_mint: &AccountInfo,
    ) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        let pool_account = &Self::deserialize_pool_account(pool_account)?;

        let a_to_b = Self::a_to_b(pool_account, from_token_mint.key, to_token_mint.key)?;
        let token_program_a;
        let token_program_b;
        if a_to_b {
            token_program_a = from_token_mint.owner;
            token_program_b = to_token_mint.owner;
        } else {
            token_program_a = to_token_mint.owner;
            token_program_b = from_token_mint.owner;
        }

        let accounts = Self::find_accounts_to_new(pool_account, token_program_a, token_program_b)
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

    /// returns [from_token_swapped_amount, to_token_swapped_amount]
    #[inline(never)]
    pub fn swap(
        &self,
        // fixed
        memo_program: &AccountInfo<'info>,
        oracle: &AccountInfo<'info>,
        tick_array_0: &AccountInfo<'info>,
        tick_array_1: &AccountInfo<'info>,
        tick_array_2: &AccountInfo<'info>,

        // variant
        from_token_account: &'info AccountInfo<'info>,
        to_token_account: &'info AccountInfo<'info>,
        token_account_signer: &AccountInfo<'info>,
        token_account_signer_seeds: &[&[&[u8]]],

        from_token_amount: u64,
    ) -> Result<(u64, u64)> {
        let pool_account = &Self::deserialize_pool_account(self.pool_account)?;
        let mut from_token_account =
            InterfaceAccount::<TokenAccount>::try_from(from_token_account)?;
        let mut to_token_account = InterfaceAccount::<TokenAccount>::try_from(to_token_account)?;

        let a_to_b = Self::a_to_b(
            pool_account,
            &from_token_account.mint,
            &to_token_account.mint,
        )?;
        let (token_owner_account_a, token_owner_account_b) = if a_to_b {
            (&from_token_account, &to_token_account)
        } else {
            (&to_token_account, &from_token_account)
        };

        let from_token_account_amount_before = from_token_account.amount;
        let to_token_account_amount_before = to_token_account.amount;

        require_gte!(from_token_account_amount_before, from_token_amount);

        whirlpool_cpi::whirlpool::cpi::swap_v2(
            CpiContext::new_with_signer(
                self.whirlpool_program.to_account_info(),
                whirlpool_cpi::whirlpool::cpi::accounts::SwapV2 {
                    token_program_a: self.token_program_a.to_account_info(),
                    token_program_b: self.token_program_b.to_account_info(),
                    memo_program: memo_program.to_account_info(),
                    token_authority: token_account_signer.to_account_info(),
                    whirlpool: self.pool_account.to_account_info(),
                    token_mint_a: self.token_mint_a.to_account_info(),
                    token_mint_b: self.token_mint_b.to_account_info(),
                    token_owner_account_a: token_owner_account_a.to_account_info(),
                    token_vault_a: self.token_vault_a.to_account_info(),
                    token_owner_account_b: token_owner_account_b.to_account_info(),
                    token_vault_b: self.token_vault_b.to_account_info(),
                    tick_array_0: tick_array_0.to_account_info(),
                    tick_array_1: tick_array_1.to_account_info(),
                    tick_array_2: tick_array_2.to_account_info(),
                    oracle: oracle.to_account_info(),
                },
                token_account_signer_seeds,
            ),
            from_token_amount,
            0,
            0,
            true,
            a_to_b,
            None,
        )?;

        from_token_account.reload()?;
        to_token_account.reload()?;
        let from_token_account_amount = from_token_account.amount;
        let to_token_account_amount = to_token_account.amount;
        let from_token_swapped_amount =
            from_token_account_amount_before - from_token_account_amount;
        let to_token_swapped_amount = to_token_account_amount - to_token_account_amount_before;

        msg!(
            "SWAP#orca: from_token_mint={}, to_token_mint={}, from_token_account_amount={}, to_token_account_amount={}, ∆from_token_amount={}, ∆to_token_amount={}",
            from_token_account.mint,
            to_token_account.mint,
            from_token_account_amount,
            to_token_account_amount,
            from_token_swapped_amount,
            to_token_swapped_amount,
        );

        Ok((from_token_swapped_amount, to_token_swapped_amount))
    }
}
