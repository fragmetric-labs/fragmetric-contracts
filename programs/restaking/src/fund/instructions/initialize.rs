use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

use crate::{constants::*, fund::*};

#[derive(Accounts)]
#[instruction(request: FundInitializeRequest)]
pub struct FundInitialize<'info> {
    #[account(mut, address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        seeds = [FUND_SEED, receipt_token_mint.key().as_ref()], // fund + <any receipt token mint account>
        bump,
        space = 8 + Fund::INIT_SPACE,
    )]
    pub fund: Account<'info, Fund>,

    #[account(
        init,
        payer = admin,
        seeds = [RECEIPT_TOKEN_AUTHORITY_SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = 8 + ReceiptTokenAuthority::INIT_SPACE,
    )]
    pub receipt_token_authority: Account<'info, ReceiptTokenAuthority>,

    // NOTE will be initialized externally
    #[account(
        init,
        payer = admin,
        seeds = [request.receipt_token_name.as_bytes()],
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

impl<'info> FundInitialize<'info> {
    #[allow(unused_variables)]
    pub fn initialize_fund(ctx: Context<Self>, request: FundInitializeRequest) -> Result<()> {
        let receipt_token_mint_addr = ctx.accounts.receipt_token_mint.key();
        msg!("receipit_token_mint: {}", receipt_token_mint_addr);

        ctx.accounts.fund.initialize(
            ctx.accounts.admin.key(),
            request.default_protocol_fee_rate,
            ctx.accounts.receipt_token_mint.key(),
            request.whitelisted_tokens,
            // ctx.accounts.receipt_token_lock_account.key(),
        )
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundInitializeRequest {
    pub receipt_token_name: String,
    pub default_protocol_fee_rate: u16,
    pub whitelisted_tokens: Vec<TokenInfo>,
}
