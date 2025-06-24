use anchor_lang::prelude::*;

use super::SPLStakePoolService;

pub struct SanctumMultiValidatorSPLStakePool;

impl anchor_lang::Id for SanctumMultiValidatorSPLStakePool {
    fn id() -> Pubkey {
        pubkey!("SPMBzsVUuoHA4Jm6KunbsotaahvVikZs1JyTW6iJvbn")
    }
}

/// For now, sanctum multi validator SPL stake pool is
/// identical to SPL stake pool.
///
/// In the future when there is change,
/// we can implement this service type.
pub(in crate::modules) type SanctumMultiValidatorSPLStakePoolService<'info> =
    SPLStakePoolService<'info, SanctumMultiValidatorSPLStakePool>;
