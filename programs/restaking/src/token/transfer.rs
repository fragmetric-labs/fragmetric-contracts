use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::Fund;

pub(crate) trait TransferHookExt<'info> {
    fn transfer_hook(
        &self,
        source_token_account: Option<&InterfaceAccount<'info, TokenAccount>>,
        destination_token_account: Option<&InterfaceAccount<'info, TokenAccount>>,
        fund: &Account<'info, Fund>,
        amount: u64,
    ) -> Result<()>;
}

impl<'info> TransferHookExt<'info> for InterfaceAccount<'info, Mint> {
    fn transfer_hook(
        &self,
        source_token_account: Option<&InterfaceAccount<'info, TokenAccount>>,
        destination_token_account: Option<&InterfaceAccount<'info, TokenAccount>>,
        fund: &Account<'info, Fund>,
        amount: u64,
    ) -> Result<()> {
        msg!(
            "transfer hook executed! amount {} passed from {:?} to {:?}",
            amount,
            source_token_account.map(Key::key),
            destination_token_account.map(Key::key),
        );
        msg!("fund pda: {:?}", fund.key());

        Ok(())
    }
}
