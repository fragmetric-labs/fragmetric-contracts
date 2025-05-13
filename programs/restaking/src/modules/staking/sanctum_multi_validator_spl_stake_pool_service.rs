use anchor_lang::prelude::*;

use super::{SPLStakePoolInterface, SPLStakePoolService};

pub(in crate::modules) struct SanctumMultiValidatorSPLStakePool;

impl SanctumMultiValidatorSPLStakePool {
    const ID: Pubkey = pubkey!("SPMBzsVUuoHA4Jm6KunbsotaahvVikZs1JyTW6iJvbn");
}

impl anchor_lang::Id for SanctumMultiValidatorSPLStakePool {
    fn id() -> Pubkey {
        Self::ID
    }
}

impl SPLStakePoolInterface for SanctumMultiValidatorSPLStakePool {}

/// For now, sanctum multi validator SPL stake pool is
/// identical to SPL stake pool.
///
/// In the future when there is change,
/// we can implement this service type.
pub(in crate::modules) type SanctumMultiValidatorSPLStakePoolService<'info> =
    SPLStakePoolService<'info, SanctumMultiValidatorSPLStakePool>;
