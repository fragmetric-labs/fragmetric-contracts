use anchor_lang::{prelude::*, solana_program};
use anchor_spl::token_interface::{Mint, TokenAccount};
use crate::modules::{fund, staking};
use crate::errors;
use crate::events;
use crate::constants::{ADMIN_PUBKEY};
use crate::utils::PDASeeds;

pub fn process_run<'info>(
    operator: &Signer<'info>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    fund_account: &mut Account<'info, fund::FundAccount>,
    fund_execution_reserve_account: &SystemAccount<'info>,
    fund_execution_reserve_account_bump: u8,
    remaining_accounts: &'info [AccountInfo<'info>],
    _current_timestamp: i64,
    _current_slot: u64,
) -> Result<()> {
    // temporary authorization
    require_eq!(operator.key(), ADMIN_PUBKEY);

    // accounts
    let [
        stake_pool_program,
        stake_pool,
        stake_pool_withdraw_authority,
        reserve_stake_account,
        manager_fee_account,
        pool_mint,
        token_program,
        fund_supported_token_account,
    ] = remaining_accounts else {
        return Err(ProgramError::NotEnoughAccountKeys)?;
    };

    let pool_mint = &remaining_accounts[5];
    let token_program = &remaining_accounts[6];
    let mut fund_supported_token_account_parsed = InterfaceAccount::<TokenAccount>::try_from(&remaining_accounts[7])?;

    // 1. stake sol to jitoSOL
    {
        let rent = Rent::get()?;
        let sol_operation_reserved_amount = fund_account.sol_operation_reserved_amount;
        if sol_operation_reserved_amount > 0 {
            let moving_amount = sol_operation_reserved_amount
                .checked_sub(fund_execution_reserve_account.get_lamports())
                .ok_or_else(|| error!(errors::ErrorCode::FundUnexpectedExecutionReservedAccountBalanceException))?;

            if moving_amount > 0 {
                fund_account.sub_lamports(moving_amount)?;
                fund_execution_reserve_account.add_lamports(moving_amount)?;
                if !rent.is_exempt(fund_account.get_lamports(), fund_account.to_account_info().data_len()) {
                    err!(errors::ErrorCode::FundUnexpectedExecutionReservedAccountBalanceException)?;
                }
                msg!("transferred sol_operation_reserved_amount to fund_execution_reserve_account");
                return Ok(()) // need to re-run
            }
        }

        let staking_lamports = fund_execution_reserve_account.get_lamports();
        if staking_lamports > 0 {
            let before_token = fund_supported_token_account_parsed.amount;
            staking::deposit_sol_to_spl_stake_pool(
                &staking::SPLStakePoolContext {
                    program: stake_pool_program.clone(),
                    stake_pool: stake_pool.clone(),
                    sol_deposit_authority: None,
                    stake_pool_withdraw_authority: stake_pool_withdraw_authority.clone(),
                    reserve_stake_account: reserve_stake_account.clone(),
                    manager_fee_account: manager_fee_account.clone(),
                    pool_mint: pool_mint.clone(),
                    token_program: token_program.clone(),
                },
                fund_execution_reserve_account.get_lamports(),
                fund_execution_reserve_account,
                fund_supported_token_account,
                &[&[
                    fund::FundAccount::EXECUTION_RESERVED_SEED,
                    &fund_account.receipt_token_mint.to_bytes(),
                    &[fund_execution_reserve_account_bump],
                ]],
            )?;
            fund_supported_token_account_parsed.reload()?;
            fund_account.sol_operation_reserved_amount = fund_account.sol_operation_reserved_amount
                .checked_sub(staking_lamports)
                .ok_or_else(|| error!(errors::ErrorCode::FundUnexpectedExecutionReservedAccountBalanceException))?;

            let minted_token = fund_supported_token_account_parsed.amount - before_token;
            let supported_token_info = fund_account.get_supported_token_mut(fund_supported_token_account_parsed.mint)?;
            supported_token_info.set_operation_reserved_amount(supported_token_info.get_operation_reserved_amount().checked_add(minted_token).unwrap());
            msg!("staked {} sol to mint {} tokens", staking_lamports, minted_token);

            require_gte!(minted_token, staking_lamports.div_ceil(2));
        }
    }

    emit!(events::OperatorProcessedJob {
        receipt_token_mint: receipt_token_mint.key(),
        fund_account: fund::FundAccountInfo::from(
            fund_account,
            receipt_token_mint,
        ),
    });

    // 2. normalize supported tokens
    // TODO: nt_opeartion_reserved_amount -> fund_account_ref.get_nt_operation_reserved_amount()
    // let mut nt_opeartion_reserved_amount = 0u64;
    // {
    //     let supported_tokens = fund_account.get_supported_tokens_iter()
    //         .map(|token| (token.get_mint().clone(), token.get_operation_reserved_amount()))
    //         .collect::<Vec<_>>();
    //     for (supported_token_mint, supported_token_operation_reserved_amount) in supported_tokens {
    //         nt_opeartion_reserved_amount += fund::normalize_lst_operation_reserved(
    //             fund_account,
    //             &supported_token_mint,
    //             supported_token_operation_reserved_amount,
    //             // TODO: pick required accounts for this fn
    //             remaining_accounts,
    //         )?;
    //     }
    // }

    // 3. restake normalized tokens
    // {
    //     fund::restake_nt_operation_reserved(
    //         fund_account,
    //         nt_opeartion_reserved_amount,
    //         // TODO: pick required accounts for this fn
    //         remaining_accounts,
    //     );
    // }

    Ok(())
}

// fn pick_account<'info, T: AccountDeserialize + Clone>(key: &Pubkey, accounts: &[AccountInfo<'info>]) -> Result<Box<AccountInfo<'info>>> {
//     accounts.iter().find(|account| {
//         return account.key.eq(key);
//     }).map_or_else(Err(Error::from(ProgramError::NotEnoughAccountKeys)), |account| {
//         let b = Box::new(Account::<T>::try_from(account)?);
//         return b.as_ref();
//     })
// }