use anchor_lang::prelude::*;


#[derive(Accounts)]
pub struct OperatorEmptyContext {
}

impl OperatorEmptyContext {
    pub fn log_message(_ctx: Context<Self>, message: String) -> Result<()> {
        msg!("{}", message);
        Ok(())
    }
}
