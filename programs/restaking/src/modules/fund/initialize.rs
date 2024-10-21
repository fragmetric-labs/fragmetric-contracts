use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, spl_token_2022, Token2022};
use anchor_spl::token_interface::Mint;
use spl_tlv_account_resolution::state::ExtraAccountMetaList;
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

use super::*;

pub fn initialize_receipt_token_lock_authority(
    receipt_token_lock_authority: &mut ReceiptTokenLockAuthority,
    receipt_token_mint: Pubkey,
    bump: u8,
) -> Result<()> {
    receipt_token_lock_authority.initialize(bump, receipt_token_mint);
    Ok(())
}

pub fn initialize_fund_account(
    fund_account: &mut FundAccount,
    receipt_token_mint: Pubkey,
    bump: u8,
) -> Result<()> {
    fund_account.initialize(bump, receipt_token_mint);
    Ok(())
}

pub fn initialize_receipt_token_mint_authority<'info>(
    admin: &Signer<'info>,
    receipt_token_mint: &InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &mut Account<ReceiptTokenMintAuthority>,
    receipt_token_program: &Program<'info, Token2022>,
    bump: u8,
) -> Result<()> {
    receipt_token_mint_authority.initialize(bump, receipt_token_mint.key());

    // set token mint authority
    token_2022::set_authority(
        CpiContext::new(
            receipt_token_program.to_account_info(),
            token_2022::SetAuthority {
                current_authority: admin.to_account_info(),
                account_or_mint: receipt_token_mint.to_account_info(),
            },
        ),
        spl_token_2022::instruction::AuthorityType::MintTokens,
        Some(receipt_token_mint_authority.key()),
    )
}

pub fn initialize_extra_account_meta_list(extra_account_meta_list: &AccountInfo) -> Result<()> {
    ExtraAccountMetaList::init::<ExecuteInstruction>(
        &mut extra_account_meta_list.try_borrow_mut_data()?,
        &extra_account_metas()?,
    )?;
    Ok(())
}

pub fn initialize_supported_token_authority(
    supported_token_authority: &mut SupportedTokenAuthority,
    receipt_token_mint: Pubkey,
    supported_token_mint: Pubkey,
    bump: u8,
) -> Result<()> {
    supported_token_authority.initialize(bump, receipt_token_mint, supported_token_mint);
    Ok(())
}

pub fn initialize_user_fund_account(
    user_fund_account: &mut UserFundAccount,
    receipt_token_mint: Pubkey,
    user: Pubkey,
    bump: u8,
) -> Result<()> {
    user_fund_account.initialize(bump, receipt_token_mint, user);
    Ok(())
}
