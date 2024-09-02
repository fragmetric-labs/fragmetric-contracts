use anchor_lang::prelude::*;

use super::*;

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
    pub fn new(name: String, description: String, pubkeys: Vec<Pubkey>) -> Result<Self> {
        require_gte!(16, name.len());
        require_gte!(128, description.len());
        require_gte!(20, pubkeys.len());

        Ok(Self {
            id: 0,
            pubkeys,
            name,
            description,
            _reserved: [0; 256],
        })
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
    pub fn new(name: String, description: String, reward_type: RewardType) -> Result<Self> {
        require_gte!(16, name.len());
        require_gte!(128, description.len());

        Ok(Self {
            id: 0,
            reward_type,
            name,
            description,
            _reserved: [0; 128],
        })
    }
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum RewardType {
    Point { decimals: u8 },
    Token { mint: Pubkey, program: Pubkey, decimals: u8 },
    SOL,
}