use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};
use fragmetric_util::{request, Upgradable};

use crate::{constants::*, fund::*, Empty};

#[derive(Accounts)]
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
        seeds = [FUND_TOKEN_AUTHORITY_SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = 8 + Empty::INIT_SPACE,
    )]
    pub fund_token_authority: Account<'info, Empty>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>, // fragSOL token mint account
    #[account(
        init,
        payer = admin,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = fund_token_authority,
        associated_token::token_program = token_program,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's fragSOL lock account

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundInitialize<'info> {
    pub fn initialize_fund(ctx: Context<Self>, request: FundInitializeRequest) -> Result<()> {
        let fund_token_authority_key = ctx.accounts.fund_token_authority.key();
        let receipt_token_mint_key = ctx.accounts.receipt_token_mint.key();
        msg!("receipt_token_mint: {}", receipt_token_mint_key);
        msg!("fund_token_authority: {}", fund_token_authority_key,);

        let args = FundInitializeArgs::from(request);
        ctx.accounts.fund.initialize(
            ctx.accounts.admin.key(),
            receipt_token_mint_key,
            // ctx.accounts.receipt_token_lock_account.key(),
        )?;
        ctx.accounts
            .fund
            .to_latest_version()
            .initialize(args.default_protocol_fee_rate, args.whitelisted_tokens)
    }
}

pub struct FundInitializeArgs {
    pub default_protocol_fee_rate: u16,
    pub whitelisted_tokens: Vec<TokenInfo>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
#[request(FundInitializeArgs)]
pub enum FundInitializeRequest {
    V1(FundInitializeRequestV1),
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundInitializeRequestV1 {
    pub default_protocol_fee_rate: u16,
    pub whitelisted_tokens: Vec<TokenInfo>,
}

impl From<FundInitializeRequest> for FundInitializeArgs {
    fn from(value: FundInitializeRequest) -> Self {
        match value {
            FundInitializeRequest::V1(value) => Self {
                default_protocol_fee_rate: value.default_protocol_fee_rate,
                whitelisted_tokens: value.whitelisted_tokens,
            },
        }
    }
}
