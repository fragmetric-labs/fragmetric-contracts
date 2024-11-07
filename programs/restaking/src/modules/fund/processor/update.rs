use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{self, spl_token_2022, Mint, TokenAccount, TokenInterface};
use spl_tlv_account_resolution::state::ExtraAccountMetaList;
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

use crate::errors::ErrorCode;
use crate::events;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::fund::*;
use crate::utils::PDASeeds;

pub fn process_update_fund_account_if_needed(
    receipt_token_mint: &InterfaceAccount<Mint>,
    fund_account: &mut Account<FundAccount>,
) -> Result<()> {
    fund_account.update_if_needed(receipt_token_mint.key());
    Ok(())
}

// migration v0.3.1
pub fn process_update_fund_reserve_account_if_needed<'info>(
    fund_reserve_account: &SystemAccount<'info>,
    receipt_token_mint: &InterfaceAccount<Mint>,
    fund_account: &Account<'info, FundAccount>,
    fund_execution_reserved_account: &SystemAccount<'info>,
    system_program: &Program<'info, System>,
    fund_execution_reserved_account_bump: u8,
) -> Result<()> {
    let rent = Rent::get()?;
    let execution_reserved_account_balance = fund_execution_reserved_account.get_lamports();
    let reserve_account_balance = fund_reserve_account.get_lamports();
    let expected_sol_reserved_amount_in_fund_account = fund_account.sol_operation_reserved_amount
        .checked_add(fund_account.withdrawal.get_sol_withdrawal_reserved_amount())
        .ok_or_else(|| error!(ErrorCode::FundUnexpectedReserveAccountBalanceException))?
        .checked_sub(execution_reserved_account_balance)
        .ok_or_else(|| error!(ErrorCode::FundUnexpectedReserveAccountBalanceException))?
        .checked_sub(reserve_account_balance)
        .ok_or_else(|| error!(ErrorCode::FundUnexpectedReserveAccountBalanceException))?;

    if expected_sol_reserved_amount_in_fund_account > 0 {
        fund_account.sub_lamports(expected_sol_reserved_amount_in_fund_account)?;
        fund_reserve_account.add_lamports(expected_sol_reserved_amount_in_fund_account)?;

        if !rent.is_exempt(fund_account.get_lamports(), AsRef::<AccountInfo>::as_ref(fund_account).data_len()) {
            err!(ErrorCode::FundUnexpectedReserveAccountBalanceException)?;
        }

        return Ok(()); // need to re-run
    }

    if execution_reserved_account_balance > 0 {
        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: fund_execution_reserved_account.to_account_info(),
                    to: fund_reserve_account.to_account_info(),
                },
                &[&[
                    FundAccount::EXECUTION_RESERVED_SEED,
                    receipt_token_mint.key().as_ref(),
                    &[fund_execution_reserved_account_bump],
                ]],
            ),
            execution_reserved_account_balance,
        )?;
    }

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

// migration v0.3.1
pub fn process_update_receipt_token_lock_account<'info>(
    payer: &Signer<'info>,
    old_receipt_token_lock_account: &InterfaceAccount<'info, TokenAccount>,
    receipt_token_lock_authority: &Account<'info, ReceiptTokenLockAuthority>,
    receipt_token_program: &Program<'info, Token2022>,
) -> Result<()> {
    token_interface::close_account(
        CpiContext::new_with_signer(
            receipt_token_program.to_account_info(),
            token_interface::CloseAccount {
                account: old_receipt_token_lock_account.to_account_info(),
                destination: payer.to_account_info(),
                authority: receipt_token_lock_authority.to_account_info()
            },
        &[
                receipt_token_lock_authority.get_signer_seeds().as_ref(),
            ]),
    )
}

// migration v0.3.1
pub fn process_update_receipt_token_mint_authority<'info>(
    receipt_token_mint: &InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &Account<'info, ReceiptTokenMintAuthority>,
    fund_account: &Account<FundAccount>,
    receipt_token_program: &Program<'info, Token2022>,
) -> Result<()> {
    token_interface::set_authority(
        CpiContext::new_with_signer(
            receipt_token_program.to_account_info(),
            token_interface::SetAuthority {
                current_authority: receipt_token_mint_authority.to_account_info(),
                account_or_mint: receipt_token_mint.to_account_info(),
            },
            &[receipt_token_mint_authority.get_signer_seeds().as_ref()],
        ),
        spl_token_2022::instruction::AuthorityType::MintTokens,
        Some(fund_account.key()),
    )
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

// migration v0.3.1
pub fn process_update_supported_token_account<'info>(
    payer: &Signer<'info>,
    supported_token_mint: &InterfaceAccount<'info, Mint>,
    old_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
    supported_token_authority: &Account<'info, SupportedTokenAuthority>,
    new_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
    supported_token_program: &Interface<'info, TokenInterface>,
) -> Result<()> {
    let amount = old_supported_token_account.amount;
    let decimals = supported_token_mint.decimals;
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            supported_token_program.to_account_info(),
            token_interface::TransferChecked {
                from: old_supported_token_account.to_account_info(),
                mint: supported_token_mint.to_account_info(),
                to: new_supported_token_account.to_account_info(),
                authority: supported_token_authority.to_account_info()
            },
            &[
                supported_token_authority.get_signer_seeds().as_ref()
            ]),
        amount,
        decimals
    )?;

    token_interface::close_account(
        CpiContext::new_with_signer(
            supported_token_program.to_account_info(),
            token_interface::CloseAccount {
                account: old_supported_token_account.to_account_info(),
                destination: payer.to_account_info(),
                authority: supported_token_authority.to_account_info()
            },
        &[
                supported_token_authority.get_signer_seeds().as_ref(),
            ]),
    )
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
