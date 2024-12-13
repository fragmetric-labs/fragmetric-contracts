use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::fund::*;
use crate::utils::{AccountLoaderExt, PDASeeds};

#[derive(Accounts)]
pub struct FundManagerFundSupportedTokenContext<'info> {
    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = fund_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub fund_account: AccountLoader<'info, FundAccount>,

    pub supported_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub supported_token_program: Interface<'info, TokenInterface>,

    #[account(
        associated_token::mint = supported_token_mint,
        associated_token::authority = fund_account,
        associated_token::token_program = supported_token_program,
    )]
    pub supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
}
