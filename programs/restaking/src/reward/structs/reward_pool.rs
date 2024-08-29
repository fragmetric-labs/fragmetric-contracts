use anchor_lang::prelude::*;

use crate::{error::ErrorCode, reward::*};

#[account]
#[derive(InitSpace)]
pub struct RewardAccount {
    pub data_version: u8,
    #[max_len(10)]
    pub holders: Vec<Holder>,
    #[max_len(20)]
    pub rewards: Vec<Reward>,
    #[max_len(5)]
    pub reward_pools: Vec<RewardPool>,
}

impl RewardAccount {
    pub fn reward_pool_mut(&mut self, id: u8) -> Result<&mut RewardPool> {
        self.reward_pools
            .get_mut(id as usize)
            .ok_or_else(|| error!(ErrorCode::RewardPoolNotFound))
    }
}

/// Token holder type.
#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Holder {
    /// ID is determined when added to reward account.
    /// At first its value is zero.
    pub id: u8,
    #[max_len(16)]
    pub name: String,
    #[max_len(128)]
    pub description: String,
    /// List of allowed pubkeys for this holder.
    #[max_len(10)]
    pub pubkeys: Vec<Pubkey>,
}

impl Holder {
    pub fn new(name: String, description: String, pubkeys: Vec<Pubkey>) -> Self {
        Self {
            id: 0,
            pubkeys,
            name,
            description,
        }
    }
}

/// Reward type.
#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Reward {
    /// ID is determined when added to reward account.
    /// At first its value is zero.
    pub id: u8,
    pub reward_type: RewardType,
    #[max_len(16)]
    pub name: String,
    #[max_len(128)]
    pub description: String,
}

impl Reward {
    pub fn new(name: String, description: String, reward_type: RewardType) -> Self {
        Self {
            id: 0,
            reward_type,
            name,
            description,
        }
    }
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum RewardType {
    Point,
    Token { mint: Pubkey },
}

impl RewardType {
    pub fn new(reward_type: String, reward_token_mint: Option<Pubkey>) -> Result<Self> {
        match (reward_type.to_lowercase().as_str(), reward_token_mint) {
            ("point", _) => Ok(Self::Point),
            ("token", Some(mint)) => Ok(Self::Token { mint }),
            _ => err!(ErrorCode::RewardInvalidRewardType)?,
        }
    }
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RewardPool {
    /// ID is determined when added to reward account.
    /// At first its value is zero.
    pub id: u8,
    #[max_len(16)]
    pub name: String,

    /// Holder id is not provided for default holder (fragmetric)
    pub holder_id: Option<u8>,
    pub custom_contribution_accrual_rate_enabled: bool,
    pub token_mint: Pubkey,

    pub initial_slot: u64,
    pub updated_slot: u64,
    pub closed_slot: Option<u64>,

    pub contribution: u128,
    pub token_allocated_amount: TokenAllocatedAmount,
    #[max_len(20)]
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
            token_mint,
            initial_slot: current_slot,
            updated_slot: current_slot,
            closed_slot: None,
            token_allocated_amount: Default::default(),
            contribution: 0,
            reward_settlements: vec![],
        }
    }
}
