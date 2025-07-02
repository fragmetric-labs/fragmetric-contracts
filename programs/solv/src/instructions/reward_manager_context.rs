use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::errors::VaultError;
use crate::states::VaultAccount;

#[event_cpi]
#[derive(Accounts)]
pub struct RewardManagerContext<'info> {
    pub reward_manager: Signer<'info>,
    /// CHECK: ..
    pub delegate: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [VaultAccount::SEED, vault_receipt_token_mint.key().as_ref()],
        bump = vault_account.load()?.get_bump(),
        has_one = reward_manager @ VaultError::VaultAdminMismatchError,
        constraint = vault_account.load()?.is_latest_version() @ VaultError::InvalidAccountDataVersionError,
    )]
    pub vault_account: AccountLoader<'info, VaultAccount>,

    pub vault_receipt_token_mint: Account<'info, Mint>,
    pub reward_token_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = reward_token_mint,
        associated_token::authority = vault_account,
    )]
    pub reward_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn process_delegate_reward_token_account(ctx: &mut Context<RewardManagerContext>) -> Result<()> {
    let RewardManagerContext {
        delegate,
        vault_account,
        reward_token_mint,
        reward_token_account,
        token_program,
        ..
    } = ctx.accounts;

    vault_account
        .load_mut()?
        .add_delegated_reward_token_mint(reward_token_mint.key())?;

    anchor_spl::token::approve(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            anchor_spl::token::Approve {
                to: reward_token_account.to_account_info(),
                delegate: delegate.to_account_info(),
                authority: vault_account.to_account_info(),
            },
            &[&vault_account.load()?.get_seeds()],
        ),
        u64::MAX,
    )?;

    Ok(())
}
