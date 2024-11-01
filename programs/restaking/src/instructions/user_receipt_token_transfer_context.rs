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

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::UserTransferredReceiptToken;
use crate::modules::{fund::*, reward::*};
use crate::utils::{AccountLoaderExt, PDASeeds};

// Order of accounts matters for this struct.
// The first 4 accounts are the accounts required for token transfer (source, mint, destination, owner)
// Remaining accounts are the extra accounts required from the ExtraAccountMetaList account
// These accounts are provided via CPI to this program from the token2022 program
#[derive(Accounts)]
pub struct UserReceiptTokenTransferContext<'info> {
    #[account(
        token::mint = receipt_token_mint,
        token::authority = owner,
    )]
    pub source_receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        token::mint = receipt_token_mint,
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
        bump = fund_account.get_bump(),
        has_one = receipt_token_mint,
        constraint = fund_account.is_latest_version() @ ErrorCode::InvalidDataVersionError,
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,

    #[account(
        mut,
        seeds = [UserFundAccount::SEED, receipt_token_mint.key().as_ref(), source_receipt_token_account.owner.as_ref()],
        bump,
    )]
    /// CHECK: will be deserialized in runtime
    pub source_fund_account: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [UserFundAccount::SEED, receipt_token_mint.key().as_ref(), destination_receipt_token_account.owner.as_ref()],
        bump,
    )]
    /// CHECK: will be deserialized in runtime
    pub destination_fund_account: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = reward_account.load()?.is_latest_version() @ ErrorCode::InvalidDataVersionError,
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,

    #[account(
        mut,
        seeds = [UserRewardAccount::SEED, receipt_token_mint.key().as_ref(), source_receipt_token_account.owner.as_ref()],
        bump,
    )]
    /// CHECK: will be deserialized in runtime
    pub source_reward_account: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [UserRewardAccount::SEED, receipt_token_mint.key().as_ref(), destination_receipt_token_account.owner.as_ref()],
        bump,
    )]
    /// CHECK: will be deserialized in runtime
    pub destination_reward_account: UncheckedAccount<'info>,
}

impl<'info> UserReceiptTokenTransferContext<'info> {
    pub fn handle_transfer(ctx: Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts.assert_is_transferring()?;

        /* token transfer is temporarily disabled */
        err!(ErrorCode::TokenNotTransferableError)?;

        let receipt_token_mint = ctx.accounts.receipt_token_mint.key();
        emit!(UserTransferredReceiptToken {
            receipt_token_mint,
            transferred_receipt_token_amount: amount,
            source_receipt_token_account: ctx.accounts.source_receipt_token_account.key(),
            source: ctx.accounts.source_receipt_token_account.owner,
            source_fund_account: UserFundAccount::placeholder(
                ctx.accounts.source_fund_account.key(),
                receipt_token_mint,
                ctx.accounts.source_receipt_token_account.amount,
            ),
            destination_receipt_token_account: ctx.accounts.destination_receipt_token_account.key(),
            destination: ctx.accounts.destination_receipt_token_account.owner,
            destination_fund_account: UserFundAccount::placeholder(
                ctx.accounts.destination_fund_account.key(),
                receipt_token_mint,
                ctx.accounts.destination_receipt_token_account.amount,
            ),
        });

        Ok(())
    }

    fn assert_is_transferring(&self) -> Result<()> {
        let source_token_account_info = self.source_receipt_token_account.to_account_info();
        let mut account_data_ref = source_token_account_info.try_borrow_mut_data()?;
        let mut account = PodStateWithExtensionsMut::<PodAccount>::unpack(*account_data_ref)?;
        let account_extension = account.get_extension_mut::<TransferHookAccount>()?;

        if !bool::from(account_extension.transferring) {
            err!(ErrorCode::TokenNotTransferringException)?
        }

        Ok(())
    }

    /*
    fn update_reward_pools(
        receipt_token_mint: Pubkey,
        amount: u64,
        reward_account: &mut RewardAccount,
        source_reward_account: &mut UserRewardAccount,
        destination_reward_account: &mut UserRewardAccount,
    ) -> Result<()> {
        let current_slot = Clock::get()?.slot;

        let (from_user_update, to_user_update) = reward_account
            .update_reward_pools_token_allocation(
                receipt_token_mint,
                amount,
                None,
                Some(source_reward_account),
                Some(destination_reward_account),
                current_slot,
            )?;

        emit!(UserUpdatedRewardPool::new(
            receipt_token_mint,
            from_user_update,
            to_user_update
        ));

        Ok(())
    }
    */
}
