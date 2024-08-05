use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{mint_to, Mint, MintTo, TokenAccount},
};

pub(crate) trait MintExt<'info>
where
    Self: 'info,
{
    fn mint_token_cpi(
        &self,
        mint: &InterfaceAccount<'info, Mint>,
        to: &InterfaceAccount<'info, TokenAccount>,
        authority: AccountInfo<'info>,
        signer_seeds: Option<&[&[&[u8]]]>,
        amount: u64,
    ) -> Result<()>;
}

impl<'info> MintExt<'info> for Program<'info, Token2022> {
    fn mint_token_cpi(
        &self,
        mint: &InterfaceAccount<'info, Mint>,
        to: &InterfaceAccount<'info, TokenAccount>,
        authority: AccountInfo<'info>,
        signer_seeds: Option<&[&[&[u8]]]>,
        amount: u64,
    ) -> Result<()> {
        let mut mint_receipt_token_cpi_ctx = CpiContext::new(
            self.to_account_info(),
            MintTo {
                mint: mint.to_account_info(),
                to: to.to_account_info(),
                authority,
            },
        );

        if let Some(signer_seeds) = signer_seeds {
            mint_receipt_token_cpi_ctx = mint_receipt_token_cpi_ctx.with_signer(signer_seeds);
        }

        mint_to(mint_receipt_token_cpi_ctx, amount)
    }
}
