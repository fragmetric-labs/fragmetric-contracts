use anchor_lang::prelude::*;
use std::mem::size_of;

#[cfg(not(feature = "no-entrypoint"))]
pub use self::security::*;
use instructions::*;
use versioning::*;

mod instructions;
#[cfg(not(feature = "no-entrypoint"))]
mod security;
mod versioning;

declare_id!("A58NQYmJCyDPsc1EfaQZ99piFopPtCYArP242rLTbYbV");

#[program]
pub mod dummy {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let user_token_amount = &mut ctx.accounts.user_token_amount;
        user_token_amount.user = ctx.accounts.user.key();
        user_token_amount.amount = 0;
        user_token_amount.bump = ctx.bumps.user_token_amount;

        // msg!("User Account Created");
        // msg!("User Amount: {}", user_token_amount.amount);
        Ok(())
    }

    pub fn increment(ctx: Context<Update>, data: UserTokenAmount) -> Result<()> {
        let user_token_amount = &mut ctx.accounts.user_token_amount;
        // msg!("Previous token {} amount: {}", user_token_amount.token, user_token_amount.amount);

        user_token_amount.token = data.token;
        user_token_amount.amount = user_token_amount.amount.checked_add(data.amount).unwrap();
        // msg!("User's token amount is incremented. Current token {} amount: {}", user_token_amount.token, user_token_amount.amount);

        emit!(Incremented {
            user: user_token_amount.key(),
            token: user_token_amount.token.clone(),
            amount: user_token_amount.amount
        });
        Ok(())
    }

    pub fn decrement(ctx: Context<Update>, data: UserTokenAmount) -> Result<()> {
        let user_token_amount = &mut ctx.accounts.user_token_amount;
        // msg!("Previous token {} amount: {}", user_token_amount.token, user_token_amount.amount);

        user_token_amount.token = data.token;
        user_token_amount.amount = user_token_amount.amount.checked_sub(data.amount).unwrap();
        // msg!("User's token amount is decremented. Current token {} amount: {}", user_token_amount.token, user_token_amount.amount);

        emit!(Decremented {
            user: user_token_amount.key(),
            token: user_token_amount.token.clone(),
            amount: user_token_amount.amount
        });
        Ok(())
    }

    pub fn versioned_method(_ctx: Context<Update>, data: VersionedState) -> Result<()> {
        match data {
            VersionedState::V1(data) => {
                // DO SOMETHING...
                emit!(VersionedEventV1 {
                    field1: data.field1,
                    field2: data.field2,
                })
            }
            VersionedState::V2(data) => {
                // DO SOMETHING...
                emit!(VersionedEventV2 {
                    field1: data.field1,
                    field2: data.field2,
                    field3: data.field3,
                    field4: data.field4,
                })
            }
        }
        err!(Errors::NotImplemented)
    }

    //////////////////////////////////////////////////////////////////////////
    /// Versioned Instructions
    //////////////////////////////////////////////////////////////////////////

    pub fn create_user_account(
        ctx: Context<CreateUserAccount>,
        request: InstructionRequest,
    ) -> Result<()> {
        instructions::create_user_account(ctx, request)
    }

    pub fn update_user_account(
        ctx: Context<UpdateUserAccount>,
        request: InstructionRequest,
    ) -> Result<()> {
        instructions::update_user_account(ctx, request)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init_if_needed,
        payer = user,
        seeds = [b"user_token_amount", user.key().as_ref()],
        bump,
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
    pub user: Pubkey,
    pub bump: u8,
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

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum VersionedState {
    V1(VersionedStateV1),
    V2(VersionedStateV2),
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct VersionedStateV1 {
    field1: u64,
    field2: String,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct VersionedStateV2 {
    field1: u64,
    field2: u32,
    field3: String,
    field4: bool,
}

#[event]
struct VersionedEventV1 {
    field1: u64,
    field2: String,
}

#[event]
struct VersionedEventV2 {
    field1: u64,
    field2: u32,
    field3: String,
    field4: bool,
}

#[error_code]
pub enum Errors {
    #[msg("invalid data format")]
    InvalidDataFormat,
    #[msg("not implemented")]
    NotImplemented,
}
