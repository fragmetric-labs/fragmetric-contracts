use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::Mint;
use spl_tlv_account_resolution::state::ExtraAccountMetaList;

use crate::constants::*;
use crate::modules::fund::*;
use crate::utils::PDASeeds;

// will be used only once
#[derive(Accounts)]
pub struct AdminReceiptTokenMintAuthorityInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub receipt_token_program: Program<'info, Token2022>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = payer,
        seeds = [ReceiptTokenMintAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = 8 + ReceiptTokenMintAuthority::INIT_SPACE,
    )]
    pub receipt_token_mint_authority: Account<'info, ReceiptTokenMintAuthority>,
}

// will be used only once
#[derive(Accounts)]
pub struct AdminReceiptTokenMintExtraAccountMetaListInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        init,
        payer = payer,
        seeds = [b"extra-account-metas", receipt_token_mint.key().as_ref()],
        bump,
        space = ExtraAccountMetaList::size_of(
            extra_account_metas_len()? + 2, // 2 is reserved space
        )?,
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct AdminReceiptTokenMintExtraAccountMetaListUpdateContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub receipt_token_program: Program<'info, Token2022>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [ReceiptTokenMintAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = receipt_token_mint_authority.bump(),
    )]
    pub receipt_token_mint_authority: Account<'info, ReceiptTokenMintAuthority>,

    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        mut,
        seeds = [b"extra-account-metas", receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
}
