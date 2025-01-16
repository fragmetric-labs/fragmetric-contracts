use anchor_lang::prelude::*;

use super::{SPLStakePoolInterface, SPLStakePoolService};

pub(in crate::modules) struct SanctumSingleValidatorSPLStakePool;

impl SanctumSingleValidatorSPLStakePool {
    const ID: Pubkey = pubkey!("SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY");
}

impl anchor_lang::Id for SanctumSingleValidatorSPLStakePool {
    fn id() -> Pubkey {
        Self::ID
    }
}

impl SPLStakePoolInterface for SanctumSingleValidatorSPLStakePool {}

/// For now, sanctum single validator SPL stake pool is
/// identical to SPL stake pool.
///
/// In the future when there is change,
/// we can implement this service type.
pub(in crate::modules) type SanctumSingleValidatorSPLStakePoolService<'info> =
    SPLStakePoolService<'info, SanctumSingleValidatorSPLStakePool>;
