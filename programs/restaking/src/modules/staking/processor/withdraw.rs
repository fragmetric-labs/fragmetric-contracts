use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke_signed, sysvar},
};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::errors::ErrorCode;
use crate::modules::fund::{FundAccount, FUND_ACCOUNT_CURRENT_VERSION};
use crate::utils::PDASeeds;

pub fn process_withdraw_sol_from_spl_stake_pool<'info>(
    remaining_accounts: &'info [AccountInfo<'info>],
    spl_stake_pool_program: &AccountInfo<'info>,
    fund_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
    fund_reserve_account: &SystemAccount<'info>,
    spl_pool_token_mint: &InterfaceAccount<'info, Mint>,
    supported_token_program: &Interface<'info, TokenInterface>,
    fund_account: &Account<'info, FundAccount>,
    signer_seeds: &[&[&[u8]]],
    token_amount: u64,
) -> Result<()> {
    let withdraw_sol_ix = spl_stake_pool::instruction::withdraw_sol(
        spl_stake_pool_program.key,
        remaining_accounts[0].key,
        remaining_accounts[1].key,
        &fund_account.key(),
        &fund_supported_token_account.key(),
        remaining_accounts[2].key,
        fund_reserve_account.key,
        remaining_accounts[3].key,
        &spl_pool_token_mint.key(),
        supported_token_program.key,
        token_amount,
    );
    // msg!("&withdraw_sol_ix.accounts[2].pubkey: {}, is_signer: {}, is_writable: {}", &withdraw_sol_ix.accounts[2].pubkey, &withdraw_sol_ix.accounts[2].is_signer, &withdraw_sol_ix.accounts[2].is_writable);
    // for (i, ix_account) in withdraw_sol_ix.accounts.clone().into_iter().enumerate() {
    //     msg!("&withdraw_sol_ix.accounts[{}].pubkey: {}, is_signer: {}, is_writable: {}", i, &ix_account.pubkey, &ix_account.is_signer, &ix_account.is_writable);
    // }

    invoke_signed(
        &withdraw_sol_ix,
        &[
            remaining_accounts[0].clone(),
            remaining_accounts[1].clone(),
            fund_account.to_account_info(),
            fund_supported_token_account.to_account_info(),
            remaining_accounts[2].clone(),
            fund_reserve_account.to_account_info(),
            remaining_accounts[3].clone(),
            spl_pool_token_mint.to_account_info(),
            remaining_accounts[4].clone(),
            remaining_accounts[5].clone(),
            remaining_accounts[6].clone(),
            supported_token_program.to_account_info(),
        ],
        signer_seeds
    )?;

    Ok(())
}
