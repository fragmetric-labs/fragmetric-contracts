use anchor_lang::prelude::*;

use crate::constants::ADMIN_PUBKEY;

#[derive(Accounts)]
pub struct AdminEmptyContext<'info> {
    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,
}

impl<'info> AdminEmptyContext<'info> {
    pub fn log_message(_ctx: Context<Self>, message: String) -> Result<()> {
        msg!("{}", message);
        Ok(())
    }
}
