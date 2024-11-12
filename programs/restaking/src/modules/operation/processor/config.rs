use anchor_lang::prelude::{msg, AccountInfo, ProgramError, Pubkey};
use anchor_lang::solana_program::clock::DEFAULT_SLOTS_PER_EPOCH;
use anchor_spl::token_2022::spl_token_2022::solana_zk_token_sdk::curve25519::scalar::Zeroable;
use anchor_spl::token_2022::spl_token_2022::solana_zk_token_sdk::instruction::Pod;
use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_bytemuck::types::{PodU16, PodU64};
use shank::ShankAccount;
use jito_vault_core::MAX_BPS;
use jito_vault_sdk::error::VaultError;

pub enum VaultDiscriminator {
    Config = 1,
    Vault = 2,
    VaultNcnTicket = 3,
    VaultOperatorDelegation = 4,
    VaultNcnSlasherTicket = 5,
    VaultNcnSlasherOperatorTicket = 6,
    VaultStakerWithdrawalTicket = 7,
    VaultUpdateStateTracker = 8,
}

impl Discriminator for Config {
    const DISCRIMINATOR: u8 = VaultDiscriminator::Config as u8;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct Config {
    /// The configuration admin
    pub admin: Pubkey,

    /// The approved restaking program for this vault
    pub restaking_program: Pubkey,

    /// The length of an epoch in slots
    pub epoch_length: PodU64,

    /// The number of vaults managed by the program
    num_vaults: PodU64,

    /// The fee cap in basis points ( withdraw and deposit )
    deposit_withdrawal_fee_cap_bps: PodU16,

    /// The maximum amount a fee can increase per epoch in basis points
    fee_rate_of_change_bps: PodU16,

    /// The amount a fee can increase above the rate of change in basis points
    fee_bump_bps: PodU16,

    /// The program fee in basis points
    program_fee_bps: PodU16,

    /// The fee wallet
    pub program_fee_wallet: Pubkey,

    /// The admin for the fee account
    pub fee_admin: Pubkey,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 229],
}

impl Config {
    /// Maximum fee cap in basis points
    pub const DEFAULT_FEES_CAP_BPS: u16 = 2_000; // 20%
    /// Maximum rate of change in fee basis pointer per epoch
    pub const DEFAULT_FEE_RATE_OF_CHANGE_BPS: u16 = 2_500; // 25%
    /// Maximum bump in fee change above the rate of change
    pub const DEFAULT_FEE_BUMP_BPS: u16 = 10; // 0.1%

    pub fn new(
        admin: Pubkey,
        restaking_program: Pubkey,
        program_fee_wallet: Pubkey,
        program_fee_bps: u16,
        bump: u8,
    ) -> Self {
        Self {
            admin,
            restaking_program,
            epoch_length: PodU64::from(DEFAULT_SLOTS_PER_EPOCH),
            num_vaults: PodU64::from(0),
            // Cannot be higher than 100%
            deposit_withdrawal_fee_cap_bps: PodU16::from(Self::DEFAULT_FEES_CAP_BPS),
            fee_rate_of_change_bps: PodU16::from(Self::DEFAULT_FEE_RATE_OF_CHANGE_BPS),
            fee_bump_bps: PodU16::from(Self::DEFAULT_FEE_BUMP_BPS),
            program_fee_bps: PodU16::from(program_fee_bps),
            program_fee_wallet,
            fee_admin: admin,
            bump,
            reserved: [0; 229],
        }
    }

    pub fn set_epoch_length(&mut self, value: u64) {
        self.epoch_length = PodU64::from(value);
    }

    pub fn get_epoch_from_slot(&self, slot: u64) -> Result<u64, VaultError> {
        slot.checked_div(self.epoch_length())
            .ok_or(VaultError::InvalidEpochLength)
    }

    pub fn epoch_length(&self) -> u64 {
        self.epoch_length.into()
    }

    pub fn num_vaults(&self) -> u64 {
        self.num_vaults.into()
    }

    pub fn deposit_withdrawal_fee_cap_bps(&self) -> u16 {
        u16::from(self.deposit_withdrawal_fee_cap_bps)
    }

    pub fn fee_rate_of_change_bps(&self) -> u16 {
        u16::from(self.fee_rate_of_change_bps)
    }

    pub fn program_fee_bps(&self) -> u16 {
        self.program_fee_bps.into()
    }

    pub fn set_program_fee_bps(&mut self, new_fee_bps: u16) -> Result<(), ProgramError> {
        if new_fee_bps > MAX_BPS {
            msg!("New fee exceeds maximum allowed fee");
            return Err(ProgramError::InvalidInstructionData);
        }
        self.program_fee_bps = PodU16::from(new_fee_bps);
        Ok(())
    }

    pub fn fee_bump_bps(&self) -> u16 {
        u16::from(self.fee_bump_bps)
    }

    pub fn set_deposit_withdrawal_fee_cap_bps(
        &mut self,
        fee_cap_bps: u16,
    ) -> Result<(), VaultError> {
        if fee_cap_bps > MAX_BPS {
            return Err(VaultError::VaultFeeCapExceeded);
        }
        self.deposit_withdrawal_fee_cap_bps = PodU16::from(fee_cap_bps);
        Ok(())
    }

    pub fn set_fee_rate_of_change_bps(
        &mut self,
        fee_rate_of_change_bps: u16,
    ) -> Result<(), VaultError> {
        if fee_rate_of_change_bps > MAX_BPS {
            return Err(VaultError::VaultFeeCapExceeded);
        }
        self.fee_rate_of_change_bps = PodU16::from(fee_rate_of_change_bps);
        Ok(())
    }

    pub fn set_fee_bump_bps(&mut self, fee_bump_bps: u16) -> Result<(), VaultError> {
        if fee_bump_bps > MAX_BPS {
            return Err(VaultError::VaultFeeCapExceeded);
        }
        self.fee_bump_bps = PodU16::from(fee_bump_bps);
        Ok(())
    }

    pub fn increment_num_vaults(&mut self) -> Result<(), VaultError> {
        let mut num_vaults: u64 = self.num_vaults.into();
        num_vaults = num_vaults.checked_add(1).ok_or(VaultError::VaultOverflow)?;
        self.num_vaults = PodU64::from(num_vaults);
        Ok(())
    }

    /// Calculate the amount of tokens collected as a program fee for withdrawing tokens from the vault.
    pub fn calculate_program_fee(program_fee_bps: u16, vrt_amount: u64) -> Result<u64, VaultError> {
        let fee = (vrt_amount as u128)
            .checked_mul(program_fee_bps as u128)
            .map(|x| x.div_ceil(MAX_BPS as u128))
            .and_then(|x| x.try_into().ok())
            .ok_or(VaultError::VaultOverflow)?;
        Ok(fee)
    }

    pub fn seeds() -> Vec<Vec<u8>> {
        vec![b"config".to_vec()]
    }

    pub fn find_program_address(program_id: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds();
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    /// Loads the vault [`jito_vault_core::config::Config`] account
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `account` - The account to load
    /// * `expect_writable` - Whether the account should be writable
    ///
    /// # Returns
    /// * `Result<(), ProgramError>` - The result of the operation
    pub fn load(
        program_id: &Pubkey,
        account: &AccountInfo,
        expect_writable: bool,
    ) -> Result<(), ProgramError> {
        if account.owner.ne(program_id) {
            msg!("Config account has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if account.data_is_empty() {
            msg!("Config account data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !account.is_writable {
            msg!("Config account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if account.data.borrow()[0].ne(&Self::DISCRIMINATOR) {
            msg!("Config account discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        if account.key.ne(&Self::find_program_address(program_id).0) {
            msg!("Config account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    pub fn set_admin(&mut self, new_admin: Pubkey) {
        self.admin = new_admin;
        self.fee_admin = new_admin;
    }
}
