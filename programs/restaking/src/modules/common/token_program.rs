use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{burn, Burn, Mint, TokenAccount},
};
use anchor_spl::token_2022::{mint_to, MintTo};

pub trait TokenProgramExt<'info>
where
    Self: 'info,
{
    fn mint_token_cpi(
        &self,
        mint: &mut InterfaceAccount<'info, Mint>,
        to: &mut InterfaceAccount<'info, TokenAccount>,
        authority: AccountInfo<'info>,
        signer_seeds: Option<&[&[&[u8]]]>,
        amount: u64,
    ) -> Result<()>;

    fn burn_token_cpi(
        &self,
        mint: &mut InterfaceAccount<'info, Mint>,
        from: &mut InterfaceAccount<'info, TokenAccount>,
        authority: AccountInfo<'info>,
        signer_seeds: Option<&[&[&[u8]]]>,
        amount: u64,
    ) -> Result<()>;
}

impl<'info> TokenProgramExt<'info> for Program<'info, Token2022> {
    fn burn_token_cpi(
        &self,
        mint: &mut InterfaceAccount<'info, Mint>,
        from: &mut InterfaceAccount<'info, TokenAccount>,
        authority: AccountInfo<'info>,
        signer_seeds: Option<&[&[&[u8]]]>,
        amount: u64,
    ) -> Result<()> {
        let mut burn_receipt_token_cpi_ctx = CpiContext::new(
            self.to_account_info(),
            Burn {
                mint: mint.to_account_info(),
                from: from.to_account_info(),
                authority,
            },
        );

        if let Some(signer_seeds) = signer_seeds {
            burn_receipt_token_cpi_ctx = burn_receipt_token_cpi_ctx.with_signer(signer_seeds);
        }

        burn(burn_receipt_token_cpi_ctx, amount)?;
        mint.reload()?;
        from.reload()
    }

    fn mint_token_cpi(
        &self,
        mint: &mut InterfaceAccount<'info, Mint>,
        to: &mut InterfaceAccount<'info, TokenAccount>,
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

        mint_to(mint_receipt_token_cpi_ctx, amount)?;
        mint.reload()?;
        to.reload()
    }
}
