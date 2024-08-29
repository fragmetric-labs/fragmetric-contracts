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

use crate::{common::*, constants::*, error::ErrorCode, fund::*, token::*};

#[derive(Accounts)]
pub struct TokenTransferHook<'info> {
    #[account(
        token::mint = receipt_token_mint,
        token::authority = owner,
    )]
    pub source_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        token::mint = receipt_token_mint,
    )]
    pub destination_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: source token account owner, can be SystemAccount or PDA owned by another program
    pub owner: UncheckedAccount<'info>,

    /// CHECK: ExtraAccountMetaList account
    #[account(
        seeds = [b"extra-account-metas", receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    // pub whitelisted_destination_token: Account<'info, WhitelistedDestinationToken>,
    #[account(
        mut,
        seeds = [Fund::SEED, receipt_token_mint.key().as_ref()],
        bump = fund.bump,
        has_one = receipt_token_mint,
    )]
    pub fund: Box<Account<'info, Fund>>,

    #[account(
        mut,
        seeds = [UserReceipt::SEED, source_token_account.owner.key().as_ref(), receipt_token_mint.key().as_ref()],
        bump = source_user_receipt.bump,
        constraint = source_user_receipt.user == source_token_account.owner.key(),
        has_one = receipt_token_mint,
    )]
    pub source_user_receipt: Box<Account<'info, UserReceipt>>,

    #[account(
        mut,
        seeds = [UserReceipt::SEED, destination_token_account.owner.key().as_ref(), receipt_token_mint.key().as_ref()],
        bump = destination_user_receipt.bump,
        constraint = destination_user_receipt.user == destination_token_account.owner.key(),
        has_one = receipt_token_mint,
    )]
    pub destination_user_receipt: Box<Account<'info, UserReceipt>>,
}

impl<'info> TokenTransferHook<'info> {
    pub fn transfer_hook(ctx: Context<Self>, amount: u64) -> Result<()> {
        // for destination in ctx.accounts.whitelisted_destination_token.addresses.iter() {
        //     if destination == &ctx.accounts.destination_token.key() {
        //         msg!("Transferring to whitelisted destination token account {}!", destination.key());
        //     }
        // }

        Self::check_token_transferring(&ctx)?;
        Self::call_transfer_hook(&ctx, amount)?;

        // Update source/destination user_receipt's receipt_token_amount
        let source_token_account_total_amount = ctx.accounts.source_token_account.amount;
        ctx.accounts
            .source_user_receipt
            .set_receipt_token_amount(source_token_account_total_amount);
        let destination_token_account_total_amount = ctx.accounts.destination_token_account.amount;
        ctx.accounts
            .destination_user_receipt
            .set_receipt_token_amount(destination_token_account_total_amount);

        emit!(UserTransferredReceiptToken {
            transferred_receipt_token_mint: ctx.accounts.receipt_token_mint.key(),
            transferred_receipt_token_amount: amount,
            source_receipt_token_account: ctx.accounts.source_token_account.key(),
            source_user: ctx.accounts.source_token_account.owner,
            source_user_receipt: Clone::clone(&ctx.accounts.source_user_receipt),
            destination_receipt_token_account: ctx.accounts.destination_token_account.key(),
            destination_user: ctx.accounts.destination_token_account.owner,
            destination_user_receipt: Clone::clone(&ctx.accounts.destination_user_receipt),
        });

        Ok(())
    }

    fn call_transfer_hook(ctx: &Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts.receipt_token_mint.transfer_hook(
            Some(&ctx.accounts.source_token_account),
            Some(&ctx.accounts.destination_token_account),
            &ctx.accounts.fund,
            amount,
        )
    }

    fn check_token_transferring(ctx: &Context<Self>) -> Result<()> {
        let source_token_account_info = ctx.accounts.source_token_account.to_account_info();
        let mut account_data_ref = source_token_account_info.try_borrow_mut_data()?;
        let mut account = PodStateWithExtensionsMut::<PodAccount>::unpack(*account_data_ref)?;
        let account_extension = account.get_extension_mut::<TransferHookAccount>()?;

        if !bool::from(account_extension.transferring) {
            return err!(ErrorCode::TokenNotCurrentlyTransferring);
        }

        Ok(())
    }
}
