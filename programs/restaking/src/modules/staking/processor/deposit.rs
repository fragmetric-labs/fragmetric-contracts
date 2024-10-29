use anchor_lang::{prelude::*, solana_program::program::invoke_signed};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::errors::ErrorCode;
use crate::modules::fund::{FundAccount, FUND_ACCOUNT_CURRENT_VERSION};
use crate::modules::staking::outer_modules;

pub fn process_move_fund_to_operation_reserve_account<'info>(
    fund_account: &mut Account<'info, FundAccount>,
    operation_reserve_account: &SystemAccount<'info>,
) -> Result<()> {
    let total_moving_sol_amount = fund_account.sol_operation_reserved_amount;

    fund_account.sub_lamports(total_moving_sol_amount)?;
    operation_reserve_account.add_lamports(total_moving_sol_amount)?;

    fund_account.sol_operation_reserved_amount = fund_account
        .sol_operation_reserved_amount
        .checked_sub(total_moving_sol_amount)
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

    Ok(())
}

#[derive(Accounts)]
pub struct DepositSolWithAuthorityToJitoStakePoolAccounts<'info> {
    /// CHECK
    pub program_id: AccountInfo<'info>,
    /// CHECK
    pub stake_pool: AccountInfo<'info>,
    /// CHECK
    pub sol_deposit_authority: AccountInfo<'info>,
    /// CHECK
    pub stake_pool_withdraw_authority: AccountInfo<'info>,
    /// CHECK
    pub reserve_stake_account: AccountInfo<'info>,
    /// CHECK
    pub lamports_from: AccountInfo<'info>,
    /// CHECK
    pub pool_tokens_to: AccountInfo<'info>,
    /// CHECK
    pub manager_fee_account: AccountInfo<'info>,
    /// CHECK
    pub referrer_pool_tokens_account: AccountInfo<'info>,
    /// CHECK
    pub pool_mint: AccountInfo<'info>,
    /// CHECK
    pub token_program_id: AccountInfo<'info>,
}

pub fn process_deposit_sol_to_spl_stake_pool<'info>(
    remaining_accounts: &'info [AccountInfo<'info>],
    operation_reserve_account: &SystemAccount<'info>,
    supported_token_account: &InterfaceAccount<'info, TokenAccount>,
    spl_pool_token_mint: &InterfaceAccount<'info, Mint>,
    supported_token_program: &Interface<'info, TokenInterface>,
    fund_account: &Account<'info, FundAccount>,
    operation_reserve_account_bump: u8,
    sol_amount: u64,
) -> Result<()> {
    let program_id = &remaining_accounts[0].key();
    let stake_pool = &remaining_accounts[1].key();
    let sol_deposit_authority = &operation_reserve_account.key();
    let stake_pool_withdraw_authority = &remaining_accounts[2].key();
    let reserve_stake_account = &remaining_accounts[3].key();
    let lamports_from = &operation_reserve_account.key();
    let pool_tokens_to = &supported_token_account.key();
    let manager_fee_account = &remaining_accounts[4].key();
    let referrer_pool_tokens_account = &supported_token_account.key();
    let pool_mint = &spl_pool_token_mint.key();
    let token_program_id = &supported_token_program.key();

    let deposit_sol_ix = outer_modules::deposit_sol_with_authority(
        program_id,
        stake_pool,
        sol_deposit_authority,
        stake_pool_withdraw_authority,
        reserve_stake_account,
        lamports_from,
        pool_tokens_to,
        manager_fee_account,
        referrer_pool_tokens_account,
        pool_mint,
        token_program_id,
        sol_amount,
    );

    let deposit_sol_account_infos = DepositSolWithAuthorityToJitoStakePoolAccounts {
        program_id: remaining_accounts[0].clone(),
        stake_pool: remaining_accounts[1].clone(),
        sol_deposit_authority: operation_reserve_account.to_account_info(),
        stake_pool_withdraw_authority: remaining_accounts[2].clone(),
        reserve_stake_account: remaining_accounts[3].clone(),
        lamports_from: operation_reserve_account.to_account_info(),
        pool_tokens_to: supported_token_account.to_account_info(),
        manager_fee_account: remaining_accounts[4].clone(),
        referrer_pool_tokens_account: supported_token_account.to_account_info(),
        pool_mint: spl_pool_token_mint.to_account_info(),
        token_program_id: supported_token_program.to_account_info(),
    }
    .to_account_infos();

    let operation_reserve_account_signer_seeds: &[&[&[u8]]] = &[&[
        FundAccount::OPERATION_RESERVED_SEED,
        &fund_account.receipt_token_mint.to_bytes(),
        &[operation_reserve_account_bump],
    ]];

    invoke_signed(
        &deposit_sol_ix,
        &deposit_sol_account_infos,
        operation_reserve_account_signer_seeds,
    )?;

    Ok(())
}
