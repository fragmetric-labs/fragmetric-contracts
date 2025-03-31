use anchor_lang::prelude::*;
use bytemuck::Zeroable;

use crate::errors::ErrorCode;

const REWARD_NAME_MAX_LEN: usize = 14;
const REWARD_DESCRIPTION_MAX_LEN: usize = 128;

/// Reward type.
#[zero_copy]
#[repr(C)]
pub(super) struct Reward {
    /// ID is determined by reward account.
    pub id: u16,
    name: [u8; REWARD_NAME_MAX_LEN],
    description: [u8; REWARD_DESCRIPTION_MAX_LEN],

    pub claimable: u8,
    pub mint: Pubkey,
    pub program: Pubkey,
    pub decimals: u8,

    _reserved: [u8; 142],
}

impl Reward {
    pub fn initialize(
        &mut self,
        id: u16,
        name: impl AsRef<str>,
        description: impl AsRef<str>,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        claimable: bool,
    ) -> anchor_lang::Result<()> {
        let name = name.as_ref().trim_matches('\0');
        let description = description.as_ref().trim_matches('\0');

        require_gte!(
            REWARD_NAME_MAX_LEN,
            name.len(),
            ErrorCode::RewardInvalidMetadataNameLengthError
        );
        require_gte!(
            REWARD_DESCRIPTION_MAX_LEN,
            description.len(),
            ErrorCode::RewardInvalidMetadataDescriptionLengthError
        );

        *self = Zeroable::zeroed();

        self.id = id;
        self.name[..name.len()].copy_from_slice(name.as_bytes());
        self.description[..description.len()].copy_from_slice(description.as_bytes());
        self.claimable = claimable as u8;
        self.mint = mint;
        self.program = program;
        self.decimals = decimals;

        Ok(())
    }

    pub fn get_name(&self) -> Result<&str> {
        Ok(std::str::from_utf8(&self.name)
            .map_err(|_| ErrorCode::UTF8DecodingException)?
            .trim_matches('\0'))
    }

    pub fn set_claimable(&mut self, claimable: bool) -> &mut Self {
        self.claimable = claimable as u8;
        self
    }

    /// Reward token can be changed only if unclaimable
    pub fn set_reward_token(&mut self, mint: Pubkey, program: Pubkey, decimals: u8) -> &mut Self {
        self.mint = mint;
        self.program = program;
        self.decimals = decimals;
        self
    }
}
