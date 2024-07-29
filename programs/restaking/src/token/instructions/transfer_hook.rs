use anchor_lang::prelude::*;
use anchor_spl::{token_2022::spl_token_2022::{extension::{transfer_hook::TransferHookAccount, BaseStateWithExtensionsMut, PodStateWithExtensionsMut}, pod::PodAccount}, token_interface::{Mint, TokenAccount}};

use crate::{constants::*, error::ErrorCode, fund::*};

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
        seeds = [FUND_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub fund: Account<'info, Fund>,
}

impl<'info> TokenTransferHook<'info> {
    pub fn transfer_hook(ctx: Context<Self>, amount: u64) -> Result<()> {
        // for destination in ctx.accounts.whitelisted_destination_token.addresses.iter() {
        //     if destination == &ctx.accounts.destination_token.key() {
        //         msg!("Transferring to whitelisted destination token account {}!", destination.key());
        //     }
        // }

        Self::check_is_transferring(&ctx)?;

        let source_token_account_key = ctx.accounts.source_token_account.key();
        let destination_token_account_key = ctx.accounts.destination_token_account.key();
        msg!("transfer hook executed! amount {} passed from {:?} to {:?}", amount, source_token_account_key, destination_token_account_key);
        msg!("fund pda: {:?}", ctx.accounts.fund.key());

        Ok(())
    }

    fn check_is_transferring(ctx: &Context<Self>) -> Result<()> {
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
