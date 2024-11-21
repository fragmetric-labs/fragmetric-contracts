use anchor_lang::prelude::*;
use anchor_spl::associated_token::{self, AssociatedToken};
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

// TODO v0.3/operation: rewrite into fund::command::...
use crate::errors::ErrorCode;
use crate::modules::{fund, normalization, pricing, restaking, staking};
use crate::utils::*;
use crate::{constants::*, events};

#[inline]
pub fn process_run<'info>(
    operator: &Signer<'info>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    fund_account: &mut Account<'info, fund::FundAccount>,
    mut remaining_accounts: &'info [AccountInfo<'info>],
    _current_timestamp: i64,
    current_slot: u64,
    command: u8,
) -> Result<()> {
    // temporary authorization
    require_keys_eq!(operator.key(), ADMIN_PUBKEY);

    match command {
        // stake sol to jitoSOL
        0 => stake_sol(receipt_token_mint, fund_account, &mut remaining_accounts)?,
        // normalize supported tokens
        // TODO: apply fund_account.nt_operation_reserved_amount
        1 => normalize_supported_tokens(receipt_token_mint, fund_account, &mut remaining_accounts)?,
        // restake normalized tokens
        2 => restake_normalized_tokens(
            operator,
            fund_account,
            &mut remaining_accounts,
            current_slot,
        )?,
        // request_withdraw normalized tokens
        3 => request_withdraw_normalized_tokens(
            operator,
            receipt_token_mint,
            fund_account,
            &mut remaining_accounts,
        )?,
        4 => withdraw_normalized_tokens(
            operator,
            receipt_token_mint,
            fund_account,
            &mut remaining_accounts,
            current_slot,
        )?,
        _ => (),
    };

    emit!(events::OperatorRanFund {
        receipt_token_mint: receipt_token_mint.key(),
        fund_account: fund::FundAccountInfo::from(fund_account, receipt_token_mint),
        executed_commands: vec![],
    });

    Ok(())
}

fn stake_sol<'info>(
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    fund_account: &mut Account<'info, fund::FundAccount>,
    remaining_accounts: &mut &'info [AccountInfo<'info>],
) -> Result<()> {
    let (fund_reserve_account, _) =
        remaining_accounts.pop_fund_reserve_account(receipt_token_mint.as_ref())?;
    let pool_program = remaining_accounts.pop_spl_stake_pool_program()?;
    let pool_account = remaining_accounts.pop_spl_stake_pool()?;
    let withdraw_authority = remaining_accounts.pop_spl_stake_pool_withdraw_authority()?;
    let reserve_stake_account = remaining_accounts.pop_spl_reserve_stake_account()?;
    let manager_fee_account = remaining_accounts.pop_spl_manager_fee_account()?;
    let pool_token_program = remaining_accounts.pop_fund_supported_token_program()?;
    let pool_mint = remaining_accounts.pop_fund_supported_token_mint(pool_token_program.key)?;
    let fund_supported_token_account_to_stake = remaining_accounts
        .pop_fund_supported_token_account(
            fund_account.as_ref(),
            pool_mint.key,
            pool_token_program.key,
        )?
        .parse_interface_account_boxed::<TokenAccount>()?;

    let staking_lamports = fund_account.sol_operation_reserved_amount;
    if staking_lamports > 0 {
        let (to_pool_token_account_amount, minted_supported_token_amount) =
            staking::SPLStakePoolService::new(
                pool_program,
                pool_account,
                pool_mint,
                &*pool_token_program,
            )?
            .deposit_sol(
                withdraw_authority,
                reserve_stake_account,
                manager_fee_account,
                fund_reserve_account,
                &fund_supported_token_account_to_stake.to_account_info(),
                &fund_account.find_reserve_account_seeds(),
                staking_lamports,
            )?;
        fund_account.sol_operation_reserved_amount = fund_account
            .sol_operation_reserved_amount
            .checked_sub(staking_lamports)
            .ok_or_else(|| error!(ErrorCode::FundUnexpectedReserveAccountBalanceException))?;

        let fund_supported_token_info = fund_account.get_supported_token_mut(&pool_mint.key())?;
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
        require_eq!(
            fund_reserve_account.lamports(),
            fund_account.sol_operation_reserved_amount
        );
    }

    Ok(())
}

fn normalize_supported_tokens<'info>(
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    fund_account: &mut Account<'info, fund::FundAccount>,
    remaining_accounts: &mut &'info [AccountInfo<'info>],
) -> Result<()> {
    let normalized_token_program = remaining_accounts.pop_normalized_token_program()?;
    let normalized_token_mint =
        remaining_accounts.pop_normalized_token_mint(normalized_token_program.key)?;
    let normalized_token_pool_account = remaining_accounts
        .pop_normalized_token_pool_account(normalized_token_mint.key)?
        .0;
    let mut fund_normalized_token_account = remaining_accounts
        .pop_fund_normalized_token_account(
            fund_account.as_ref(),
            normalized_token_mint.key,
            normalized_token_program.key,
        )?
        .parse_interface_account_boxed::<TokenAccount>()?;
    let fund_supported_token_program_to_normalize =
        remaining_accounts.pop_fund_supported_token_program()?;
    let fund_supported_token_mint_to_normalize = remaining_accounts
        .pop_fund_supported_token_mint(fund_supported_token_program_to_normalize.key)?;
    let mut fund_supported_token_account_to_normalize = remaining_accounts
        .pop_fund_supported_token_account(
            fund_account.as_ref(),
            fund_supported_token_mint_to_normalize.key,
            fund_supported_token_program_to_normalize.key,
        )?
        .parse_interface_account_boxed::<TokenAccount>()?;
    let mut normalized_token_pool_supported_token_lock_account = remaining_accounts
        .pop_normalized_token_pool_supported_token_lock_account(
            normalized_token_pool_account.key,
            fund_supported_token_mint_to_normalize.key,
            fund_supported_token_program_to_normalize.key,
        )?
        .parse_interface_account_boxed::<TokenAccount>()?;
    let pricing_source_accounts = *remaining_accounts;

    let normalizing_supported_token_amount = fund_account
        .get_supported_token_mut(&fund_supported_token_account_to_normalize.mint)?
        .get_operation_reserved_amount();
    if normalizing_supported_token_amount > 0 {
        let mut normalized_token_pool_account_parsed =
            normalized_token_pool_account.parse_account_boxed()?;
        let mut normalized_token_mint_parsed =
            normalized_token_mint.parse_interface_account_boxed()?;
        let fund_supported_token_mint_to_normalize_parsed =
            fund_supported_token_mint_to_normalize.parse_interface_account_boxed()?;

        // TODO v0.3/fund: register normalized token's pricing source from FundService::new_pricing_service_checked
        let mut pricing_service = fund::FundService::new(receipt_token_mint, fund_account)?
            .new_pricing_service(pricing_source_accounts)?
            .register_token_pricing_source_account(normalized_token_mint)
            .register_token_pricing_source_account(normalized_token_pool_account);
        pricing_service.resolve_token_pricing_source(
            &normalized_token_mint.key(),
            &pricing::TokenPricingSource::NormalizedTokenPool {
                mint_address: normalized_token_mint.key(),
                pool_address: normalized_token_pool_account.key(),
            },
        )?;

        let before_fund_normalized_token_amount = fund_normalized_token_account.amount;

        normalization::NormalizedTokenPoolService::new(
            &mut *normalized_token_pool_account_parsed,
            &mut normalized_token_mint_parsed,
            &normalized_token_program,
        )?
        .normalize_supported_token(
            &fund_normalized_token_account,
            &fund_supported_token_account_to_normalize,
            &*normalized_token_pool_supported_token_lock_account,
            &*fund_supported_token_mint_to_normalize_parsed,
            &fund_supported_token_program_to_normalize,
            &fund_account.to_account_info(),
            &[fund_account.get_signer_seeds().as_ref()],
            normalizing_supported_token_amount,
            &pricing_service,
        )?;

        fund_supported_token_account_to_normalize.reload()?;
        fund_normalized_token_account.reload()?;
        normalized_token_pool_supported_token_lock_account.reload()?;
        let minted_normalized_token_amount =
            fund_normalized_token_account.amount - before_fund_normalized_token_amount;

        let fund_supported_token_info_to_normalize = fund_account
            .get_supported_token_mut(&fund_supported_token_account_to_normalize.mint)?;
        fund_supported_token_info_to_normalize.set_operation_reserved_amount(0);
        fund_supported_token_info_to_normalize.set_operating_amount(
            fund_supported_token_info_to_normalize.get_operating_amount()
                + normalizing_supported_token_amount,
        );

        msg!(
            "normalized {} tokens to mint {} normalized tokens",
            normalizing_supported_token_amount,
            minted_normalized_token_amount
        );

        require_gte!(
            minted_normalized_token_amount,
            normalizing_supported_token_amount.div_ceil(2)
        );
        require_eq!(
            fund_supported_token_info_to_normalize.get_operation_reserved_amount(),
            fund_supported_token_account_to_normalize.amount
        );
        require_eq!(
            fund_supported_token_info_to_normalize.get_operating_amount(),
            normalized_token_pool_supported_token_lock_account.amount
        );
    }

    Ok(())
}

fn restake_normalized_tokens<'info>(
    operator: &Signer<'info>,
    fund_account: &mut Account<'info, fund::FundAccount>,
    remaining_accounts: &mut &'info [AccountInfo<'info>],
    current_slot: u64,
) -> Result<()> {
    let normalized_token_program = remaining_accounts.pop_normalized_token_program()?;
    let normalized_token_mint =
        remaining_accounts.pop_normalized_token_mint(normalized_token_program.key)?;
    let mut fund_normalized_token_account = remaining_accounts
        .pop_fund_normalized_token_account(
            fund_account.as_ref(),
            normalized_token_mint.key,
            normalized_token_program.key,
        )?
        .parse_interface_account_boxed::<TokenAccount>()?;
    let jito_vault_program = remaining_accounts.pop_jito_vault_program()?;
    let jito_vault_config = remaining_accounts.pop_jito_vault_config()?;
    let jito_vault_account = remaining_accounts.pop_jito_vault_account()?;
    let jito_vault_receipt_token_program =
        remaining_accounts.pop_jito_vault_receipt_token_program()?;
    let jito_vault_receipt_token_mint = remaining_accounts.pop_jito_vault_receipt_token_mint()?;
    let mut jito_vault_supported_token_account = remaining_accounts
        .pop_jito_vault_supported_token_account(
            normalized_token_mint.key,
            normalized_token_program.key,
        )?
        .parse_interface_account_boxed::<TokenAccount>()?;
    let jito_vault_update_state_tracker =
        remaining_accounts.pop_jito_vault_update_state_tracker(jito_vault_config, current_slot)?;
    let jito_vault_fee_wallet_token_account =
        remaining_accounts.pop_jito_vault_fee_wallet_token_account()?;
    let mut fund_jito_vault_receipt_token_account = remaining_accounts
        .pop_fund_jito_vault_receipt_token_account(fund_account.as_ref())?
        .parse_interface_account_boxed::<TokenAccount>()?;
    let system_program = remaining_accounts.pop_system_program()?;

    let restaking_nt_amount = fund_normalized_token_account.amount;
    if restaking_nt_amount > 0 {
        let before_fund_vrt_amount = fund_jito_vault_receipt_token_account.amount;

        let ctx = restaking::jito::JitoRestakingVaultContext {
            vault_program: jito_vault_program.clone(),
            vault_config: jito_vault_config.clone(),
            vault: jito_vault_account.clone(),
            vault_receipt_token_mint: jito_vault_receipt_token_mint.clone(),
            vault_receipt_token_program: jito_vault_receipt_token_program.to_account_info(),
            vault_supported_token_program: normalized_token_program.to_account_info(),
            vault_supported_token_mint: normalized_token_mint.clone(),
            vault_supported_token_account: jito_vault_supported_token_account.to_account_info(),
        };

        restaking::jito::update_vault_if_needed(
            &ctx,
            operator,
            fund_account.as_ref(),
            &[fund_account.get_signer_seeds().as_ref()],
            jito_vault_update_state_tracker,
            system_program.as_ref(),
            current_slot,
        )?;

        restaking::jito::deposit(
            &ctx,
            fund_normalized_token_account.as_ref().as_ref(),
            restaking_nt_amount,
            jito_vault_fee_wallet_token_account,
            fund_jito_vault_receipt_token_account.as_ref().as_ref(),
            restaking_nt_amount,
            fund_account.as_ref(),
            &[fund_account.get_signer_seeds().as_ref()],
        )?;

        jito_vault_supported_token_account.reload()?;
        fund_normalized_token_account.reload()?;
        fund_jito_vault_receipt_token_account.reload()?;
        let minted_fund_vrt_amount =
            fund_jito_vault_receipt_token_account.amount - before_fund_vrt_amount;

        msg!(
            "restaked {} nt to mint {} vrt",
            restaking_nt_amount,
            minted_fund_vrt_amount
        );

        require_gte!(minted_fund_vrt_amount, restaking_nt_amount);
    }

    Ok(())
}

fn request_withdraw_normalized_tokens<'info>(
    operator: &Signer<'info>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    fund_account: &mut Account<'info, fund::FundAccount>,
    remaining_accounts: &mut &'info [AccountInfo<'info>],
) -> Result<()> {
    let normalized_token_program = remaining_accounts.pop_normalized_token_program()?;
    let normalized_token_mint =
        remaining_accounts.pop_normalized_token_mint(normalized_token_program.key)?;
    let jito_vault_program = remaining_accounts.pop_jito_vault_program()?;
    let jito_vault_config = remaining_accounts.pop_jito_vault_config()?;
    let jito_vault_account = remaining_accounts.pop_jito_vault_account()?;
    let jito_vault_receipt_token_program =
        remaining_accounts.pop_jito_vault_receipt_token_program()?;
    let jito_vault_receipt_token_mint = remaining_accounts.pop_jito_vault_receipt_token_mint()?;
    let jito_vault_supported_token_account = remaining_accounts
        .pop_jito_vault_supported_token_account(
            normalized_token_mint.key,
            normalized_token_program.key,
        )?;
    let (vault_base_account, vault_base_account_bump) =
        remaining_accounts.pop_vault_base_account(receipt_token_mint.as_ref())?;
    let jito_vault_withdrawal_ticket =
        remaining_accounts.pop_jito_vault_withdrawal_ticket(vault_base_account.key, Some(false))?;
    let jito_vault_withdrawal_ticket_token_account = remaining_accounts
        .pop_jito_vault_withdrawal_ticket_token_account(
            jito_vault_withdrawal_ticket.key,
            Some(false),
        )?;
    let mut fund_jito_vault_receipt_token_account = remaining_accounts
        .pop_fund_jito_vault_receipt_token_account(fund_account.as_ref())?
        .parse_interface_account_boxed::<TokenAccount>()?;
    let system_program = remaining_accounts.pop_system_program()?;
    let associated_token_program = remaining_accounts.pop_associated_token_program()?;

    let unrestaking_fund_vrt_amount = fund_jito_vault_receipt_token_account.amount;
    if unrestaking_fund_vrt_amount > 0 {
        restaking::jito::request_withdraw(
            &restaking::jito::JitoRestakingVaultContext {
                vault_program: jito_vault_program.clone(),
                vault_config: jito_vault_config.clone(),
                vault: jito_vault_account.clone(),
                vault_receipt_token_mint: jito_vault_receipt_token_mint.clone(),
                vault_receipt_token_program: jito_vault_receipt_token_program.to_account_info(),
                vault_supported_token_program: normalized_token_program.to_account_info(),
                vault_supported_token_mint: normalized_token_mint.clone(),
                vault_supported_token_account: jito_vault_supported_token_account.clone(),
            },
            operator,
            jito_vault_withdrawal_ticket,
            jito_vault_withdrawal_ticket_token_account,
            fund_jito_vault_receipt_token_account.as_ref().as_ref(),
            vault_base_account,
            system_program.as_ref(),
            associated_token_program.as_ref(),
            fund_account.as_ref(),
            &[
                fund_account.get_signer_seeds().as_ref(),
                &[
                    restaking::jito::JitoRestakingVault::VAULT_BASE_ACCOUNT1_SEED,
                    &receipt_token_mint.key().to_bytes(),
                    &[vault_base_account_bump],
                ],
            ],
            unrestaking_fund_vrt_amount,
        )?;

        fund_jito_vault_receipt_token_account.reload()?;

        require_eq!(fund_jito_vault_receipt_token_account.amount, 0,);

        msg!("requested unrestaking {} vrt", unrestaking_fund_vrt_amount,);
    }

    Ok(())
}

fn withdraw_normalized_tokens<'info>(
    operator: &Signer<'info>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    fund_account: &mut Account<'info, fund::FundAccount>,
    remaining_accounts: &mut &'info [AccountInfo<'info>],
    current_slot: u64,
) -> Result<()> {
    let normalized_token_program = remaining_accounts.pop_normalized_token_program()?;
    let normalized_token_mint =
        remaining_accounts.pop_normalized_token_mint(normalized_token_program.key)?;
    let fund_normalized_token_account = remaining_accounts.pop_fund_normalized_token_account(
        fund_account.as_ref(),
        normalized_token_mint.key,
        normalized_token_program.key,
    )?;
    let jito_vault_program = remaining_accounts.pop_jito_vault_program()?;
    let jito_vault_config = remaining_accounts.pop_jito_vault_config()?;
    let jito_vault_account = remaining_accounts.pop_jito_vault_account()?;
    let jito_vault_receipt_token_program =
        remaining_accounts.pop_jito_vault_receipt_token_program()?;
    let jito_vault_receipt_token_mint = remaining_accounts.pop_jito_vault_receipt_token_mint()?;
    let jito_vault_supported_token_account = remaining_accounts
        .pop_jito_vault_supported_token_account(
            normalized_token_mint.key,
            normalized_token_program.key,
        )?;
    let jito_vault_update_state_tracker =
        remaining_accounts.pop_jito_vault_update_state_tracker(jito_vault_config, current_slot)?;
    let jito_vault_withdrawal_ticket = remaining_accounts.pop_jito_vault_withdrawal_ticket(
        &find_vault_base_account_address(receipt_token_mint.as_ref()).0,
        Some(true),
    )?;
    let jito_vault_withdrawal_ticket_token_account = remaining_accounts
        .pop_jito_vault_withdrawal_ticket_token_account(
            jito_vault_withdrawal_ticket.key,
            Some(true),
        )?
        .parse_interface_account_boxed::<TokenAccount>()?;
    let jito_vault_fee_wallet_token_account =
        remaining_accounts.pop_jito_vault_fee_wallet_token_account()?;
    let jito_vault_program_fee_wallet_token_account =
        remaining_accounts.pop_jito_vault_program_fee_wallet_token_account()?;
    let system_program = remaining_accounts.pop_system_program()?;

    let ctx = restaking::jito::JitoRestakingVaultContext {
        vault_program: jito_vault_program.clone(),
        vault_config: jito_vault_config.clone(),
        vault: jito_vault_account.clone(),
        vault_receipt_token_mint: jito_vault_receipt_token_mint.clone(),
        vault_receipt_token_program: jito_vault_receipt_token_program.to_account_info(),
        vault_supported_token_mint: normalized_token_mint.clone(),
        vault_supported_token_program: normalized_token_program.to_account_info(),
        vault_supported_token_account: jito_vault_supported_token_account.to_account_info(),
    };

    let withdrawing_vrt_amount = jito_vault_withdrawal_ticket_token_account.amount;
    if withdrawing_vrt_amount > 0 {
        restaking::jito::update_vault_if_needed(
            &ctx,
            operator,
            fund_account.as_ref(),
            &[fund_account.get_signer_seeds().as_ref()],
            jito_vault_update_state_tracker,
            system_program.as_ref(),
            current_slot,
        )?;

        restaking::jito::withdraw(
            &ctx,
            jito_vault_withdrawal_ticket, // vault_withdrawal_ticket
            jito_vault_withdrawal_ticket_token_account.as_ref().as_ref(), // vault_withdrawal_ticket_token_account
            fund_normalized_token_account,
            jito_vault_fee_wallet_token_account,
            jito_vault_program_fee_wallet_token_account,
            fund_account.as_ref(), // signer
            system_program.as_ref(),
        )?;
    }

    Ok(())
}

trait RemainingAccounts<'info> {
    fn pop_account_info_with_seeds(
        &mut self,
        seeds: &[&[u8]],
        program_id: &Pubkey,
        owner: Option<&Pubkey>,
        account_name: impl ToString + Copy,
    ) -> Result<(&'info AccountInfo<'info>, u8)>;
    fn pop_account_info_with_address(
        &mut self,
        address: &Pubkey,
        owner: Option<&Pubkey>,
        account_name: impl ToString + Copy,
    ) -> Result<&'info AccountInfo<'info>>;
    fn pop_account_info(
        &mut self,
        owner: Option<&Pubkey>,
        account_name: impl ToString + Copy,
    ) -> Result<&'info AccountInfo<'info>>;
    fn pop_associated_token_account_info(
        &mut self,
        mint: &Pubkey,
        authority: &Pubkey,
        token_program: &Pubkey,
        owner: Option<&Pubkey>,
        account_name: impl ToString + Copy,
    ) -> Result<&'info AccountInfo<'info>>;

    fn pop_system_program(&mut self) -> Result<Program<'info, System>>;
    fn pop_associated_token_program(&mut self) -> Result<Program<'info, AssociatedToken>>;

    // Fund

    fn pop_fund_reserve_account(
        &mut self,
        receipt_token_mint: &AccountInfo,
    ) -> Result<(&'info AccountInfo<'info>, u8)>;

    // SPL

    fn pop_spl_stake_pool_program(&mut self) -> Result<&'info AccountInfo<'info>>;
    fn pop_spl_stake_pool(&mut self) -> Result<&'info AccountInfo<'info>>;
    fn pop_spl_stake_pool_withdraw_authority(&mut self) -> Result<&'info AccountInfo<'info>>;
    fn pop_spl_reserve_stake_account(&mut self) -> Result<&'info AccountInfo<'info>>;
    fn pop_spl_manager_fee_account(&mut self) -> Result<&'info AccountInfo<'info>>;

    // Supported Tokens

    fn pop_fund_supported_token_program(&mut self) -> Result<Interface<'info, TokenInterface>>;
    fn pop_fund_supported_token_mint(
        &mut self,
        supported_token_program: &Pubkey,
    ) -> Result<&'info AccountInfo<'info>>;
    fn pop_fund_supported_token_account(
        &mut self,
        fund_account: &AccountInfo,
        supported_token_mint: &Pubkey,
        supported_token_program: &Pubkey,
    ) -> Result<&'info AccountInfo<'info>>;
    fn pop_normalized_token_pool_supported_token_lock_account(
        &mut self,
        normalized_token_pool_account: &Pubkey,
        supported_token_mint: &Pubkey,
        supported_token_program: &Pubkey,
    ) -> Result<&'info AccountInfo<'info>>;

    // NTP

    fn pop_normalized_token_pool_account(
        &mut self,
        normalized_token_mint: &Pubkey,
    ) -> Result<(&'info AccountInfo<'info>, u8)>;

    // Normalized Tokens

    fn pop_normalized_token_program(&mut self) -> Result<Program<'info, Token>>;
    fn pop_normalized_token_mint(
        &mut self,
        normalized_token_program: &Pubkey,
    ) -> Result<&'info AccountInfo<'info>>;
    fn pop_fund_normalized_token_account(
        &mut self,
        fund_account: &AccountInfo,
        normalized_token_mint: &Pubkey,
        normalized_token_program: &Pubkey,
    ) -> Result<&'info AccountInfo<'info>>;
    fn pop_jito_vault_supported_token_account(
        &mut self,
        normalized_token_mint: &Pubkey,
        normalized_token_program: &Pubkey,
    ) -> Result<&'info AccountInfo<'info>>;

    // Restaking

    fn pop_vault_base_account(
        &mut self,
        receipt_token_mint: &AccountInfo,
    ) -> Result<(&'info AccountInfo<'info>, u8)>;

    // Jito

    fn pop_jito_vault_program(&mut self) -> Result<&'info AccountInfo<'info>>;
    fn pop_jito_vault_config(&mut self) -> Result<&'info AccountInfo<'info>>;
    fn pop_jito_vault_account(&mut self) -> Result<&'info AccountInfo<'info>>;
    fn pop_jito_vault_update_state_tracker(
        &mut self,
        jito_vault_config: &AccountInfo,
        current_slot: u64,
    ) -> Result<&'info AccountInfo<'info>>;

    fn pop_jito_vault_withdrawal_ticket(
        &mut self,
        vault_base_account: &Pubkey,
        initialized: Option<bool>,
    ) -> Result<&'info AccountInfo<'info>>;

    fn pop_jito_vault_receipt_token_program(&mut self) -> Result<Program<'info, Token>>;
    fn pop_jito_vault_receipt_token_mint(&mut self) -> Result<&'info AccountInfo<'info>>;
    fn pop_jito_vault_fee_wallet_token_account(&mut self) -> Result<&'info AccountInfo<'info>>;
    fn pop_jito_vault_program_fee_wallet_token_account(
        &mut self,
    ) -> Result<&'info AccountInfo<'info>>;
    fn pop_fund_jito_vault_receipt_token_account(
        &mut self,
        fund_account: &AccountInfo,
    ) -> Result<&'info AccountInfo<'info>>;
    fn pop_jito_vault_withdrawal_ticket_token_account(
        &mut self,
        jito_vault_withdrawal_ticket: &Pubkey,
        initialized: Option<bool>,
    ) -> Result<&'info AccountInfo<'info>>;
}

impl<'info> RemainingAccounts<'info> for &'info [AccountInfo<'info>] {
    fn pop_account_info_with_seeds(
        &mut self,
        seeds: &[&[u8]],
        program_id: &Pubkey,
        owner: Option<&Pubkey>,
        account_name: impl ToString + Copy,
    ) -> Result<(&'info AccountInfo<'info>, u8)> {
        let (address, bump) = Pubkey::find_program_address(seeds, program_id);
        Ok((
            self.pop_account_info_with_address(&address, owner, account_name)?,
            bump,
        ))
    }

    fn pop_account_info_with_address(
        &mut self,
        address: &Pubkey,
        owner: Option<&Pubkey>,
        account_name: impl ToString + Copy,
    ) -> Result<&'info AccountInfo<'info>> {
        use anchor_lang::error::*;

        let account = self.pop_account_info(owner, account_name)?;
        if account.key != address {
            return Err(Error::from(ErrorCode::ConstraintAddress)
                .with_pubkeys((*account.key, *address))
                .with_account_name(account_name))?;
        }

        Ok(account)
    }

    fn pop_account_info(
        &mut self,
        owner: Option<&Pubkey>,
        account_name: impl ToString + Copy,
    ) -> Result<&'info AccountInfo<'info>> {
        use anchor_lang::error::*;

        if self.is_empty() {
            return Err(
                Error::from(ErrorCode::AccountNotEnoughKeys).with_account_name(account_name)
            );
        }
        let account = &self[0];
        *self = &self[1..];

        if let Some(owner) = owner {
            if account.owner != owner {
                return Err(Error::from(ErrorCode::ConstraintOwner)
                    .with_pubkeys((*account.owner, *owner))
                    .with_account_name(account_name));
            }
        }

        Ok(account)
    }

    fn pop_associated_token_account_info(
        &mut self,
        mint: &Pubkey,
        authority: &Pubkey,
        token_program: &Pubkey,
        owner: Option<&Pubkey>,
        account_name: impl ToString + Copy,
    ) -> Result<&'info AccountInfo<'info>> {
        let address = associated_token::get_associated_token_address_with_program_id(
            authority,
            mint,
            token_program,
        );
        self.pop_account_info_with_address(&address, owner, account_name)
    }

    #[inline(never)]
    fn pop_system_program(&mut self) -> Result<Program<'info, System>> {
        Ok(Program::try_from(
            self.pop_account_info(None, "system_program")?,
        )?)
    }

    #[inline(never)]
    fn pop_associated_token_program(&mut self) -> Result<Program<'info, AssociatedToken>> {
        Ok(Program::try_from(
            self.pop_account_info(None, "associated_token_program")?,
        )?)
    }

    #[inline(never)]
    fn pop_fund_reserve_account(
        &mut self,
        receipt_token_mint: &AccountInfo,
    ) -> Result<(&'info AccountInfo<'info>, u8)> {
        self.pop_account_info_with_seeds(
            &[
                fund::FundAccount::RESERVE_SEED,
                receipt_token_mint.key().as_ref(),
            ],
            &crate::ID,
            Some(&Pubkey::default()),
            "fund_reserve_account",
        )
    }

    #[inline(never)]
    fn pop_spl_stake_pool_program(&mut self) -> Result<&'info AccountInfo<'info>> {
        self.pop_account_info_with_address(&spl_stake_pool::ID, None, "spl_stake_pool_program")
    }

    #[inline(never)]
    fn pop_spl_stake_pool(&mut self) -> Result<&'info AccountInfo<'info>> {
        self.pop_account_info(Some(&spl_stake_pool::ID), "spl_stake_pool")
    }

    #[inline(never)]
    fn pop_spl_stake_pool_withdraw_authority(&mut self) -> Result<&'info AccountInfo<'info>> {
        self.pop_account_info(None, "spl_stake_pool_withdraw_authority")
    }

    #[inline(never)]
    fn pop_spl_reserve_stake_account(&mut self) -> Result<&'info AccountInfo<'info>> {
        self.pop_account_info(None, "spl_reserve_stake_account")
    }

    #[inline(never)]
    fn pop_spl_manager_fee_account(&mut self) -> Result<&'info AccountInfo<'info>> {
        self.pop_account_info(None, "spl_manager_fee_account")
    }

    #[inline(never)]
    fn pop_fund_supported_token_program(&mut self) -> Result<Interface<'info, TokenInterface>> {
        Interface::try_from(self.pop_account_info(None, "fund_supported_token_program")?).map_err(
            |e| {
                anchor_lang::error::Error::from(e).with_account_name("fund_supported_token_program")
            },
        )
    }

    #[inline(never)]
    fn pop_fund_supported_token_mint(
        &mut self,
        supported_token_program: &Pubkey,
    ) -> Result<&'info AccountInfo<'info>> {
        self.pop_account_info(Some(supported_token_program), "fund_supported_token_mint")
    }

    #[inline(never)]
    fn pop_fund_supported_token_account(
        &mut self,
        fund_account: &AccountInfo,
        supported_token_mint: &Pubkey,
        supported_token_program: &Pubkey,
    ) -> Result<&'info AccountInfo<'info>> {
        self.pop_associated_token_account_info(
            supported_token_mint,
            fund_account.key,
            supported_token_program,
            Some(supported_token_program),
            "fund_supported_token_account",
        )
    }

    #[inline(never)]
    fn pop_normalized_token_pool_supported_token_lock_account(
        &mut self,
        normalized_token_pool_account: &Pubkey,
        supported_token_mint: &Pubkey,
        supported_token_program: &Pubkey,
    ) -> Result<&'info AccountInfo<'info>> {
        self.pop_associated_token_account_info(
            supported_token_mint,
            normalized_token_pool_account,
            supported_token_program,
            Some(supported_token_program),
            "normalized_token_pool_supported_token_lock_account",
        )
    }

    #[inline(never)]
    fn pop_normalized_token_pool_account(
        &mut self,
        normalized_token_mint: &Pubkey,
    ) -> Result<(&'info AccountInfo<'info>, u8)> {
        self.pop_account_info_with_seeds(
            &[
                normalization::NormalizedTokenPoolAccount::SEED,
                normalized_token_mint.as_ref(),
            ],
            &crate::ID,
            Some(&crate::ID),
            "normalized_token_pool_account",
        )
    }

    #[inline(never)]
    fn pop_normalized_token_program(&mut self) -> Result<Program<'info, Token>> {
        Program::try_from(self.pop_account_info(None, "normalized_token_program")?).map_err(|e| {
            anchor_lang::error::Error::from(e).with_account_name("normalized_token_program")
        })
    }

    #[inline(never)]
    fn pop_normalized_token_mint(
        &mut self,
        normalized_token_program: &Pubkey,
    ) -> Result<&'info AccountInfo<'info>> {
        self.pop_account_info(Some(normalized_token_program), "normalized_token_mint")
    }

    #[inline(never)]
    fn pop_fund_normalized_token_account(
        &mut self,
        fund_account: &AccountInfo,
        normalized_token_mint: &Pubkey,
        normalized_token_program: &Pubkey,
    ) -> Result<&'info AccountInfo<'info>> {
        self.pop_associated_token_account_info(
            normalized_token_mint,
            fund_account.key,
            normalized_token_program,
            Some(normalized_token_program),
            "fund_normalized_token_account",
        )
    }

    #[inline(never)]
    fn pop_jito_vault_supported_token_account(
        &mut self,
        normalized_token_mint: &Pubkey,
        normalized_token_program: &Pubkey,
    ) -> Result<&'info AccountInfo<'info>> {
        self.pop_associated_token_account_info(
            normalized_token_mint,
            &FRAGSOL_JITO_VAULT_ACCOUNT_ADDRESS,
            normalized_token_program,
            Some(normalized_token_program),
            "jito_vault_supported_token_account",
        )
    }

    #[inline(never)]
    fn pop_vault_base_account(
        &mut self,
        receipt_token_mint: &AccountInfo,
    ) -> Result<(&'info AccountInfo<'info>, u8)> {
        let (address, bump) = find_vault_base_account_address(receipt_token_mint);
        Ok((
            self.pop_account_info_with_address(
                &address,
                Some(&Pubkey::default()),
                "vault_base_account",
            )?,
            bump,
        ))
    }

    #[inline(never)]
    fn pop_jito_vault_program(&mut self) -> Result<&'info AccountInfo<'info>> {
        self.pop_account_info_with_address(&JITO_VAULT_PROGRAM_ID, None, "jito_vault_program")
    }

    #[inline(never)]
    fn pop_jito_vault_config(&mut self) -> Result<&'info AccountInfo<'info>> {
        self.pop_account_info_with_address(
            &FRAGSOL_JITO_VAULT_CONFIG_ADDRESS,
            None,
            "jito_vault_config",
        )
    }

    #[inline(never)]
    fn pop_jito_vault_account(&mut self) -> Result<&'info AccountInfo<'info>> {
        self.pop_account_info_with_address(
            &FRAGSOL_JITO_VAULT_ACCOUNT_ADDRESS,
            None,
            "jito_vault_account",
        )
    }

    #[inline(never)]
    fn pop_jito_vault_update_state_tracker(
        &mut self,
        jito_vault_config: &AccountInfo,
        current_slot: u64,
    ) -> Result<&'info AccountInfo<'info>> {
        use anchor_lang::error::*;
        use jito_bytemuck::AccountDeserialize;
        use jito_vault_core::config::Config;

        let data = jito_vault_config
            .try_borrow_data()
            .map_err(|e| Error::from(e).with_account_name("jito_vault_cupdate_state_tracker"))?;
        let config = Config::try_from_slice_unchecked(&data)
            .map_err(|e| Error::from(e).with_account_name("jito_vault_cupdate_state_tracker"))?;
        let ncn_epoch = current_slot
            .checked_div(config.epoch_length())
            .ok_or_else(|| error!(crate::errors::ErrorCode::CalculationArithmeticException))?;
        msg!("Current epoch: {}", ncn_epoch);

        Ok(self
            .pop_account_info_with_seeds(
                &[
                    b"vault_update_state_tracker",
                    FRAGSOL_JITO_VAULT_ACCOUNT_ADDRESS.as_ref(),
                    &ncn_epoch.to_le_bytes(),
                ],
                &JITO_VAULT_PROGRAM_ID,
                Some(&Pubkey::default()),
                "jito_vault_update_state_tracker",
            )?
            .0)
    }

    #[inline(never)]
    fn pop_jito_vault_withdrawal_ticket(
        &mut self,
        vault_base_account: &Pubkey,
        initialized: Option<bool>,
    ) -> Result<&'info AccountInfo<'info>> {
        Ok(self
            .pop_account_info_with_seeds(
                &[
                    b"vault_staker_withdrawal_ticket",
                    FRAGSOL_JITO_VAULT_ACCOUNT_ADDRESS.as_ref(),
                    vault_base_account.as_ref(),
                ],
                &JITO_VAULT_PROGRAM_ID,
                initialized
                    .map(|initialized| {
                        initialized
                            .then_some(JITO_VAULT_PROGRAM_ID)
                            .unwrap_or(Pubkey::default())
                    })
                    .as_ref(),
                "jito_vault_withdrawal_ticket",
            )?
            .0)
    }

    #[inline(never)]
    fn pop_jito_vault_receipt_token_program(&mut self) -> Result<Program<'info, Token>> {
        Program::try_from(self.pop_account_info(None, "jito_vault_receipt_token_program")?).map_err(
            |e| {
                anchor_lang::error::Error::from(e)
                    .with_account_name("jito_vault_receipt_token_program")
            },
        )
    }

    #[inline(never)]
    fn pop_jito_vault_receipt_token_mint(&mut self) -> Result<&'info AccountInfo<'info>> {
        self.pop_account_info_with_address(
            &FRAGSOL_JITO_VAULT_RECEIPT_TOKEN_MINT_ADDRESS,
            Some(&Token::id()),
            "jito_vault_receipt_token_mint",
        )
    }

    #[inline(never)]
    fn pop_jito_vault_fee_wallet_token_account(&mut self) -> Result<&'info AccountInfo<'info>> {
        self.pop_associated_token_account_info(
            &FRAGSOL_JITO_VAULT_RECEIPT_TOKEN_MINT_ADDRESS,
            &ADMIN_PUBKEY,
            &Token::id(),
            Some(&Token::id()),
            "jito_vault_fee_wallet_token_account",
        )
    }

    #[inline(never)]
    fn pop_jito_vault_program_fee_wallet_token_account(
        &mut self,
    ) -> Result<&'info AccountInfo<'info>> {
        self.pop_associated_token_account_info(
            &FRAGSOL_JITO_VAULT_RECEIPT_TOKEN_MINT_ADDRESS,
            &JITO_VAULT_PROGRAM_FEE_WALLET,
            &Token::id(),
            Some(&Token::id()),
            "jito_vault_program_fee_wallet_token_account",
        )
    }

    #[inline(never)]
    fn pop_fund_jito_vault_receipt_token_account(
        &mut self,
        fund_account: &AccountInfo,
    ) -> Result<&'info AccountInfo<'info>> {
        self.pop_associated_token_account_info(
            &FRAGSOL_JITO_VAULT_RECEIPT_TOKEN_MINT_ADDRESS,
            fund_account.key,
            &Token::id(),
            Some(&Token::id()),
            "fund_jito_vault_receipt_token_account",
        )
    }

    #[inline(never)]
    fn pop_jito_vault_withdrawal_ticket_token_account(
        &mut self,
        jito_vault_withdrawal_ticket: &Pubkey,
        initialized: Option<bool>,
    ) -> Result<&'info AccountInfo<'info>> {
        self.pop_associated_token_account_info(
            &FRAGSOL_JITO_VAULT_RECEIPT_TOKEN_MINT_ADDRESS,
            jito_vault_withdrawal_ticket,
            &Token::id(),
            initialized
                .map(|initialized| {
                    initialized
                        .then_some(Token::id())
                        .unwrap_or(Pubkey::default())
                })
                .as_ref(),
            "jito_vault_withdrawal_ticket_token_account",
        )
    }
}

fn find_vault_base_account_address(receipt_token_mint: &AccountInfo) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            restaking::jito::JitoRestakingVault::VAULT_BASE_ACCOUNT1_SEED,
            receipt_token_mint.key.as_ref(),
        ],
        &crate::ID,
    )
}
