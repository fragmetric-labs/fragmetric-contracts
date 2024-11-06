use anchor_lang::prelude::*;
use anchor_spl::token_2022::spl_token_2022;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};

use crate::modules::normalization::*;
use crate::utils::PDASeeds;

// migration v0.3.1
pub fn process_update_supported_token_lock_account<'info>(
    payer: &Signer<'info>,
    supported_token_mint: &InterfaceAccount<'info, Mint>,
    old_supported_token_lock_account: &InterfaceAccount<'info, TokenAccount>,
    new_supported_token_lock_account: &InterfaceAccount<'info, TokenAccount>,
    normalized_token_pool_account: &mut Account<'info, NormalizedTokenPoolAccount>,
    supported_token_program: &Interface<'info, TokenInterface>,
) -> Result<()> {
    normalized_token_pool_account
        .get_supported_token_mut(supported_token_mint.key())?
        .set_lock_account(new_supported_token_lock_account.key());

    let amount = old_supported_token_lock_account.amount;
    let decimals = supported_token_mint.decimals;
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            supported_token_program.to_account_info(),
            token_interface::TransferChecked {
                from: old_supported_token_lock_account.to_account_info(),
                mint: supported_token_mint.to_account_info(),
                to: new_supported_token_lock_account.to_account_info(),
                authority: normalized_token_pool_account.to_account_info()
            },
            &[
                normalized_token_pool_account.get_signer_seeds().as_ref()
            ]),
        amount,
        decimals
    )?;

    token_interface::close_account(
        CpiContext::new_with_signer(
            supported_token_program.to_account_info(),
            token_interface::CloseAccount {
                account: old_supported_token_lock_account.to_account_info(),
                destination: payer.to_account_info(),
                authority: normalized_token_pool_account.to_account_info()
            },
            &[
                normalized_token_pool_account.get_signer_seeds().as_ref()
            ]),
    )
}

pub fn process_add_supported_token<'info>(
    supported_token_mint: &InterfaceAccount<Mint>,
    supported_token_lock_account: &InterfaceAccount<'info, TokenAccount>,
    normalized_token_pool_account: &mut Account<NormalizedTokenPoolAccount>,
    supported_token_program: &Interface<'info, TokenInterface>,
) -> Result<()> {
    normalized_token_pool_account.add_new_supported_token(
        supported_token_mint.key(),
        supported_token_program.key(),
        supported_token_lock_account.key(),
    )
}
