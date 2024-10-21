use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use spl_tlv_account_resolution::state::ExtraAccountMetaList;
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

use crate::events;

use super::*;

pub fn update_fund_account_if_needed(
    fund_account: &mut FundAccount,
    receipt_token_mint: Pubkey,
) -> Result<()> {
    fund_account.update_if_needed(receipt_token_mint);
    Ok(())
}

pub fn update_user_fund_account_if_needed(
    user_fund_account: &mut UserFundAccount,
    receipt_token_mint: Pubkey,
    user: Pubkey,
) -> Result<()> {
    user_fund_account.update_if_needed(receipt_token_mint, user);
    Ok(())
}

pub fn update_extra_account_meta_list_if_needed(
    extra_account_meta_list: &AccountInfo,
) -> Result<()> {
    ExtraAccountMetaList::update::<ExecuteInstruction>(
        &mut extra_account_meta_list.try_borrow_mut_data()?,
        &extra_account_metas()?,
    )?;
    Ok(())
}

pub fn update_sol_capacity_amount(
    fund_account: &mut FundAccount,
    receipt_token_mint: &Mint,
    capacity_amount: u64,
) -> Result<()> {
    fund_account.set_sol_capacity_amount(capacity_amount)?;
    emit_fund_manager_updated_fund_event(
        fund_account,
        receipt_token_mint,
        fund_account.receipt_token_mint,
    )
}

pub fn update_supported_token_capacity_amount(
    fund_account: &mut FundAccount,
    receipt_token_mint: &Mint,
    token: Pubkey,
    capacity_amount: u64,
) -> Result<()> {
    fund_account
        .supported_token_mut(token)?
        .set_capacity_amount(capacity_amount)?;
    emit_fund_manager_updated_fund_event(
        fund_account,
        receipt_token_mint,
        fund_account.receipt_token_mint,
    )
}

pub fn update_withdrawal_enabled_flag(
    fund_account: &mut FundAccount,
    receipt_token_mint: &Mint,
    enabled: bool,
) -> Result<()> {
    fund_account
        .withdrawal_status
        .set_withdrawal_enabled_flag(enabled);
    emit_fund_manager_updated_fund_event(
        fund_account,
        receipt_token_mint,
        fund_account.receipt_token_mint,
    )
}

pub fn update_sol_withdrawal_fee_rate(
    fund_account: &mut FundAccount,
    receipt_token_mint: &Mint,
    sol_withdrawal_fee_rate: u16,
) -> Result<()> {
    fund_account
        .withdrawal_status
        .set_sol_withdrawal_fee_rate(sol_withdrawal_fee_rate);
    emit_fund_manager_updated_fund_event(
        fund_account,
        receipt_token_mint,
        fund_account.receipt_token_mint,
    )
}

pub fn update_batch_processing_threshold(
    fund_account: &mut FundAccount,
    receipt_token_mint: &Mint,
    amount: Option<u64>,
    duration: Option<i64>,
) -> Result<()> {
    fund_account
        .withdrawal_status
        .set_batch_processing_threshold(amount, duration);
    emit_fund_manager_updated_fund_event(
        fund_account,
        receipt_token_mint,
        fund_account.receipt_token_mint,
    )
}

pub fn add_supported_token(
    fund_account: &mut FundAccount,
    receipt_token_mint: &Mint,
    supported_token_mint: &Mint,
    supported_token_mint_address: Pubkey,
    supported_token_program: Pubkey,
    capacity_amount: u64,
    pricing_source: TokenPricingSource,
    pricing_sources: &[AccountInfo],
) -> Result<()> {
    fund_account.add_supported_token(
        supported_token_mint_address,
        supported_token_program,
        supported_token_mint.decimals,
        capacity_amount,
        pricing_source,
    )?;
    fund_account.update_token_prices(pricing_sources)?;
    emit_fund_manager_updated_fund_event(
        fund_account,
        receipt_token_mint,
        fund_account.receipt_token_mint,
    )
}

pub fn update_prices(
    fund_account: &mut FundAccount,
    receipt_token_mint: &Mint,
    pricing_sources: &[AccountInfo],
) -> Result<()> {
    fund_account.update_token_prices(pricing_sources)?;

    emit!(events::OperatorUpdatedFundPrice {
        receipt_token_mint: fund_account.receipt_token_mint,
        fund_account: FundAccountInfo::new(
            fund_account,
            fund_account.receipt_token_sol_value_per_token(
                receipt_token_mint.decimals,
                receipt_token_mint.supply,
            )?,
            receipt_token_mint.supply,
        ),
    });

    Ok(())
}

/// Receipt token price & supply might be outdated.
fn emit_fund_manager_updated_fund_event(
    fund_account: &FundAccount,
    receipt_token_mint: &Mint,
    receipt_token_mint_address: Pubkey,
) -> Result<()> {
    let receipt_token_price = fund_account.receipt_token_sol_value_per_token(
        receipt_token_mint.decimals,
        receipt_token_mint.supply,
    )?;

    emit!(events::FundManagerUpdatedFund {
        receipt_token_mint: receipt_token_mint_address,
        fund_account: FundAccountInfo::new(
            fund_account,
            receipt_token_price,
            receipt_token_mint.supply,
        ),
    });

    Ok(())
}
