use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use spl_tlv_account_resolution::state::ExtraAccountMetaList;

use crate::constants::*;
use crate::modules::fund::*;

// will be used only once
#[event_cpi]
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
            receipt_token_extra_account_metas_len()? + 2, // 2 is reserved space
        )?,
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct AdminReceiptTokenMintExtraAccountMetaListUpdateContext<'info> {
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        mut,
        seeds = [b"extra-account-metas", receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
}
