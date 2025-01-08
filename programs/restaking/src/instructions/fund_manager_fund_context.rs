use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::fund::FundAccount;
use crate::utils::{AccountLoaderExt, PDASeeds};

#[event_cpi]
#[derive(Accounts)]
pub struct FundManagerFundContext<'info> {
    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = fund_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub fund_account: AccountLoader<'info, FundAccount>,
}

// TODO: migration v0.3.2
#[derive(Accounts)]
pub struct FundManagerFundAccountCloseContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    #[account(
        mut,
        close = payer,
        seeds = [FundAccount::SEED, FRAGSOL_MINT_ADDRESS.as_ref()],
        bump,
    )]
    pub fund_account: AccountLoader<'info, FundAccount>,
}

use crate::modules::fund::UserFundAccount;

// TODO: migration v0.3.3 - only dev
#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct FundManagerUserFundContext<'info> {
    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    #[account(
        seeds = [FundAccount::SEED, FRAGSOL_MINT_ADDRESS.as_ref()],
        bump,
    )]
    pub fund_account: AccountLoader<'info, FundAccount>,

    #[account(
        mut,
        seeds = [UserFundAccount::SEED, FRAGSOL_MINT_ADDRESS.as_ref(), user.as_ref()],
        bump,
    )]
    pub user_fund_account: Account<'info, UserFundAccount>,
}

use anchor_spl::token_interface::{TokenAccount, TokenInterface};

// TODO: migration v0.4.0
#[derive(Accounts)]
pub struct FundManagerChangeFundTokenAccountContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    #[account(
        seeds = [FundAccount::SEED, FRAGSOL_MINT_ADDRESS.as_ref()],
        bump,
    )]
    pub fund_account: AccountLoader<'info, FundAccount>,

    #[account(
        seeds = [FundAccount::RESERVE_SEED, FRAGSOL_MINT_ADDRESS.as_ref()],
        bump,
    )]
    pub fund_reserve_account: SystemAccount<'info>,

    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub token_program: Interface<'info, TokenInterface>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = fund_account,
        associated_token::token_program = token_program,
    )]
    pub old_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = fund_reserve_account,
        associated_token::token_program = token_program,
    )]
    pub new_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
}
