use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::spl_token_2022::{
        extension::{
            transfer_hook::TransferHookAccount, BaseStateWithExtensionsMut,
            PodStateWithExtensionsMut,
        },
        pod::PodAccount,
    },
    token_interface::{Mint, TokenAccount},
};

use crate::errors::ErrorCode;
use crate::modules::{fund::*, reward::*};
use crate::utils::{AccountLoaderExt, PDASeeds};

// Order of accounts matters for this struct.
// The first 4 accounts are the accounts required for token transfer (source, mint, destination, owner)
// Remaining accounts are the extra accounts required from the ExtraAccountMetaList account
// These accounts are provided via CPI to this program from the token2022 program
#[derive(Accounts)]
pub struct UserReceiptTokenTransferContext<'info> {
    #[account(
        associated_token::mint = receipt_token_mint,
        associated_token::authority = source_receipt_token_account.owner,
        associated_token::token_program = anchor_spl::token_2022::Token2022::id(),
    )]
    pub source_receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        associated_token::mint = receipt_token_mint,
        associated_token::authority = destination_receipt_token_account.owner,
        associated_token::token_program = anchor_spl::token_2022::Token2022::id(),
    )]
    pub destination_receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: source token account owner, can be SystemAccount or PDA owned by another program
    pub owner: UncheckedAccount<'info>,

    /// CHECK: ExtraAccountMetaList account
    #[account(
        seeds = [b"extra-account-metas", receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = fund_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub fund_account: AccountLoader<'info, FundAccount>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = reward_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,
    // remaining extra accounts shall be provided through remaining_accounts
}

impl<'info> UserReceiptTokenTransferContext<'info> {
    pub fn assert_is_transferring(&self) -> Result<()> {
        let source_token_account_info = self.source_receipt_token_account.to_account_info();
        let mut account_data_ref = source_token_account_info.try_borrow_mut_data()?;
        let mut account = PodStateWithExtensionsMut::<PodAccount>::unpack(*account_data_ref)?;
        let account_extension = account.get_extension_mut::<TransferHookAccount>()?;

        if !bool::from(account_extension.transferring) {
            err!(ErrorCode::TokenNotTransferringException)?
        }

        Ok(())
    }
}
