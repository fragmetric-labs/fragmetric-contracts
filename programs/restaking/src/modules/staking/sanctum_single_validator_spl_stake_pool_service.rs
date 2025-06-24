use anchor_lang::prelude::*;

use super::SPLStakePoolService;

pub struct SanctumSingleValidatorSPLStakePool;

impl anchor_lang::Id for SanctumSingleValidatorSPLStakePool {
    fn id() -> Pubkey {
        pubkey!("SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY")
    }
}

/// For now, sanctum single validator SPL stake pool is
/// identical to SPL stake pool.
///
/// In the future when there is change,
/// we can implement this service type.
pub(in crate::modules) type SanctumSingleValidatorSPLStakePoolService<'info> =
    SPLStakePoolService<'info, SanctumSingleValidatorSPLStakePool>;
