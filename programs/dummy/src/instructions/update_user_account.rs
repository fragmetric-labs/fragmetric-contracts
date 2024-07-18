use anchor_lang::prelude::*;
use fragmetric_util::Upgradable;

use crate::versioning::*;

#[derive(Accounts)]
pub struct UpdateUserAccount<'info> {
    #[account(
        mut,
        seeds = [user.key().as_ref()],
        bump,
        realloc = 8 + AccountData::INIT_SPACE,
        realloc::payer = user,
        realloc::zero = false,
    )]
    pub user_account: Account<'info, AccountData>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn update_user_account(
    ctx: Context<UpdateUserAccount>,
    request: InstructionRequest,
) -> Result<()> {
    // Checks the account owner
    require_keys_eq!(ctx.accounts.user.key(), ctx.accounts.user_account.owner);

    msg!(&format!(
        "Updated user account (User = {}, ID = {}))",
        ctx.accounts.user.key(),
        ctx.accounts.user_account.key()
    ));

    // Call business logic
    ctx.accounts
        .user_account
        .to_latest_version()
        .update(request.into())
}
