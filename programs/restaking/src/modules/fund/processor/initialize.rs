use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, spl_token_2022, Token2022};
use anchor_spl::token_interface::Mint;
use spl_tlv_account_resolution::state::ExtraAccountMetaList;
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

use crate::modules::fund::*;

pub fn process_initialize_fund_account<'info>(
    admin: &Signer<'info>,
    receipt_token_mint: &InterfaceAccount<'info, Mint>,
    fund_account: &mut Account<FundAccount>,
    receipt_token_program: &Program<'info, Token2022>,
    bump: u8,
) -> Result<()> {
    fund_account.initialize(bump, receipt_token_mint.key());

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
        Some(fund_account.key()),
    )
}

pub fn process_initialize_extra_account_meta_list(
    extra_account_meta_list: &AccountInfo,
) -> Result<()> {
    ExtraAccountMetaList::init::<ExecuteInstruction>(
        &mut extra_account_meta_list.try_borrow_mut_data()?,
        &extra_account_metas()?,
    )?;
    Ok(())
}

pub fn process_initialize_user_fund_account(
    user: &Signer,
    receipt_token_mint: &InterfaceAccount<Mint>,
    user_fund_account: &mut Account<UserFundAccount>,
    bump: u8,
) -> Result<()> {
    user_fund_account.initialize(bump, receipt_token_mint.key(), user.key());
    Ok(())
}
