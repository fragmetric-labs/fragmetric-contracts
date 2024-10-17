use anchor_lang::prelude::*;
use spl_tlv_account_resolution::{account::ExtraAccountMeta, seeds::Seed};

use crate::modules::fund::*;
use crate::modules::reward::{RewardAccount, UserRewardAccount};
use crate::utils::PDASeeds;

pub(crate) fn extra_account_metas_len() -> Result<usize> {
    Ok(extra_account_metas()?.len())
}

pub(in crate::modules) fn extra_account_metas() -> Result<Vec<ExtraAccountMeta>> {
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
