use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::common::PDASignerSeeds;
use crate::modules::reward::*;

#[account]
#[derive(InitSpace)]
pub struct RewardAccount {
    pub data_version: u8,
    pub bump: u8,
    pub receipt_token_mint: Pubkey,
    pub account_size: u32,

    #[max_len(HOLDERS_MAX_LEN)]
    pub holders: Vec<Holder>,

    #[max_len(REWARDS_INIT_LEN)]
    pub rewards: Vec<Reward>,

    #[max_len(REWARD_POOLS_INIT_LEN)]
    pub reward_pools: Vec<RewardPool>,
}

impl PDASignerSeeds<3> for RewardAccount {
    const SEED: &'static [u8] = b"reward";

    fn signer_seeds(&self) -> [&[u8]; 3] {
        [
            Self::SEED,
            self.receipt_token_mint.as_ref(),
            self.bump_as_slice(),
        ]
    }

    fn bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl RewardAccount {
    pub fn initialize_if_needed(&mut self, bump: u8, receipt_token_mint: Pubkey, account_size: u32) {
        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
        }
        self.account_size = account_size;
    }

    pub fn reward_pool_mut(&mut self, id: u8) -> Result<&mut RewardPool> {
        self.reward_pools
            .get_mut(id as usize)
            .ok_or_else(|| error!(ErrorCode::RewardPoolNotFoundError))
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

    pub initial_slot: u64,
    pub updated_slot: u64,
    pub closed_slot: Option<u64>,

    pub contribution: u128,
    pub token_allocated_amount: TokenAllocatedAmount,
    pub _reserved: [u8; 256],

    #[max_len(REWARDS_INIT_LEN)]
    pub reward_settlements: Vec<RewardSettlement>,
}

impl RewardPool {
    pub fn new(
        name: String,
        holder_id: Option<u8>,
        custom_contribution_accrual_rate_enabled: bool,
        current_slot: u64,
    ) -> Result<Self> {
        require_gte!(16, name.len());

        Ok(Self {
            id: 0,
            name,
            holder_id,
            custom_contribution_accrual_rate_enabled,
            initial_slot: current_slot,
            updated_slot: current_slot,
            closed_slot: None,
            token_allocated_amount: Default::default(),
            contribution: 0,
            _reserved: [0; 256],
            reward_settlements: vec![],
        })
    }
}
