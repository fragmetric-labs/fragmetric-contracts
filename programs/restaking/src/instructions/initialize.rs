use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

// use crate::state::*;
use crate::fund::*;

#[derive(Accounts)]
#[instruction(receipt_token_name: String)]
pub struct InitializeFund<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        seeds = [b"fund", receipt_token_mint.key().as_ref()], // fund + <any receipt token mint account>
        bump,
        space = 8 + 4 + 4 + 4 + 1070,
    )]
    pub fund: Account<'info, Fund>,

    #[account(
        init,
        payer = admin,
        seeds = [b"receipt_token_authority", receipt_token_mint.key().as_ref()],
        bump,
        space = 8 + 32,
    )]
    pub receipt_token_authority: Account<'info, ReceiptTokenAuthority>,

    #[account(
        init,
        payer = admin,
        seeds = [receipt_token_name.as_bytes().as_ref()],
        bump,
        mint::token_program = token_program,
        mint::decimals = 9,
        mint::authority = receipt_token_authority,
        mint::freeze_authority = receipt_token_authority,
        extensions::transfer_hook::authority = receipt_token_authority,
        extensions::transfer_hook::program_id = crate::ID,
    )]
    pub receipt_token_mint: InterfaceAccount<'info, Mint>, // fragSOL token mint account
    // #[account(
    //     init,
    //     payer = admin,
    //     seeds = [b"receipt_lock", receipt_token_mint.key().as_ref()],
    //     bump,
    //     token::mint = receipt_token_mint,
    //     token::authority = fund,
    // )]
    // pub receipt_token_lock_account: InterfaceAccount<'info, TokenAccount>, // fund's fragSOL lock account
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeFund>,
    receipt_token_name: String,
    default_protocol_fee_rate: u16,
    whitelisted_tokens: Vec<Pubkey>,
    lst_caps: Vec<u64>,
    lsts_amount_in: Vec<u128>,
) -> Result<()> {
    let fund = &mut ctx.accounts.fund;
    let receipt_token_mint_addr = ctx.accounts.receipt_token_mint.to_account_info().key;
    msg!("receipit_token_mint: {}", receipt_token_mint_addr);

    Ok((fund.initialize(
        ctx.accounts.admin.key(),
        default_protocol_fee_rate,
        whitelisted_tokens,
        lst_caps,
        ctx.accounts.receipt_token_mint.key(),
        // ctx.accounts.receipt_token_lock_account.key(),
        lsts_amount_in,
    ))?)
}
