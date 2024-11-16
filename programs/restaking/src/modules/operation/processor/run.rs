use crate::constants::ADMIN_PUBKEY;
use crate::errors;
use crate::events;
use crate::modules::{fund, normalization, pricing, restaking, staking};
use crate::utils::*;
use anchor_lang::{prelude::*, solana_program, CheckOwner};
use anchor_spl::token::accessor::mint;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use std::clone;

// TODO v0.3/operation: rewrite into fund::command::...
pub fn process_run<'info>(
    operator: &Signer<'info>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    fund_account: &mut Account<'info, fund::FundAccount>,
    remaining_accounts: &'info [AccountInfo<'info>],
    _current_timestamp: i64,
    _current_slot: u64,
    command: u8, // 0, 1, 2
) -> Result<()> {
    require_eq!(operator.key(), ADMIN_PUBKEY);

    // stake sol to jitoSOL
    if command == 0 {
        // accounts
        let [
            // staking
            fund_reserve_account,
            pool_program,
            pool_account,
            withdraw_authority,
            reserve_stake_account,
            manager_fee_account,
            pool_mint,
            pool_token_program,
            fund_supported_token_account_to_stake,
            ..,
        ] = remaining_accounts else {
            return Err(ProgramError::NotEnoughAccountKeys)?;
        };

        let staking_lamports = fund_account.sol_operation_reserved_amount;
        if staking_lamports > 0 {
            let (to_pool_token_account_amount, minted_supported_token_amount) = staking::SPLStakePoolService::new(
                pool_program,
                pool_account,
                pool_mint,
                pool_token_program,
            )?
                .deposit_sol(
                    withdraw_authority,
                    reserve_stake_account,
                    manager_fee_account,
                    fund_reserve_account,
                    fund_supported_token_account_to_stake,
                    &fund_account.find_reserve_account_seeds(),
                    staking_lamports,
                )?;
            fund_account.sol_operation_reserved_amount = fund_account
                .sol_operation_reserved_amount
                .checked_sub(staking_lamports)
                .ok_or_else(|| {
                    error!(
                        errors::ErrorCode::FundUnexpectedReserveAccountBalanceException
                    )
                })?;

            let fund_supported_token_info = fund_account
                .get_supported_token_mut(&pool_mint.key())?;
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
                to_pool_token_account_amount,
            );
        }
    }


    // normalize supported tokens
    if command == 1 {
        let [
            // normalization
            normalized_token_pool_account,
            normalized_token_mint,
            normalized_token_program,
            fund_normalized_token_account,
            fund_supported_token_account_to_normalize,
            fund_supported_token_mint_to_normalize,
            fund_supported_token_program_to_normalize,
            normalized_token_pool_supported_token_lock_account,
            // pricing
            pricing_sources @ ..,
        ] = remaining_accounts else {
            return Err(ProgramError::NotEnoughAccountKeys)?;
        };

        // create pricing service from fund state
        let mut pricing_service = fund::FundService::new(
            receipt_token_mint,
            fund_account,
        )?
            .new_pricing_service(&pricing_sources)?;

        let mut normalized_token_pool_account_parsed =
            parse_account_boxed(normalized_token_pool_account)?;
        let mut fund_supported_token_account_to_normalize_parsed =
            parse_interface_account_boxed::<TokenAccount>(
                fund_supported_token_account_to_normalize,
            )?;
        let fund_supported_token_info_to_normalize = fund_account
            .get_supported_token_mut(&fund_supported_token_account_to_normalize_parsed.mint)?;
        let mut fund_normalized_token_account_parsed =
            parse_interface_account_boxed::<TokenAccount>(fund_normalized_token_account)?;

        let normalizing_supported_token_amount =
            fund_supported_token_info_to_normalize.get_operation_reserved_amount();
        if normalizing_supported_token_amount > 0 {
            let before_fund_supported_token_amount =
                fund_supported_token_account_to_normalize_parsed.amount;
            let before_fund_normalized_token_amount = fund_normalized_token_account_parsed.amount;
            let mut normalized_token_mint_parsed =
                parse_interface_account_boxed(normalized_token_mint)?;
            let normalized_token_program_parsed = parse_program_boxed::<Token>(normalized_token_program)?;
            let fund_supported_token_mint_to_normalize_parsed =
                parse_interface_account_boxed(fund_supported_token_mint_to_normalize)?;

            let fund_supported_token_program_to_normalize_parsed = parse_interface_boxed::<TokenInterface>(fund_supported_token_program_to_normalize)?;
            let mut normalized_token_pool_supported_token_lock_account_parsed =
                parse_interface_account_boxed::<TokenAccount>(
                    normalized_token_pool_supported_token_lock_account,
                )?;
            let mut normalizer = normalization::NormalizedTokenPoolService::new(
                &mut *normalized_token_pool_account_parsed,
                &mut normalized_token_mint_parsed,
                &normalized_token_program_parsed,
            )?;

            // TODO v0.3/fund: register normalized token's pricing source from FundService::new_pricing_service_checked
            pricing_service
                .register_token_pricing_source_account(normalized_token_mint.as_ref())
                .register_token_pricing_source_account(normalized_token_pool_account.as_ref())
                .resolve_token_pricing_source(
                    &normalized_token_mint.key(),
                    &pricing::TokenPricingSource::NormalizedTokenPool {
                        mint_address: normalized_token_mint.key(),
                        pool_address: normalized_token_pool_account.key(),
                    },
                )?;

            normalizer.normalize_supported_token(
                &fund_normalized_token_account_parsed,
                &fund_supported_token_account_to_normalize_parsed,
                &normalized_token_pool_supported_token_lock_account_parsed,
                &fund_supported_token_mint_to_normalize_parsed,
                &fund_supported_token_program_to_normalize_parsed,
                fund_account.to_account_info().as_ref(),
                &[fund_account.get_signer_seeds().as_ref()],
                normalizing_supported_token_amount,

                &pricing_service,
            )?;
            fund_supported_token_account_to_normalize_parsed.reload()?;
            let fund_supported_token_info_to_normalize = fund_account
                .get_supported_token_mut(&fund_supported_token_account_to_normalize_parsed.mint)?;
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
        }
    }

    // restake normalized tokens
    if command == 2 {
        let [
            // normalization
            normalized_token_mint,
            normalized_token_program,
            fund_normalized_token_account,
            // restaking
            jito_vault_program,
            jito_vault_config,
            jito_vault_account,
            jito_vault_receipt_token_mint,
            jito_vault_receipt_token_program,
            jito_vault_supported_token_account,
            jito_vault_fee_receipt_token_account,
            fund_jito_vault_receipt_token_account,
            ..,
        ] = remaining_accounts else {
            return Err(ProgramError::NotEnoughAccountKeys)?;
        };

        let fund_normalized_token_account_parsed =
            parse_interface_account_boxed::<TokenAccount>(fund_normalized_token_account)?;
        let restaking_nt_amount = fund_normalized_token_account_parsed.amount;

        if restaking_nt_amount > 0 {
            let mut fund_jito_vault_receipt_token_account_parsed =
                parse_interface_account_boxed::<TokenAccount>(fund_jito_vault_receipt_token_account)?;
            let before_fund_vrt_amount = fund_jito_vault_receipt_token_account_parsed.amount;

            restaking::jito::deposit(
                &restaking::jito::JitoRestakingVaultContext {
                    vault_program: jito_vault_program.clone(),
                    vault_config: jito_vault_config.clone(),
                    vault: jito_vault_account.clone(),
                    vault_receipt_token_mint: jito_vault_receipt_token_mint.clone(),
                    vault_receipt_token_program: jito_vault_receipt_token_program.clone(),
                    vault_supported_token_mint: normalized_token_mint.clone(),
                    vault_supported_token_program: normalized_token_program.clone(),
                    vault_supported_token_account: jito_vault_supported_token_account.clone(),
                },
                fund_normalized_token_account, // supported_token_account,
                restaking_nt_amount, // supported_token_amount_in,

                jito_vault_fee_receipt_token_account,
                fund_jito_vault_receipt_token_account, // vault_receipt_token_account,
                restaking_nt_amount, // vault_receipt_token_min_amount_out,

                fund_account.as_ref(), // signer

                &[fund_account.get_signer_seeds().as_ref()],
            )?;

            fund_jito_vault_receipt_token_account_parsed.reload()?;
            let minted_fund_vrt_amount = fund_jito_vault_receipt_token_account_parsed.amount - before_fund_vrt_amount;

            msg!(
                "restaked {} nt to mint {} vrt",
                restaking_nt_amount,
                minted_fund_vrt_amount
            );
            require_gte!(minted_fund_vrt_amount, restaking_nt_amount);
        }
    }

    emit!(events::OperatorProcessedJob {
        receipt_token_mint: receipt_token_mint.key(),
        fund_account: fund::FundAccountInfo::from(fund_account, receipt_token_mint),
    });

    Ok(())
}
