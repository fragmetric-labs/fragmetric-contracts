#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

pub(crate) use custom_account::*;
pub(crate) use init_if_needed_by_pda::*;
pub(crate) use system_program::*;

mod custom_account;
mod init_if_needed_by_pda;
mod system_program;

/// drops sub-decimal values.
/// when both numerator and denominator are zero, returns amount.
pub(crate) fn proportional_amount(amount: u64, numerator: u64, denominator: u64) -> Option<u64> {
    if numerator == 0 && denominator == 0 {
        return Some(amount);
    }

    u64::try_from(
        (amount as u128)
            .checked_mul(numerator as u128)?
            .checked_div(denominator as u128)?,
    )
    .ok()
}

#[cfg(target_os = "solana")]
pub(crate) fn timestamp_now() -> Result<i64> {
    Ok(Clock::get()?.unix_timestamp)
}

#[cfg(not(target_os = "solana"))]
pub(crate) fn timestamp_now() -> Result<i64> {
    Ok(std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64)
}
