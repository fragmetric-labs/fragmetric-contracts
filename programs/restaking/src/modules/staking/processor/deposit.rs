use anchor_lang::{prelude::*, solana_program::program::invoke_signed};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use spl_stake_pool;

use crate::errors::ErrorCode;
use crate::modules::fund::{FundAccount, FUND_ACCOUNT_CURRENT_VERSION};

pub struct SPLStakePoolContext<'info> {
    pub program: AccountInfo<'info>,
    pub stake_pool: AccountInfo<'info>,

    pub sol_deposit_authority: Option<AccountInfo<'info>>,
    pub stake_pool_withdraw_authority: AccountInfo<'info>,
    pub reserve_stake_account: AccountInfo<'info>,
    // FROM ACCOUNT
    // pub lamports_from: AccountInfo<'info>,
    // TO ACCOUNT
    // pub pool_tokens_to: AccountInfo<'info>,
    pub manager_fee_account: AccountInfo<'info>,
    // pub referrer_pool_tokens_account: AccountInfo<'info>,
    pub pool_mint: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

pub fn deposit_sol_to_spl_stake_pool<'info>(
    ctx: &SPLStakePoolContext<'info>,
    lamports: u64,
    lamports_from: &AccountInfo<'info>,
    pool_tokens_to: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    require_eq!(spl_stake_pool::ID, ctx.program.key());

    let ix = spl_stake_pool::instruction::deposit_sol(
        ctx.program.key,
        ctx.stake_pool.key,
        ctx.stake_pool_withdraw_authority.key,
        ctx.reserve_stake_account.key,
        lamports_from.key,
        pool_tokens_to.key,
        ctx.manager_fee_account.key,
        pool_tokens_to.key,
        ctx.pool_mint.key,
        ctx.token_program.key,
        lamports,
    );

    invoke_signed(
        &ix,
        &[
            ctx.program.clone(),
            ctx.stake_pool.clone(),
            ctx.stake_pool_withdraw_authority.clone(),
            ctx.reserve_stake_account.clone(),
            lamports_from.clone(),
            pool_tokens_to.clone(),
            ctx.manager_fee_account.clone(),
            pool_tokens_to.clone(),
            ctx.pool_mint.clone(),
            ctx.token_program.clone(),
        ],
        signer_seeds,
    )?;

    Ok(())
}
