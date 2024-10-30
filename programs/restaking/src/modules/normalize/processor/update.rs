use anchor_lang::prelude::*;
use anchor_spl::token_2022::spl_token_2022;
use anchor_spl::token_interface::{self, TokenAccount, TokenInterface};

use crate::modules::normalize::*;

pub fn process_update_supported_token_lock_account_authority<'info>(
    fund_manager: &Signer<'info>,
    supported_token_lock_account: &InterfaceAccount<'info, TokenAccount>,
    normalized_token_pool_account: &Account<NormalizedTokenPoolAccount>,
    supported_token_program: &Interface<'info, TokenInterface>,
) -> Result<()> {
    token_interface::set_authority(
        CpiContext::new(
            supported_token_program.to_account_info(),
            token_interface::SetAuthority {
                current_authority: fund_manager.to_account_info(),
                account_or_mint: supported_token_lock_account.to_account_info(),
            },
        ),
        spl_token_2022::instruction::AuthorityType::AccountOwner,
        Some(normalized_token_pool_account.key()),
    )
}
