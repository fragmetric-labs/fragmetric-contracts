use anchor_lang::prelude::*;
use anchor_spl::{token_2022::Token2022, token_interface::Mint};
use spl_tlv_account_resolution::{account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList};
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

use crate::{constants::*, fund::*};

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    /// CHECK: ExtraAccountaMetaList Account, must use these seeds
    #[account(
        init,
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump,
        space = ExtraAccountMetaList::size_of(
            InitializeExtraAccountMetaList::extra_account_metas()?.len(),
        )?,
        payer = payer
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    // #[account(
    //     init_if_needed,
    //     payer = payer,
    //     space = 8 + WhitelistedDestinationToken::INIT_SPACE,
    //     seeds = [b"whitelisted_destination"],
    //     bump,
    // )]
    // pub whitelisted_destination_token: Account<'info, WhitelistedDestinationToken>,

    #[account(
        mut,
        seeds = [FUND_SEED, mint.key().as_ref()],
        bump,
    )]
    pub fund: Account<'info, Fund>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

// #[account]
// #[derive(InitSpace)]
// pub struct WhitelistedDestinationToken {
//     #[max_len(50)]
//     pub addresses: Vec<Pubkey>,
// }

impl<'info> InitializeExtraAccountMetaList<'info> {
    pub fn initialize_extra_account_meta_list(ctx: Context<Self>) -> Result<()> {
        let extra_account_meta_list_key = ctx.accounts.extra_account_meta_list.key();
        msg!("extra_account_meta_list_key: {:?}", extra_account_meta_list_key);

        let extra_account_metas = Self::extra_account_metas()?;

        // initialize ExtraAccountMetaList account with extra accounts
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &extra_account_metas
        )?;

        Ok(())
    }

    pub fn extra_account_metas() -> Result<Vec<ExtraAccountMeta>> {
        let extra_account_metas = vec![
            // index 5, fund pda
            ExtraAccountMeta::new_with_seeds(
                &[
                    Seed::Literal { bytes: FUND_SEED.to_vec(), },
                    Seed::AccountKey { index: 1 }, // mint address
                ],
                false, // is_signer,
                true, // is_writable
            )?,
        ];

        Ok(extra_account_metas)
    }
}
