#![allow(dead_code)]
use anchor_lang::{prelude::*, Discriminator};

use super::TokenPriceCalculator;

#[derive(AnchorDeserialize)]
#[cfg_attr(test, derive(AnchorSerialize, InitSpace))]
pub struct MarinadeStakePool {
    msol_mint: Pubkey,
    admin_authority: Pubkey,
    // Target for withdrawing rent reserve SOLs. Save bot wallet account here
    operational_sol_account: Pubkey,
    // treasury - external accounts managed by marinade DAO
    // pub treasury_sol_account: Pubkey,
    treasury_msol_account: Pubkey,
    // Bump seeds:
    reserve_bump_seed: u8,
    msol_mint_authority_bump_seed: u8,
    rent_exempt_for_token_acc: u64, // Token-Account For rent exempt
    // fee applied on rewards
    reward_fee: Fee,
    stake_system: StakeSystem,
    validator_system: ValidatorSystem, //includes total_balance = total stake under management
    // sum of all the orders received in this epoch
    // must not be used for stake-unstake amount calculation
    // only for reference
    // epoch_stake_orders: u64,
    // epoch_unstake_orders: u64,
    liq_pool: LiqPool,
    available_reserve_balance: u64, // reserve_pda.lamports() - self.rent_exempt_for_token_acc. Virtual value (real may be > because of transfers into reserve). Use Update* to align
    msol_supply: u64, // Virtual value (may be < because of token burn). Use Update* to align
    // For FE. Don't use it for token amount calculation
    msol_price: u64,
    ///count tickets for delayed-unstake
    circulating_ticket_count: u64,
    ///total lamports amount of generated and not claimed yet tickets
    circulating_ticket_balance: u64,
    lent_from_reserve: u64,
    min_deposit: u64,
    min_withdraw: u64,
    staking_sol_cap: u64,
    emergency_cooling_down: u64,
    /// emergency pause
    pause_authority: Pubkey,
    paused: bool,
    // delayed unstake account fee
    // to avoid economic attacks this value should not be zero
    // (this is required because tickets are ready at the end of the epoch)
    // preferred value is one epoch rewards
    delayed_unstake_fee: FeeCents,
    // withdraw stake account fee
    // to avoid economic attacks this value should not be zero
    // (this is required because stake accounts are delivered immediately)
    // preferred value is one epoch rewards
    withdraw_stake_account_fee: FeeCents,
    withdraw_stake_account_enabled: bool,
    // Limit moving stakes from one validator to another
    // by calling redelegate, emergency_unstake and partial_unstake
    // in case of stolen validator manager key or broken delegation strategy bot
    last_stake_move_epoch: u64,     // epoch of the last stake move action
    stake_moved: u64,               // total amount of moved SOL during the epoch #stake_move_epoch
    max_stake_moved_per_epoch: Fee, // % of total_lamports_under_control
}

#[derive(AnchorDeserialize)]
#[cfg_attr(test, derive(AnchorSerialize, InitSpace))]
struct Fee {
    basis_points: u32,
}

#[derive(AnchorDeserialize)]
#[cfg_attr(test, derive(AnchorSerialize, InitSpace))]
struct FeeCents {
    bp_cents: u32,
}

#[derive(AnchorDeserialize)]
#[cfg_attr(test, derive(AnchorSerialize, InitSpace))]
struct LiqPool {
    lp_mint: Pubkey,
    lp_mint_authority_bump_seed: u8,
    sol_leg_bump_seed: u8,
    msol_leg_authority_bump_seed: u8,
    msol_leg: Pubkey,
    //The next 3 values define the SOL/mSOL Liquidity pool fee curve params
    // We assume this pool is always UNBALANCED, there should be more SOL than mSOL 99% of the time
    ///Liquidity target. If the Liquidity reach this amount, the fee reaches lp_min_discount_fee
    lp_liquidity_target: u64, // 10_000 SOL initially
    /// Liquidity pool max fee
    lp_max_fee: Fee, //3% initially
    /// SOL/mSOL Liquidity pool min fee
    lp_min_fee: Fee, //0.3% initially
    /// Treasury cut
    treasury_cut: Fee, //2500 => 25% how much of the Liquid unstake fee goes to treasury_msol_account
    lp_supply: u64, // virtual lp token supply. May be > real supply because of burning tokens. Use UpdateLiqPool to align it with real value
    lent_from_sol_leg: u64,
    liquidity_sol_cap: u64,
}

#[derive(AnchorDeserialize)]
#[cfg_attr(test, derive(AnchorSerialize, InitSpace))]
struct StakeSystem {
    stake_list: List,
    //pub last_update_epoch: u64,
    //pub updated_during_last_epoch: u32,
    delayed_unstake_cooling_down: u64,
    stake_deposit_bump_seed: u8,
    stake_withdraw_bump_seed: u8,
    /// set by admin, how much slots before the end of the epoch, stake-delta can start
    slots_for_stake_delta: u64,
    /// Marks the start of stake-delta operations, meaning that if somebody starts a delayed-unstake ticket
    /// after this var is set with epoch_num the ticket will have epoch_created = current_epoch+1
    /// (the user must wait one more epoch, because their unstake-delta will be execute in this epoch)
    last_stake_delta_epoch: u64,
    min_stake: u64, // Minimal stake account delegation
    /// can be set by validator-manager-auth to allow a second run of stake-delta to stake late stakers in the last minute of the epoch
    /// so we maximize user's rewards
    extra_stake_delta_runs: u32,
}

#[derive(AnchorDeserialize)]
#[cfg_attr(test, derive(AnchorSerialize, InitSpace))]
struct ValidatorSystem {
    validator_list: List,
    manager_authority: Pubkey,
    total_validator_score: u32,
    /// sum of all active lamports staked
    total_active_balance: u64,
    /// DEPRECATED, no longer used
    auto_add_validator_enabled: u8,
}

#[derive(AnchorDeserialize)]
#[cfg_attr(test, derive(AnchorSerialize, InitSpace))]
struct List {
    account: Pubkey,
    item_size: u32,
    count: u32,
    // Unused
    _reserved1: Pubkey,
    _reserved2: u32,
}

impl Discriminator for MarinadeStakePool {
    const DISCRIMINATOR: [u8; 8] = [216, 146, 107, 94, 104, 75, 182, 177];
}

impl Owner for MarinadeStakePool {
    fn owner() -> Pubkey {
        Self::PROGRAM_ID
    }
}

#[cfg(test)]
impl AccountSerialize for MarinadeStakePool {
    fn try_serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        if writer.write_all(&Self::DISCRIMINATOR).is_err() {
            return Err(ErrorCode::AccountDidNotSerialize.into());
        }

        if AnchorSerialize::serialize(self, writer).is_err() {
            return Err(ErrorCode::AccountDidNotSerialize.into());
        }

        Ok(())
    }
}

impl AccountDeserialize for MarinadeStakePool {
    fn try_deserialize(buf: &mut &[u8]) -> Result<Self> {
        if buf.len() < Self::DISCRIMINATOR.len() {
            return Err(ErrorCode::AccountDiscriminatorNotFound)?;
        }
        let given_disc = &buf[..8];
        if Self::DISCRIMINATOR != given_disc {
            return Err(error!(ErrorCode::AccountDiscriminatorMismatch).with_account_name("State"));
        }
        Self::try_deserialize_unchecked(buf)
    }

    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
        let mut data: &[u8] = &buf[8..];
        AnchorDeserialize::deserialize(&mut data)
            .map_err(|_| ErrorCode::AccountDidNotDeserialize.into())
    }
}

impl TokenPriceCalculator for MarinadeStakePool {
    fn calculate_token_price(&self, token_amount: u64) -> Result<u64> {
        self.msol_to_sol(token_amount)
    }
}

impl MarinadeStakePool {
    pub const PROGRAM_ID: Pubkey = pubkey!("MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD");

    fn msol_to_sol(&self, msol_amount: u64) -> Result<u64> {
        crate::utils::proportional_amount(
            msol_amount,
            self.total_value_staked_lamports(),
            self.msol_supply,
        )
        .ok_or_else(|| error!(crate::errors::ErrorCode::CalculationArithmeticException))
    }

    fn total_value_staked_lamports(&self) -> u64 {
        self.total_lamports_under_control()
            .saturating_sub(self.circulating_ticket_balance)
    }

    fn total_lamports_under_control(&self) -> u64 {
        self.validator_system.total_active_balance
            + self.total_cooling_down()
            + self.available_reserve_balance
    }

    fn total_cooling_down(&self) -> u64 {
        self.stake_system.delayed_unstake_cooling_down + self.emergency_cooling_down
    }

    #[cfg(test)]
    /// 1 Token = 1.2 SOL
    pub fn dummy_pricing_source_account_info<'a>(
        lamports: &'a mut u64,
        data: &'a mut [u8],
    ) -> AccountInfo<'a> {
        const DUMMY_PUBKEY: Pubkey = pubkey!("dummyMarinadePoo1PricingSourceAccount1nfo11");

        let mut this = Self::try_deserialize_unchecked(&mut &*data).unwrap();
        this.msol_supply = 1_000_000;
        this.available_reserve_balance = 1_200_000;
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
