use anchor_lang::prelude::*;
use anchor_spl::token_2022::spl_token_2022::instruction::AuthorityType;
use anchor_spl::token_2022::{set_authority, SetAuthority, Token2022};
use anchor_spl::token_interface::Mint;
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
};
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

use crate::constants::*;
use crate::modules::common::PDASignerSeeds;
use crate::modules::fund::{FundAccount, ReceiptTokenMintAuthority, UserFundAccount};
use crate::modules::reward::{RewardAccount, UserRewardAccount};

#[derive(Accounts)]
pub struct AdminReceiptTokenMintInitialContext<'info> {
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

    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        init,
        payer = payer,
        seeds = [b"extra-account-metas", receipt_token_mint.key().as_ref()],
        bump,
        space = ExtraAccountMetaList::size_of(
            extra_account_metas()?.len() + 2, // 2 is reserved space
        )?,
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
}

impl<'info> AdminReceiptTokenMintInitialContext<'info> {
    pub fn initialize_mint_authority_and_extra_account_meta_list(ctx: Context<Self>) -> Result<()> {

        // initialize_extra_account_meta_list
        let extra_account_metas = extra_account_metas()?;

        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &extra_account_metas,
        )?;

        // transfer_receipt_token_mint_authority
        ctx.accounts
            .receipt_token_mint_authority
            .initialize_if_needed(
                ctx.bumps.receipt_token_mint_authority,
                ctx.accounts.receipt_token_mint.key(),
            );

        let set_authority_cpi_ctx = CpiContext::new(
            ctx.accounts.receipt_token_program.to_account_info(),
            SetAuthority {
                current_authority: ctx.accounts.admin.to_account_info(),
                account_or_mint: ctx.accounts.receipt_token_mint.to_account_info(),
            },
        );

        set_authority(
            set_authority_cpi_ctx,
            AuthorityType::MintTokens,
            Some(ctx.accounts.receipt_token_mint_authority.key()),
        )
    }
}

#[derive(Accounts)]
pub struct AdminReceiptTokenMintContext<'info> {
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
        bump = receipt_token_mint_authority.bump,
        has_one = receipt_token_mint,
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

impl <'info> AdminReceiptTokenMintContext<'info> {
    pub fn update_extra_account_meta_list(ctx: Context<Self>) -> Result<()> {
        let extra_account_metas = extra_account_metas()?;

        ExtraAccountMetaList::update::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &extra_account_metas,
        )?;

        Ok(())
    }
}

fn extra_account_metas() -> Result<Vec<ExtraAccountMeta>> {
    let extra_account_metas = vec![
        // index 5, fund account
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: FundAccount::SEED.to_vec(),
                },
                Seed::AccountKey { index: 1 }, // receipt_token_mint
            ],
            false, // is_signer,
            true,  // is_writable
        )?,
        // index 6, source user fund account
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: UserFundAccount::SEED.to_vec(),
                },
                Seed::AccountKey { index: 1 }, // receipt_token_mint
                Seed::AccountData {
                    account_index: 0,
                    data_index: 32,
                    length: 32,
                }, // source_token_account.owner, data_index starts from the sum of the front indexes' bytes
            ],
            false, // is_signer
            true,  // is_writable
        )?,
        // index 7, destination user fund account
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: UserFundAccount::SEED.to_vec(),
                },
                Seed::AccountKey { index: 1 }, // receipt_token_mint
                Seed::AccountData {
                    account_index: 2,
                    data_index: 32,
                    length: 32,
                }, // destination_token_account.owner
            ],
            false, // is_signer
            true,  // is_writable
        )?,
        // index 8, reward account
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: RewardAccount::SEED.to_vec(),
                },
                Seed::AccountKey { index: 1 }, // receipt_token_mint
            ],
            false, // is_signer
            true,  // is_writable
        )?,
        // index 9, source user reward account
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: UserRewardAccount::SEED.to_vec(),
                },
                Seed::AccountKey { index: 1 }, // receipt_token_mint
                Seed::AccountData {
                    account_index: 0,
                    data_index: 32,
                    length: 32,
                }, // source_token_account.owner
            ],
            false, // is_signer
            true,  // is_writable
        )?,
        // index 10, destination user reward account
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: UserRewardAccount::SEED.to_vec(),
                },
                Seed::AccountKey { index: 1 }, // receipt_token_mint
                Seed::AccountData {
                    account_index: 2,
                    data_index: 32,
                    length: 32,
                }, // destination_token_account.owner
            ],
            false, // is_signer
            true,  // is_writable
        )?,
    ];

    Ok(extra_account_metas)
}