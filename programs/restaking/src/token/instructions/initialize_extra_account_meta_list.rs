use anchor_lang::prelude::*;
use anchor_spl::{token_2022::Token2022, token_interface::Mint};
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
};
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

use crate::{common::*, constants::*, fund::*};

#[derive(Accounts)]
pub struct TokenInitializeExtraAccountMetaList<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    /// CHECK: ExtraAccountaMetaList Account, must use these seeds
    #[account(
        init_if_needed,
        seeds = [b"extra-account-metas", receipt_token_mint.key().as_ref()],
        bump,
        space = ExtraAccountMetaList::size_of(
            TokenInitializeExtraAccountMetaList::extra_account_metas()?.len(),
        )?,
        payer = payer
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    // #[account(
    //     init_if_needed,
    //     payer = payer,
    //     space = 8 + WhitelistedDestinationToken::INIT_SPACE,
    //     seeds = [b"whitelisted_destination"],
    //     bump,
    // )]
    // pub whitelisted_destination_token: Account<'info, WhitelistedDestinationToken>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

// #[account]
// #[derive(InitSpace)]
// pub struct WhitelistedDestinationToken {
//     #[max_len(50)]
//     pub addresses: Vec<Pubkey>,
// }

impl<'info> TokenInitializeExtraAccountMetaList<'info> {
    pub fn initialize_extra_account_meta_list(ctx: Context<Self>) -> Result<()> {
        let extra_account_meta_list_key = ctx.accounts.extra_account_meta_list.key();
        msg!(
            "extra_account_meta_list_key: {:?}",
            extra_account_meta_list_key
        );

        let extra_account_metas = Self::extra_account_metas()?;

        // initialize ExtraAccountMetaList account with extra accounts
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &extra_account_metas,
        )?;

        Ok(())
    }

    pub fn update_extra_account_meta_list(ctx: Context<Self>) -> Result<()> {
        let extra_account_meta_list_key = ctx.accounts.extra_account_meta_list.key();
        msg!(
            "extra_account_meta_list_key: {:?}",
            extra_account_meta_list_key
        );

        let extra_account_metas = Self::extra_account_metas()?;

        ExtraAccountMetaList::update::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &extra_account_metas,
        )?;

        Ok(())
    }

    pub fn extra_account_metas() -> Result<Vec<ExtraAccountMeta>> {
        let extra_account_metas = vec![
            // index 5, fund pda
            ExtraAccountMeta::new_with_seeds(
                &[
                    Seed::Literal {
                        bytes: Fund::SEED.to_vec(),
                    },
                    Seed::AccountKey { index: 1 }, // receipt_token_mint
                ],
                false, // is_signer,
                true,  // is_writable
            )?,
            // index 6, source user receipt
            ExtraAccountMeta::new_with_seeds(
                &[
                    Seed::Literal {
                        bytes: UserReceipt::SEED.to_vec(),
                    },
                    Seed::AccountData {
                        account_index: 0,
                        data_index: 32,
                        length: 32,
                    }, // source_token_account.owner, data_index starts from the sum of the front indexes' bytes
                    Seed::AccountKey { index: 1 }, // receipt_token_mint
                ],
                false, // is_signer
                true,  // is_writable
            )?,
            // index 7, destination user receipt
            ExtraAccountMeta::new_with_seeds(
                &[
                    Seed::Literal {
                        bytes: UserReceipt::SEED.to_vec(),
                    },
                    Seed::AccountData {
                        account_index: 2,
                        data_index: 32,
                        length: 32,
                    }, // destination_token_account.owner
                    Seed::AccountKey { index: 1 }, // receipt_token_mint
                ],
                false, // is_signer
                true,  // is_writable
            )?,
        ];

        Ok(extra_account_metas)
    }
}
