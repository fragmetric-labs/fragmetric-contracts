#![allow(dead_code)]
use anchor_lang::{
    prelude::*,
    solana_program::clock::{Epoch, UnixTimestamp},
};

use super::TokenPriceCalculator;

#[repr(C)]
#[derive(AnchorDeserialize)]
#[cfg_attr(test, derive(AnchorSerialize))]
pub struct SplStakePool {
    /// Account type, must be StakePool currently
    account_type: AccountType,
    /// Manager authority, allows for updating the staker, manager, and fee
    /// account
    manager: Pubkey,
    /// Staker authority, allows for adding and removing validators, and
    /// managing stake distribution
    staker: Pubkey,
    /// Stake deposit authority
    ///
    /// If a depositor pubkey is specified on initialization, then deposits must
    /// be signed by this authority. If no deposit authority is specified,
    /// then the stake pool will default to the result of:
    /// `Pubkey::find_program_address(
    ///     &[&stake_pool_address.as_ref(), b"deposit"],
    ///     program_id,
    /// )`
    stake_deposit_authority: Pubkey,
    /// Stake withdrawal authority bump seed
    /// for `create_program_address(&[state::StakePool account, "withdrawal"])`
    stake_withdraw_bump_seed: u8,
    /// Validator stake list storage account
    validator_list: Pubkey,
    /// Reserve stake account, holds deactivated stake
    reserve_stake: Pubkey,
    /// Pool Mint
    pool_mint: Pubkey,
    /// Manager fee account
    manager_fee_account: Pubkey,
    /// Pool token program id
    token_program_id: Pubkey,
    /// Total stake under management.
    /// Note that if `last_update_epoch` does not match the current epoch then
    /// this field may not be accurate
    total_lamports: u64,
    /// Total supply of pool tokens (should always match the supply in the Pool
    /// Mint)
    pool_token_supply: u64,
    /// Last epoch the `total_lamports` field was updated
    last_update_epoch: u64,
    /// Lockup that all stakes in the pool must have
    lockup: Lockup,
    /// Fee taken as a proportion of rewards each epoch
    epoch_fee: Fee,
    /// Fee for next epoch
    next_epoch_fee: FutureEpoch<Fee>,
    /// Preferred deposit validator vote account pubkey
    preferred_deposit_validator_vote_address: Option<Pubkey>,
    /// Preferred withdraw validator vote account pubkey
    preferred_withdraw_validator_vote_address: Option<Pubkey>,
    /// Fee assessed on stake deposits
    stake_deposit_fee: Fee,
    /// Fee assessed on withdrawals
    stake_withdrawal_fee: Fee,
    /// Future stake withdrawal fee, to be set for the following epoch
    next_stake_withdrawal_fee: FutureEpoch<Fee>,
    /// Fees paid out to referrers on referred stake deposits.
    /// Expressed as a percentage (0 - 100) of deposit fees.
    /// i.e. `stake_deposit_fee`% of stake deposited is collected as deposit
    /// fees for every deposit and `stake_referral_fee`% of the collected
    /// stake deposit fees is paid out to the referrer
    stake_referral_fee: u8,
    /// Toggles whether the `DepositSol` instruction requires a signature from
    /// this `sol_deposit_authority`
    sol_deposit_authority: Option<Pubkey>,
    /// Fee assessed on SOL deposits
    sol_deposit_fee: Fee,
    /// Fees paid out to referrers on referred SOL deposits.
    /// Expressed as a percentage (0 - 100) of SOL deposit fees.
    /// i.e. `sol_deposit_fee`% of SOL deposited is collected as deposit fees
    /// for every deposit and `sol_referral_fee`% of the collected SOL
    /// deposit fees is paid out to the referrer
    sol_referral_fee: u8,
    /// Toggles whether the `WithdrawSol` instruction requires a signature from
    /// the `deposit_authority`
    sol_withdraw_authority: Option<Pubkey>,
    /// Fee assessed on SOL withdrawals
    sol_withdrawal_fee: Fee,
    /// Future SOL withdrawal fee, to be set for the following epoch
    next_sol_withdrawal_fee: FutureEpoch<Fee>,
    /// Last epoch's total pool tokens, used only for APR estimation
    last_epoch_pool_token_supply: u64,
    /// Last epoch's total lamports, used only for APR estimation
    last_epoch_total_lamports: u64,
}

/// Enum representing the account type managed by the program
#[derive(AnchorDeserialize)]
#[cfg_attr(test, derive(AnchorSerialize))]
enum AccountType {
    /// If the account has not been initialized, the enum will be 0
    Uninitialized,
    /// Stake pool
    StakePool,
    /// Validator stake list
    ValidatorList,
}

#[derive(AnchorDeserialize)]
#[cfg_attr(test, derive(AnchorSerialize))]
struct Lockup {
    /// UnixTimestamp at which this stake will allow withdrawal, unless the
    ///   transaction is signed by the custodian
    unix_timestamp: UnixTimestamp,
    /// epoch height at which this stake will allow withdrawal, unless the
    ///   transaction is signed by the custodian
    epoch: Epoch,
    /// custodian signature on a transaction exempts the operation from
    ///  lockup constraints
    custodian: Pubkey,
}

#[repr(C)]
#[derive(AnchorDeserialize)]
#[cfg_attr(test, derive(AnchorSerialize))]
struct Fee {
    /// denominator of the fee ratio
    denominator: u64,
    /// numerator of the fee ratio
    numerator: u64,
}

#[repr(C)]
#[derive(AnchorDeserialize)]
#[cfg_attr(test, derive(AnchorSerialize))]
enum FutureEpoch<T> {
    /// Nothing is set
    None,
    /// Value is ready after the next epoch boundary
    One(T),
    /// Value is ready after two epoch boundaries
    Two(T),
}

impl Owner for SplStakePool {
    fn owner() -> Pubkey {
        Self::PROGRAM_ID
    }
}

#[cfg(test)]
impl AccountSerialize for SplStakePool {
    fn try_serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        if AnchorSerialize::serialize(self, writer).is_err() {
            return Err(ErrorCode::AccountDidNotSerialize.into());
        }

        Ok(())
    }
}

impl AccountDeserialize for SplStakePool {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
        AnchorDeserialize::deserialize(buf).map_err(|_| ErrorCode::AccountDidNotDeserialize.into())
    }
}

impl TokenPriceCalculator for SplStakePool {
    fn calculate_token_price(&self, token_amount: u64) -> Result<u64> {
        self.calculate_lamports_from_pool_tokens(token_amount)
    }
}

impl SplStakePool {
    pub const PROGRAM_ID: Pubkey = pubkey!("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy");

    fn calculate_lamports_from_pool_tokens(&self, pool_tokens: u64) -> Result<u64> {
        crate::utils::proportional_amount(pool_tokens, self.total_lamports, self.pool_token_supply)
            .ok_or_else(|| error!(crate::errors::ErrorCode::CalculationArithmeticException))
    }

    #[cfg(test)]
    /// 1 Token = 1.4 SOL
    pub fn dummy_pricing_source_account_info<'a>(
        lamports: &'a mut u64,
        data: &'a mut [u8],
    ) -> AccountInfo<'a> {
        const DUMMY_PUBKEY: Pubkey = pubkey!("dummySp1StakePoo1PricingSourceAccount1nfo11");

        let mut this = Self::try_deserialize_unchecked(&mut &*data).unwrap();
        this.pool_token_supply = 1_000_000;
        this.total_lamports = 1_400_000;
        let mut writer = unsafe { &mut *(data as *mut [u8]) };
        this.try_serialize(&mut writer).unwrap();

        AccountInfo::new(
            &DUMMY_PUBKEY,
            false,
            false,
            lamports,
            data,
            &Self::PROGRAM_ID,
            false,
            0,
        )
    }
}
