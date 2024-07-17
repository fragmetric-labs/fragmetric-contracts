use anchor_lang::prelude::*;
use std::mem::size_of;

#[cfg(not(feature = "no-entrypoint"))]
// use {default_env::default_env, solana_security_txt::security_txt};
use solana_security_txt::security_txt;

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    // Required fields
    name: "Fragmetric",
    project_url: "http://example.com",
    contacts: "email:example@example.com,link:https://example.com/security,discord:example#1234",
    policy: "https://github.com/solana-labs/solana/blob/master/SECURITY.md",

    // Optional Fields
    preferred_languages: "en",
    source_code: "https://github.com/example/example",
    // source_revision: default_env!("GITHUB_SHA", ""),
    // source_release: default_env!("GITHUB_REF_NAME", ""),
    encryption: "
-----BEGIN PGP PUBLIC KEY BLOCK-----
Comment: Alice's OpenPGP certificate
Comment: https://www.ietf.org/id/draft-bre-openpgp-samples-01.html

mDMEXEcE6RYJKwYBBAHaRw8BAQdArjWwk3FAqyiFbFBKT4TzXcVBqPTB3gmzlC/U
b7O1u120JkFsaWNlIExvdmVsYWNlIDxhbGljZUBvcGVucGdwLmV4YW1wbGU+iJAE
ExYIADgCGwMFCwkIBwIGFQoJCAsCBBYCAwECHgECF4AWIQTrhbtfozp14V6UTmPy
MVUMT0fjjgUCXaWfOgAKCRDyMVUMT0fjjukrAPoDnHBSogOmsHOsd9qGsiZpgRnO
dypvbm+QtXZqth9rvwD9HcDC0tC+PHAsO7OTh1S1TC9RiJsvawAfCPaQZoed8gK4
OARcRwTpEgorBgEEAZdVAQUBAQdAQv8GIa2rSTzgqbXCpDDYMiKRVitCsy203x3s
E9+eviIDAQgHiHgEGBYIACAWIQTrhbtfozp14V6UTmPyMVUMT0fjjgUCXEcE6QIb
DAAKCRDyMVUMT0fjjlnQAQDFHUs6TIcxrNTtEZFjUFm1M0PJ1Dng/cDW4xN80fsn
0QEA22Kr7VkCjeAEC08VSTeV+QFsmz55/lntWkwYWhmvOgE=
=iIGO
-----END PGP PUBLIC KEY BLOCK-----
",
    auditors: "None"
//     acknowledgements: "
// The following hackers could've stolen all our money but didn't:
// - EncryptX
// "
}

declare_id!("A58NQYmJCyDPsc1EfaQZ99piFopPtCYArP242rLTbYbV");

#[program]
pub mod dummy {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let user_token_amount = &mut ctx.accounts.user_token_amount;
        user_token_amount.user = ctx.accounts.user.key();
        user_token_amount.amount = 0;
        user_token_amount.bump = user_token_amount.bump.clone();

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

    pub fn versioned_method(ctx: Context<Update>, data: VersionedState) -> Result<()> {
        match data {
            VersionedState::V1(data) => {
                // DO SOMETHING...
                emit!(VersionedEventV1 {
                    field1: data.field1,
                    field2: data.field2,
                })
            },
            VersionedState::V2(data) => {
                // DO SOMETHING...
                emit!(VersionedEventV2 {
                    field1: data.field1,
                    field2: data.field2,
                    field3: data.field3,
                    field4: data.field4,
                })
            },
        }
        return err!(Errors::NotImplemented)
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