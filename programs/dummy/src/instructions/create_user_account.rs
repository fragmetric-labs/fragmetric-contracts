use anchor_lang::prelude::*;
use fragmetric_util::Upgradable;

use crate::versioning::*;

#[derive(Accounts)]
pub struct CreateUserAccount<'info> {
    #[account(
        init,
        seeds = [user.key().as_ref()],
        bump,
        payer = user,
        space = 8 + AccountData::INIT_SPACE,
    )]
    pub user_account: Account<'info, AccountData>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn create_user_account(
    ctx: Context<CreateUserAccount>,
    request: InstructionRequest,
) -> Result<()> {
    // Set metadata for new account
    ctx.accounts.user_account.owner = ctx.accounts.user.key();
    ctx.accounts.user_account.created_at = Clock::get()?.unix_timestamp;

    msg!(&format!(
        "Created user account (User = {}, ID = {}))",
        ctx.accounts.user.key(),
        ctx.accounts.user_account.key()
    ));

    // Call business logic
    ctx.accounts
        .user_account
        .to_latest_version()
        .update(request.into())
}
