use anchor_lang::{prelude::*, solana_program, CheckOwner};
use anchor_spl::token::accessor::mint;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::ADMIN_PUBKEY;
use crate::errors;
use crate::events;
use crate::modules::{fund, normalize, pricing, staking};
use crate::utils::*;

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
        // staking
        stake_pool_program,
        stake_pool,
        stake_pool_withdraw_authority,
        reserve_stake_account,
        manager_fee_account,
        pool_mint,
        token_program,
        fund_supported_token_account_to_stake,
        // normalization
        normalized_token_pool_account,
        normalized_token_mint,
        normalized_token_program,
        fund_normalized_token_account,
        fund_supported_token_account_to_normalize,
        fund_supported_token_account_authority_to_normalize,
        fund_supported_token_mint_to_normalize,
        fund_supported_token_program_to_normalize,
        normalized_token_pool_supported_token_lock_account,
        pricing_source_accounts @..,
    ] = remaining_accounts else {
        return Err(ProgramError::NotEnoughAccountKeys)?;
    };

    // 1. stake sol to jitoSOL
    {
        let mut fund_supported_token_account_to_stake_parsed =
            parse_interface_account_boxed::<TokenAccount>(fund_supported_token_account_to_stake)?;
        let sol_operation_reserved_amount = fund_account.sol_operation_reserved_amount;
        if sol_operation_reserved_amount > 0 {
            let moving_amount = sol_operation_reserved_amount
                .checked_sub(fund_execution_reserve_account.get_lamports())
                .ok_or_else(|| {
                    error!(
                        errors::ErrorCode::FundUnexpectedExecutionReservedAccountBalanceException
                    )
                })?;

            if moving_amount > 0 {
                let rent = Rent::get()?;
                fund_account.sub_lamports(moving_amount)?;
                fund_execution_reserve_account.add_lamports(moving_amount)?;
                if !rent.is_exempt(
                    fund_account.get_lamports(),
                    fund_account.to_account_info().data_len(),
                ) {
                    err!(
                        errors::ErrorCode::FundUnexpectedExecutionReservedAccountBalanceException
                    )?;
                }
                msg!("transferred sol_operation_reserved_amount={} to fund_execution_reserve_account={}", moving_amount, fund_execution_reserve_account.get_lamports());
                return Ok(()); // need to re-run
            }
        }

        let staking_lamports = fund_execution_reserve_account.get_lamports();
        if staking_lamports > 0 {
            let before_fund_supported_token_amount =
                fund_supported_token_account_to_stake_parsed.amount;
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
                fund_supported_token_account_to_stake,
                &[&[
                    fund::FundAccount::EXECUTION_RESERVED_SEED,
                    &fund_account.receipt_token_mint.to_bytes(),
                    &[fund_execution_reserve_account_bump],
                ]],
            )?;
            fund_supported_token_account_to_stake_parsed.reload()?;
            fund_account.sol_operation_reserved_amount = fund_account
                .sol_operation_reserved_amount
                .checked_sub(staking_lamports)
                .ok_or_else(|| {
                    error!(
                        errors::ErrorCode::FundUnexpectedExecutionReservedAccountBalanceException
                    )
                })?;

            let minted_supported_token_amount = fund_supported_token_account_to_stake_parsed.amount
                - before_fund_supported_token_amount;
            let fund_supported_token_info = fund_account
                .get_supported_token_mut(fund_supported_token_account_to_stake_parsed.mint)?;
            fund_supported_token_info.set_operation_reserved_amount(
                fund_supported_token_info
                    .get_operation_reserved_amount()
                    .checked_add(minted_supported_token_amount)
                    .unwrap(),
            );
            msg!(
                "staked {} sol to mint {} tokens",
                staking_lamports,
                minted_supported_token_amount
            );

            require_gte!(minted_supported_token_amount, staking_lamports.div_ceil(2));
            require_eq!(
                fund_supported_token_info.get_operation_reserved_amount(),
                fund_supported_token_account_to_stake_parsed.amount
            );
        }
    }

    // create pricing calculator
    let mut pricing_source_map =
        fund::create_pricing_source_map(fund_account, pricing_source_accounts)?;
    pricing_source_map.insert(
        normalized_token_mint.key(),
        (
            pricing::TokenPricingSource::NormalizedTokenPool {
                mint_address: normalized_token_mint.key(),
                pool_address: normalized_token_pool_account.key(),
            },
            vec![normalized_token_mint, normalized_token_pool_account],
        ),
    );

    // 2. normalize supported tokens
    // TODO: nt_opeartion_reserved_amount -> fund_account_ref.get_nt_operation_reserved_amount()
    {
        let normalized_token_pool_account_parsed =
            parse_account_boxed(normalized_token_pool_account)?;
        let mut fund_supported_token_account_to_normalize_parsed =
            parse_interface_account_boxed::<TokenAccount>(
                fund_supported_token_account_to_normalize,
            )?;
        let fund_supported_token_account_authority_to_normalize_parsed =
            parse_account_boxed::<fund::SupportedTokenAuthority>(
                fund_supported_token_account_authority_to_normalize,
            )?;
        let fund_supported_token_info_to_normalize = fund_account
            .get_supported_token_mut(fund_supported_token_account_to_normalize_parsed.mint)?;
        let mut fund_normalized_token_account_parsed =
            parse_interface_account_boxed::<TokenAccount>(fund_normalized_token_account)?;

        let normalizing_supported_token_amount =
            fund_supported_token_info_to_normalize.get_operation_reserved_amount();
        if normalizing_supported_token_amount > 0 {
            let before_fund_supported_token_amount =
                fund_supported_token_account_to_normalize_parsed.amount;
            let before_fund_normalized_token_amount = fund_normalized_token_account_parsed.amount;
            let normalized_token_mint_parsed =
                parse_interface_account_boxed(normalized_token_mint)?;
            let normalized_token_program_parsed = normalized_token_program.try_into()?;
            let fund_supported_token_mint_to_normalize_parsed =
                parse_interface_account_boxed(fund_supported_token_mint_to_normalize)?;
            let fund_supported_token_program_to_normalize_parsed =
                fund_supported_token_program_to_normalize.try_into()?;
            let mut normalized_token_pool_supported_token_lock_account_parsed =
                parse_interface_account_boxed::<TokenAccount>(
                    normalized_token_pool_supported_token_lock_account,
                )?;
            let mut normalizer = normalize::NormalizedTokenPoolAdapter::new(
                normalized_token_pool_account_parsed,
                normalized_token_mint_parsed,
                normalized_token_program_parsed,
                fund_supported_token_mint_to_normalize_parsed,
                fund_supported_token_program_to_normalize_parsed,
                normalized_token_pool_supported_token_lock_account_parsed.clone(),
            )?;
            let denominated_amount_per_normalized_token =
                normalizer.get_denominated_amount_per_normalized_token()?;
            normalize::normalize_supported_token(
                &mut normalizer,
                &fund_normalized_token_account_parsed,
                &fund_supported_token_account_to_normalize_parsed,
                fund_supported_token_account_authority_to_normalize.clone(),
                &[fund_supported_token_account_authority_to_normalize_parsed
                    .get_signer_seeds()
                    .as_ref()],
                normalizing_supported_token_amount,
                // TODO: revisit later about pricing interface and dependency graph
                pricing::calculate_token_amount_as_sol(
                    fund_supported_token_mint_to_normalize.key(),
                    &pricing_source_map,
                    normalizing_supported_token_amount,
                )?,
                pricing::calculate_token_amount_as_sol(
                    normalized_token_mint.key(),
                    &pricing_source_map,
                    denominated_amount_per_normalized_token,
                )?,
            )?;
            fund_supported_token_account_to_normalize_parsed.reload()?;
            fund_supported_token_info_to_normalize.set_operation_reserved_amount(
                fund_supported_token_info_to_normalize.get_operation_reserved_amount()
                    - normalizing_supported_token_amount,
            );
            let normalized_fund_supported_token_amount = before_fund_supported_token_amount
                - fund_supported_token_account_to_normalize_parsed.amount;
            require_eq!(
                normalized_fund_supported_token_amount,
                normalizing_supported_token_amount
            );

            fund_normalized_token_account_parsed.reload()?;
            let minted_normalized_token_amount =
                fund_normalized_token_account_parsed.amount - before_fund_normalized_token_amount;
            fund_supported_token_info_to_normalize.set_operating_amount(
                fund_supported_token_info_to_normalize.get_operating_amount()
                    + normalizing_supported_token_amount,
            );
            msg!(
                "normalized {} tokens to mint {} normalized tokens",
                normalizing_supported_token_amount,
                minted_normalized_token_amount
            );

            normalized_token_pool_supported_token_lock_account_parsed.reload()?;
            require_gte!(
                minted_normalized_token_amount,
                normalizing_supported_token_amount.div_ceil(2)
            );
            require_eq!(
                fund_supported_token_info_to_normalize.get_operation_reserved_amount(),
                fund_supported_token_account_to_normalize_parsed.amount
            );
            require_eq!(
                fund_supported_token_info_to_normalize.get_operating_amount(),
                normalized_token_pool_supported_token_lock_account_parsed.amount
            );

            let normalized_token_pool_account = normalizer.into_pool_account();
            normalized_token_pool_account.exit(&crate::ID)?;
        }
    }

    // 3. restake normalized tokens
    // {
    //     fund::restake_nt_operation_reserved(
    //         fund_account,
    //         nt_opeartion_reserved_amount,
    //         // TODO: pick required accounts for this fn
    //         remaining_accounts,
    //     );
    // }

    emit!(events::OperatorProcessedJob {
        receipt_token_mint: receipt_token_mint.key(),
        fund_account: fund::FundAccountInfo::from(fund_account, receipt_token_mint,),
    });

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
