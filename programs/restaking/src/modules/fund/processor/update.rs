use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};
use spl_tlv_account_resolution::state::ExtraAccountMetaList;
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

use crate::errors::ErrorCode;
use crate::events;
use crate::modules::fund::*;
use crate::modules::pricing::{self, TokenAmount, TokenPricingSource};

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
        fund_account: FundAccountInfo::from(
            fund_account,
            get_one_receipt_token_as_sol(receipt_token_mint, fund_account)?,
            receipt_token_mint.supply,
        ),
    });

    Ok(())
}

pub(in crate::modules) fn update_asset_prices<'info>(
    fund_account: &mut Account<FundAccount>,
    pricing_sources: &'info [AccountInfo<'info>],
) -> Result<()> {
    let mut one_token_as_sol_list = Vec::new();
    let pricing_source_accounts = pricing::create_pricing_sources_map(pricing_sources);
    for token in fund_account.get_supported_tokens_iter() {
        let mut token_amount_as_sol = 0u64;
        let mut stack = Vec::from([(
            token.get_pricing_source(),
            token.get_denominated_amount_per_token()?,
        )]);

        while let Some((source, token_amount)) = stack.pop() {
            match pricing::calculate_token_amount_as_sol(
                &pricing_source_accounts,
                source,
                token_amount,
            )? {
                TokenAmount::SOLAmount(sol_amount) => {
                    token_amount_as_sol = token_amount_as_sol
                        .checked_add(sol_amount)
                        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
                }
                TokenAmount::TokenAmounts(token_amounts) => {
                    for (mint, token_amount) in token_amounts {
                        let source = fund_account.get_supported_token(mint)?.get_pricing_source();
                        stack.push((source, token_amount));
                    }
                }
            }
        }

        one_token_as_sol_list.push(token_amount_as_sol);
    }

    for (token, one_token_as_sol) in fund_account
        .get_supported_tokens_iter_mut()
        .zip(one_token_as_sol_list)
    {
        token.one_token_as_sol = one_token_as_sol;
    }

    Ok(())
}

fn emit_fund_manager_updated_fund_event(
    receipt_token_mint: &InterfaceAccount<Mint>,
    fund_account: &Account<FundAccount>,
) -> Result<()> {
    emit!(events::FundManagerUpdatedFund {
        receipt_token_mint: receipt_token_mint.key(),
        fund_account: FundAccountInfo::from(
            fund_account,
            get_one_receipt_token_as_sol(receipt_token_mint, fund_account)?,
            receipt_token_mint.supply,
        ),
    });

    Ok(())
}
