use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct LogMessage {}

impl LogMessage {
    pub fn log_message(_ctx: Context<Self>, message: String) -> Result<()> {
        msg!("{}", message);
        Ok(())
    }
}
