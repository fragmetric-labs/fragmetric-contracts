use anchor_lang::prelude::*;
use anchor_spl::{token_2022::{spl_token_2022::instruction::AuthorityType, Token2022}, token_interface::{set_authority, Mint, SetAuthority}};

use crate::constants::*;
use crate::modules::common::PDASignerSeeds;
use crate::modules::fund::ReceiptTokenMintAuthority;

#[derive(Accounts)]
pub struct TokenSetReceiptTokenMintAuthority<'info> {
    #[account(mut, address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS, mint::authority = admin)]
    pub receipt_token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [ReceiptTokenMintAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = receipt_token_mint_authority.bump,
    )]
    pub receipt_token_mint_authority: Account<'info, ReceiptTokenMintAuthority>,

    pub token_program: Program<'info, Token2022>,
}

impl<'info> TokenSetReceiptTokenMintAuthority<'info> {
    pub fn set_receipt_token_mint_authority(ctx: Context<Self>) -> Result<()> {
        let set_authority_cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                current_authority: ctx.accounts.admin.to_account_info(),
                account_or_mint: ctx.accounts.receipt_token_mint.to_account_info(),
            },
        );

        set_authority(
            set_authority_cpi_ctx,
            AuthorityType::MintTokens,
            Some(ctx.accounts.receipt_token_mint_authority.key()),
        )
    }
}
