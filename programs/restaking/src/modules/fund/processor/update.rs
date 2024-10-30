use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};
use spl_tlv_account_resolution::state::ExtraAccountMetaList;
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

use crate::errors::ErrorCode;
use crate::events;
use crate::modules::normalize::NormalizedTokenPoolAccount;
use crate::modules::pricing::{self, TokenPricingSource, TokenPricingSourceMap};
use crate::modules::{fund::*, normalize};

pub fn process_update_fund_account_if_needed(
    receipt_token_mint: &InterfaceAccount<Mint>,
    fund_account: &mut Account<FundAccount>,
) -> Result<()> {
    fund_account.update_if_needed(receipt_token_mint.key());
    Ok(())
}

pub fn process_update_user_fund_account_if_needed(
    user: &Signer,
    receipt_token_mint: &InterfaceAccount<Mint>,
    user_fund_account: &mut UserFundAccount,
) -> Result<()> {
    user_fund_account.update_if_needed(receipt_token_mint.key(), user.key());
    Ok(())
}

pub fn process_update_extra_account_meta_list_if_needed(
    extra_account_meta_list: &AccountInfo,
) -> Result<()> {
    ExtraAccountMetaList::update::<ExecuteInstruction>(
        &mut extra_account_meta_list.try_borrow_mut_data()?,
        &extra_account_metas()?,
    )?;
    Ok(())
}

pub fn process_update_sol_capacity_amount(
    receipt_token_mint: &InterfaceAccount<Mint>,
    fund_account: &mut Account<FundAccount>,
    capacity_amount: u64,
) -> Result<()> {
    fund_account.set_sol_capacity_amount(capacity_amount)?;
    emit_fund_manager_updated_fund_event(receipt_token_mint, fund_account)
}

pub fn process_update_supported_token_capacity_amount(
    receipt_token_mint: &InterfaceAccount<Mint>,
    fund_account: &mut Account<FundAccount>,
    token: Pubkey,
    capacity_amount: u64,
) -> Result<()> {
    fund_account
        .get_supported_token_mut(token)?
        .set_capacity_amount(capacity_amount)?;
    emit_fund_manager_updated_fund_event(receipt_token_mint, fund_account)
}

pub fn process_update_withdrawal_enabled_flag(
    receipt_token_mint: &InterfaceAccount<Mint>,
    fund_account: &mut Account<FundAccount>,
    enabled: bool,
) -> Result<()> {
    fund_account.withdrawal.set_withdrawal_enabled_flag(enabled);
    emit_fund_manager_updated_fund_event(receipt_token_mint, fund_account)
}

pub fn process_update_sol_withdrawal_fee_rate(
    receipt_token_mint: &InterfaceAccount<Mint>,
    fund_account: &mut Account<FundAccount>,
    sol_withdrawal_fee_rate: u16,
) -> Result<()> {
    fund_account
        .withdrawal
        .set_sol_withdrawal_fee_rate(sol_withdrawal_fee_rate)?;
    emit_fund_manager_updated_fund_event(receipt_token_mint, fund_account)
}

pub fn process_update_batch_processing_threshold(
    receipt_token_mint: &InterfaceAccount<Mint>,
    fund_account: &mut Account<FundAccount>,
    amount: Option<u64>,
    duration: Option<i64>,
) -> Result<()> {
    fund_account
        .withdrawal
        .set_batch_processing_threshold(amount, duration);
    emit_fund_manager_updated_fund_event(receipt_token_mint, fund_account)
}

pub fn process_add_supported_token<'info>(
    receipt_token_mint: &InterfaceAccount<Mint>,
    supported_token_mint: &InterfaceAccount<Mint>,
    fund_account: &mut Account<FundAccount>,
    supported_token_program: &Interface<TokenInterface>,
    capacity_amount: u64,
    pricing_source: TokenPricingSource,
    pricing_sources: &'info [AccountInfo<'info>],
) -> Result<()> {
    fund_account.add_supported_token(
        supported_token_mint.key(),
        supported_token_program.key(),
        supported_token_mint.decimals,
        capacity_amount,
        pricing_source,
    )?;
    update_asset_prices(fund_account, pricing_sources)?;
    emit_fund_manager_updated_fund_event(receipt_token_mint, fund_account)
}

pub fn process_update_prices<'info>(
    receipt_token_mint: &InterfaceAccount<Mint>,
    fund_account: &mut Account<FundAccount>,
    pricing_sources: &'info [AccountInfo<'info>],
) -> Result<()> {
    update_asset_prices(fund_account, pricing_sources)?;

    emit!(events::OperatorUpdatedFundPrice {
        receipt_token_mint: receipt_token_mint.key(),
        fund_account: FundAccountInfo::from(fund_account, receipt_token_mint),
    });

    Ok(())
}

pub(in crate::modules) fn update_asset_prices<'info>(
    fund_account: &mut Account<FundAccount>,
    pricing_sources: &'info [AccountInfo<'info>],
) -> Result<()> {
    let pricing_source_map = create_pricing_source_map(fund_account, pricing_sources)?;
    fund_account
        .get_supported_tokens_iter_mut()
        .try_for_each(|token| token.update_one_token_as_sol(&pricing_source_map))
}

fn emit_fund_manager_updated_fund_event(
    receipt_token_mint: &InterfaceAccount<Mint>,
    fund_account: &Account<FundAccount>,
) -> Result<()> {
    emit!(events::FundManagerUpdatedFund {
        receipt_token_mint: receipt_token_mint.key(),
        fund_account: FundAccountInfo::from(fund_account, receipt_token_mint),
    });

    Ok(())
}
