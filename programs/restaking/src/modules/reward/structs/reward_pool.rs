use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::reward::*;

/// Token holder type.
#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Holder {
    /// ID is determined when added to reward account.
    /// At first its value is zero.
    pub id: u8,
    #[max_len(REWARD_METADATA_NAME_MAX_LEN)]
    pub name: String,
    #[max_len(REWARD_METADATA_DESCRIPTION_MAX_LEN)]
    pub description: String,
    /// List of allowed pubkeys for this holder.
    #[max_len(HOLDER_PUBKEYS_MAX_LEN)]
    pub pubkeys: Vec<Pubkey>,
    pub _reserved: [u8; 256],
}

impl Holder {
    pub fn new(name: String, description: String, pubkeys: Vec<Pubkey>) -> Self {
        Self {
            id: 0,
            pubkeys,
            name,
            description,
            _reserved: [0; 256],
        }
    }
}

/// Reward type.
#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Reward {
    /// ID is determined when added to reward account.
    /// At first its value is zero.
    pub id: u8,
    #[max_len(REWARD_METADATA_NAME_MAX_LEN)]
    pub name: String,
    #[max_len(REWARD_METADATA_DESCRIPTION_MAX_LEN)]
    pub description: String,
    pub reward_type: RewardType,
    pub _reserved: [u8; 128],
}

impl Reward {
    pub fn new(name: String, description: String, reward_type: RewardType) -> Self {
        Self {
            id: 0,
            reward_type,
            name,
            description,
            _reserved: [0; 128],
        }
    }
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum RewardType {
    Point { decimals: u8 },
    Token { mint: Pubkey, decimals: u8 },
    SOL,
}

#[account]
#[derive(InitSpace)]
pub struct RewardAccount {
    pub data_version: u8,
    #[max_len(HOLDERS_MAX_LEN)]
    pub holders: Vec<Holder>,
    #[max_len(REWARDS_MAX_LEN)]
    pub rewards: Vec<Reward>,
    #[max_len(REWARD_POOLS_MAX_LEN)]
    pub reward_pools: Vec<RewardPool>,
}

impl RewardAccount {
    pub fn reward_pool_mut(&mut self, id: u8) -> Result<&mut RewardPool> {
        self.reward_pools
            .get_mut(id as usize)
            .ok_or_else(|| error!(ErrorCode::RewardPoolNotFound))
    }
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RewardPool {
    /// ID is determined when added to reward account.
    /// At first its value is zero.
    pub id: u8,
    #[max_len(REWARD_METADATA_NAME_MAX_LEN)]
    pub name: String,

    /// Holder id is not provided for default holder (fragmetric)
    pub holder_id: Option<u8>,
    pub custom_contribution_accrual_rate_enabled: bool,
    pub receipt_token_mint: Pubkey,

    pub initial_slot: u64,
    pub updated_slot: u64,
    pub closed_slot: Option<u64>,

    pub contribution: u128,
    pub token_allocated_amount: TokenAllocatedAmount,
    pub _reserved: [u8; 256],
    #[max_len(REWARDS_MAX_LEN)]
    pub reward_settlements: Vec<RewardSettlement>,
}

impl RewardPool {
    pub fn new(
        name: String,
        holder_id: Option<u8>,
        custom_contribution_accrual_rate_enabled: bool,
        token_mint: Pubkey,
        current_slot: u64,
    ) -> Self {
        Self {
            id: 0,
            name,
            holder_id,
            custom_contribution_accrual_rate_enabled,
            receipt_token_mint: token_mint,
            initial_slot: current_slot,
            updated_slot: current_slot,
            closed_slot: None,
            token_allocated_amount: Default::default(),
            contribution: 0,
            _reserved: [0; 256],
            reward_settlements: vec![],
        }
    }
}
