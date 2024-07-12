use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

// use crate::state::*;
use crate::fund::*;

#[derive(Accounts)]
#[instruction(receipt_token_name: String)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        // seeds = [b"fund", receipt_token_mint.key().as_ref()], // fund + <any receipt token mint account>
        seeds = [b"fund"],
        bump,
        space = 8 + 4 + 4 + 914,
    )]
    pub fund: Account<'info, Fund>,

    #[account(
        init,
        payer = admin,
        seeds = [receipt_token_name.as_bytes().as_ref()],
        bump,
        mint::token_program = token_program,
        mint::decimals = 9,
        mint::authority = fund,
        mint::freeze_authority = fund,
        extensions::transfer_hook::authority = fund,
        extensions::transfer_hook::program_id = crate::ID,
    )]
    pub receipt_token_mint: InterfaceAccount<'info, Mint>, // fragSOL token mint account
    #[account(
        init,
        payer = admin,
        seeds = [b"receipt_lock", receipt_token_mint.key().as_ref()],
        bump,
        token::mint = receipt_token_mint,
        token::authority = fund,
    )]
    pub receipt_token_lock_account: InterfaceAccount<'info, TokenAccount>, // fund's fragSOL lock account

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Initialize>,
    receipt_token_name: String,
    default_protocol_fee_rate: u16,
    whitelisted_tokens: Vec<Pubkey>,
    lst_caps: Vec<u64>,
) -> Result<()> {
    let fund = &mut ctx.accounts.fund;

    Ok((fund.initialize(
        ctx.accounts.admin.key(),
        default_protocol_fee_rate,
        whitelisted_tokens,
        lst_caps,
        ctx.accounts.receipt_token_mint.key(),
        ctx.accounts.receipt_token_lock_account.key(),
    ))?)
}
