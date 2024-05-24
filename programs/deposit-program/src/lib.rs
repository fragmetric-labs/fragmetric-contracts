use anchor_lang::prelude::*;
use std::mem::size_of;

declare_id!("5yYKAKV5r62ooXrKZNpxr9Bkk7CTtpyJ8sXD7k2WryUc");

#[program]
pub mod deposit_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let user_token_amount = &mut ctx.accounts.user_token_amount;
        user_token_amount.amount = 0;

        msg!("User Account Created");
        msg!("User Amount: {}", user_token_amount.amount);
        Ok(())
    }

    pub fn increment(ctx: Context<Update>, data: UserTokenAmount) -> Result<()> {
        let user_token_amount = &mut ctx.accounts.user_token_amount;
        msg!("Previous token {} amount: {}", user_token_amount.token, user_token_amount.amount);

        user_token_amount.token = data.token;
        user_token_amount.amount = user_token_amount.amount.checked_add(data.amount).unwrap();
        msg!("User's token amount is incremented. Current token {} amount: {}", user_token_amount.token, user_token_amount.amount);

        emit!(Incremented {
            user: user_token_amount.key(),
            token: user_token_amount.token.clone(),
            amount: user_token_amount.amount
        });
        Ok(())
    }

    pub fn decrement(ctx: Context<Update>, data: UserTokenAmount) -> Result<()> {
        let user_token_amount = &mut ctx.accounts.user_token_amount;
        msg!("Previous token {} amount: {}", user_token_amount.token, user_token_amount.amount);

        user_token_amount.token = data.token;
        user_token_amount.amount = user_token_amount.amount.checked_sub(data.amount).unwrap();
        msg!("User's token amount is decremented. Current token {} amount: {}", user_token_amount.token, user_token_amount.amount);

        emit!(Decremented {
            user: user_token_amount.key(),
            token: user_token_amount.token.clone(),
            amount: user_token_amount.amount
        });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + size_of::<UserTokenAmount>(),
    )]
    pub user_token_amount: Account<'info, UserTokenAmount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub user_token_amount: Account<'info, UserTokenAmount>,
    pub user: Signer<'info>,
}

#[account]
pub struct UserTokenAmount {
    pub token: String,
    pub amount: u64,
}

#[event]
pub struct Incremented {
    pub user: Pubkey,
    pub token: String,
    pub amount: u64,
}

#[event]
pub struct Decremented {
    pub user: Pubkey,
    pub token: String,
    pub amount: u64,
}
