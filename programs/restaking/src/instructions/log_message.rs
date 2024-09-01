use anchor_lang::prelude::*;

use crate::constants::ADMIN_PUBKEY;

#[derive(Accounts)]
pub struct LogMessage<'info> {
    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,
}

impl<'info> LogMessage<'info> {
    pub fn log_message(_ctx: Context<Self>, message: String) -> Result<()> {
        msg!("{}", message);
        Ok(())
    }
}
