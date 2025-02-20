use anchor_lang::prelude::*;
use anchor_spl::token_interface::*;
use spl_tlv_account_resolution::account::ExtraAccountMeta;
use spl_tlv_account_resolution::seeds::Seed;
use spl_tlv_account_resolution::state::ExtraAccountMetaList;
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

use crate::modules::reward;
use crate::utils::PDASeeds;

use super::*;

pub struct FundReceiptTokenConfigurationService<'a, 'info> {
    extra_account_meta_list: &'a UncheckedAccount<'info>,
}

impl<'a, 'info> FundReceiptTokenConfigurationService<'a, 'info> {
    pub fn new(extra_account_meta_list: &'a UncheckedAccount<'info>) -> Result<Self> {
        Ok(Self {
            extra_account_meta_list,
        })
    }

    pub fn process_initialize_extra_account_meta_list(&self) -> Result<()> {
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut self.extra_account_meta_list.try_borrow_mut_data()?,
            &receipt_token_extra_account_metas()?,
        )?;
        Ok(())
    }

    pub fn process_update_extra_account_meta_list_if_needed(&self) -> Result<()> {
        ExtraAccountMetaList::update::<ExecuteInstruction>(
            &mut self.extra_account_meta_list.try_borrow_mut_data()?,
            &receipt_token_extra_account_metas()?,
        )?;
        Ok(())
    }
}

pub(crate) fn receipt_token_extra_account_metas_len() -> Result<usize> {
    Ok(receipt_token_extra_account_metas()?.len())
}

fn receipt_token_extra_account_metas() -> Result<Vec<ExtraAccountMeta>> {
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
        // index 6, reward account
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: reward::RewardAccount::SEED.to_vec(),
                },
                Seed::AccountKey { index: 1 }, // receipt_token_mint
            ],
            false, // is_signer
            true,  // is_writable
        )?,
        // index 7, source user fund account
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
        // index 8, source user reward account
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: reward::UserRewardAccount::SEED.to_vec(),
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
        // index 9, destination user fund account
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
        // index 10, destination user reward account
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: reward::UserRewardAccount::SEED.to_vec(),
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
        // index 11, event authority
        ExtraAccountMeta::new_with_seeds(
            &[Seed::Literal {
                bytes: b"__event_authority".to_vec(),
            }],
            false, // is_signer,
            false, // is_writable
        )?,
        // index 12, this program
        ExtraAccountMeta::new_with_pubkey(
            &crate::ID,
            false, // is_signer,
            false, // is_writable
        )?,
    ];

    Ok(extra_account_metas)
}
