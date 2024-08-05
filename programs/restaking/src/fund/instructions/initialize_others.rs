use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use fragmetric_util::Upgradable;

use crate::{constants::*, fund::*};

#[derive(Accounts)]
pub struct FundInitializeOthers<'info> {
    #[account(mut, address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [FUND_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub fund: Account<'info, Fund>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,
}

impl<'info> FundInitializeOthers<'info> {
    pub fn initialize_sol_withdrawal_fee_rate(ctx: Context<Self>, sol_withdrawal_fee_rate: u16) -> Result<()> {
        ctx.accounts.fund
            .to_latest_version()
            .set_sol_withdrawal_fee_rate(sol_withdrawal_fee_rate)
    }

    pub fn initialize_whitelisted_tokens(ctx: Context<Self>, whitelisted_tokens: Vec<TokenInfo>) -> Result<()> {
        ctx.accounts.fund
            .to_latest_version()
            .set_whitelisted_tokens(whitelisted_tokens)
    }
}
