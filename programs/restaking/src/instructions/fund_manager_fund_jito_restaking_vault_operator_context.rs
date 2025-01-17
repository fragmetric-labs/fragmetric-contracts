use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::errors::ErrorCode;
use crate::utils::PDASeeds;
use crate::{constants::*, modules::fund::FundAccount, utils::AccountLoaderExt};

#[event_cpi]
#[derive(Accounts)]
pub struct FundManagerFundJitoRestakingVaultOperatorInitialContext<'info> {
    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = fund_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub fund_account: AccountLoader<'info, FundAccount>,

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: just need to validate vault state is owned by the vault program
    #[account(address = JITO_VAULT_PROGRAM_ID)]
    pub vault_program: UncheckedAccount<'info>,

    /// CHECK: will be validated by pricing service
    pub vault_account: UncheckedAccount<'info>,

    /// CHECK: just need to validate vault operator is owned by the jito restaking program
    #[account(address = JITO_RESTAKING_PROGRAM_ID)]
    pub jito_restaking_program: UncheckedAccount<'info>,

    /// CHECK: just verify that it's owner is Jito Restaking Program
    pub vault_operator: UncheckedAccount<'info>,
}
