use anchor_lang::prelude::*;
use anchor_spl::{token_2022::spl_token_2022::{extension::{transfer_hook::TransferHookAccount, BaseStateWithExtensionsMut, PodStateWithExtensionsMut}, pod::PodAccount}, token_interface::{Mint, TokenAccount}};

use crate::{constants::*, error::ErrorCode, fund::*};

#[derive(Accounts)]
pub struct TransferHook<'info> {
    #[account(
        token::mint = mint,
        token::authority = owner,
    )]
    pub source_token: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>, // receipt token mint account

    #[account(
        token::mint = mint,
    )]
    pub destination_token: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: source token account owner, can be SystemAccount or PDA owned by another program
    pub owner: UncheckedAccount<'info>,

    /// CHECK: ExtraAccountMetaList account
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump,
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    // pub whitelisted_destination_token: Account<'info, WhitelistedDestinationToken>,

    #[account(
        mut,
        seeds = [FUND_SEED, mint.key().as_ref()],
        bump,
    )]
    pub fund: Account<'info, Fund>,
}

impl<'info> TransferHook<'info> {
    pub fn transfer_hook(ctx: Context<Self>, amount: u64) -> Result<()> {
        // for destination in ctx.accounts.whitelisted_destination_token.addresses.iter() {
        //     if destination == &ctx.accounts.destination_token.key() {
        //         msg!("Transferring to whitelisted destination token account {}!", destination.key());
        //     }
        // }

        Self::check_is_transferring(&ctx)?;

        let source_token_key = ctx.accounts.source_token.key();
        let destination_token_key = ctx.accounts.destination_token.key();
        msg!("transfer hook executed! amount {} passed from {:?} to {:?}", amount, source_token_key, destination_token_key);
        msg!("fund pda: {:?}", ctx.accounts.fund.key());

        Ok(())
    }

    fn check_is_transferring(ctx: &Context<Self>) -> Result<()> {
        let source_token_info = ctx.accounts.source_token.to_account_info();
        let mut account_data_ref = source_token_info.try_borrow_mut_data()?;
        let mut account = PodStateWithExtensionsMut::<PodAccount>::unpack(*account_data_ref)?;
        let account_extension = account.get_extension_mut::<TransferHookAccount>()?;

        if !bool::from(account_extension.transferring) {
            return err!(ErrorCode::TokenNotCurrentlyTransferring);
        }

        Ok(())
    }
}
