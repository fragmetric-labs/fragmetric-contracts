#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

use crate::errors::ErrorCode;

pub trait PDASeeds<const N: usize> {
    const SEED: &'static [u8];

    fn seeds(&self) -> [&[u8]; N];
    fn bump_ref(&self) -> &u8;

    fn signer_seeds(&self) -> Vec<&[u8]> {
        let mut signer_seeds = self.seeds().to_vec();
        signer_seeds.push(std::slice::from_ref(self.bump_ref()));
        signer_seeds
    }
}

/// drops sub-decimal values.
/// when both numerator and denominator are zero, returns amount.
pub fn proportional_amount(amount: u64, numerator: u64, denominator: u64) -> Option<u64> {
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
pub fn timestamp_now() -> Result<i64> {
    Ok(Clock::get()?.unix_timestamp)
}

#[cfg(not(target_os = "solana"))]
pub fn timestamp_now() -> Result<i64> {
    Ok(std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64)
}

/// Truncates null (0x0000) at the end.
pub fn from_utf8_trim_null(v: &[u8]) -> Result<String> {
    Ok(std::str::from_utf8(v)
        .map_err(|_| ErrorCode::DecodeInvalidUtf8FormatException)?
        .replace('\0', ""))
}
