use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::errors::ErrorCode;

const HOLDER_NAME_MAX_LEN: usize = 14;
const HOLDER_DESCRIPTION_MAX_LEN: usize = 128;
const HOLDER_PUBKEYS_MAX_LEN_1: usize = 8;
// const HOLDER_PUBKEYS_MAX_LEN_2: usize = 4;

/// Token holder type.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Holder {
    /// ID is determined by reward account.
    id: u8,
    name: [u8; HOLDER_NAME_MAX_LEN],
    description: [u8; HOLDER_DESCRIPTION_MAX_LEN],

    num_pubkeys: u8,

    _reserved: [u64; 32],

    /// List of allowed pubkeys for this holder.
    pubkeys_1: [Pubkey; HOLDER_PUBKEYS_MAX_LEN_1],
}

// When you want to extend pubkeys array at update v2...
// ```
// pub struct HolderExtV2 {
//     id: u8,
//     num_pubkeys: u8,
//     _padding: [u8; 14],
//     pubkeys_2: [Pubkey; HOLDER_PUBKEYS_MAX_LEN_2],
// }
// ```
// And add new field holders_1_ext_v2: [HolderExtV2; REWARD_ACCOUNT_HOLDERS_MAX_LEN_1] to reward account.

impl Holder {
    pub(super) fn initialize(
        &mut self,
        id: u8,
        name: String,
        description: String,
        pubkeys: &[Pubkey],
    ) -> Result<()> {
        require_gte!(
            HOLDER_NAME_MAX_LEN,
            name.len(),
            ErrorCode::RewardInvalidMetadataNameLengthError
        );
        require_gte!(
            HOLDER_DESCRIPTION_MAX_LEN,
            description.len(),
            ErrorCode::RewardInvalidMetadataDescriptionLengthError
        );
        require_gte!(
            HOLDER_PUBKEYS_MAX_LEN_1,
            pubkeys.len(),
            ErrorCode::RewardExceededMaxHolderPubkeysException
        );

        self.id = id;
        self.name[..name.len()].copy_from_slice(name.as_bytes());
        self.description[..description.len()].copy_from_slice(description.as_bytes());
        self.num_pubkeys = pubkeys.len() as u8;
        self.pubkeys_1[..pubkeys.len()].copy_from_slice(pubkeys);

        Ok(())
    }

    pub(super) fn get_id(&self) -> u8 {
        self.id
    }

    pub(super) fn get_name(&self) -> Result<&str> {
        Ok(std::str::from_utf8(&self.name)
            .map_err(|_| crate::errors::ErrorCode::DecodeInvalidUtf8FormatException)?
            .trim_matches('\0'))
    }

    /// How to integrate multiple fields into a single array slice or whatever...
    /// You may change the return type if needed
    fn get_pubkeys(&self) -> &[Pubkey] {
        &self.pubkeys_1[..self.num_pubkeys as usize]
    }

    pub(super) fn has_pubkey(&self, key: &Pubkey) -> bool {
        self.get_pubkeys().contains(key)
    }
}

const REWARD_NAME_MAX_LEN: usize = 14;
const REWARD_DESCRIPTION_MAX_LEN: usize = 128;

/// Reward type.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Reward {
    /// ID is determined by reward account.
    id: u16,
    name: [u8; REWARD_NAME_MAX_LEN],
    description: [u8; REWARD_DESCRIPTION_MAX_LEN],

    // RewardType as u8 representation
    reward_type_discriminant: u8,
    token_mint: Pubkey,
    token_program: Pubkey,
    decimals: u8,
    _padding: [u8; 14],

    _reserved: [u64; 16],
}

impl Reward {
    pub(super) fn initialize(
        &mut self,
        id: u16,
        name: String,
        description: String,
        reward_type: RewardType,
    ) -> Result<()> {
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

        self.id = id;
        self.name[..name.len()].copy_from_slice(name.as_bytes());
        self.description[..description.len()].copy_from_slice(description.as_bytes());
        // RewardType
        self.reward_type_discriminant = reward_type.get_discriminant();
        self.token_mint = reward_type.get_token_mint().unwrap_or_default();
        self.token_program = reward_type.get_token_program().unwrap_or_default();
        self.decimals = reward_type.get_decimals().unwrap_or_default();

        Ok(())
    }

    pub(super) fn get_name(&self) -> Result<&str> {
        Ok(std::str::from_utf8(&self.name)
            .map_err(|_| crate::errors::ErrorCode::DecodeInvalidUtf8FormatException)?
            .trim_matches('\0'))
    }

    // fn reward_type(&self) -> Result<RewardType> {
    //     let reward_type = match self.reward_type_discriminant {
    //         // Point
    //         0 => RewardType::Point {
    //             decimals: self.decimals,
    //         },
    //         // Token
    //         1 => RewardType::Token {
    //             mint: self.token_mint,
    //             program: self.token_program,
    //             decimals: self.decimals,
    //         },
    //         // SOL
    //         2 => RewardType::SOL,
    //         // Unknown
    //         _ => {
    //             return Err(ErrorCode::RewardInvalidRewardType)?;
    //         }
    //     };

    //     Ok(reward_type)
    // }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
#[non_exhaustive]
pub enum RewardType {
    Point {
        decimals: u8,
    },
    Token {
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
    },
    #[allow(clippy::upper_case_acronyms)]
    SOL,
}

impl RewardType {
    fn get_discriminant(&self) -> u8 {
        match self {
            RewardType::Point { .. } => 0,
            RewardType::Token { .. } => 1,
            RewardType::SOL => 2,
        }
    }

    fn get_token_mint(&self) -> Option<Pubkey> {
        match self {
            Self::Token { mint, .. } => Some(*mint),
            Self::Point { .. } | Self::SOL => None,
        }
    }

    fn get_token_program(&self) -> Option<Pubkey> {
        match self {
            Self::Token { program, .. } => Some(*program),
            Self::Point { .. } | Self::SOL => None,
        }
    }

    fn get_decimals(&self) -> Option<u8> {
        match self {
            Self::Point { decimals } | Self::Token { decimals, .. } => Some(*decimals),
            Self::SOL => None,
        }
    }
}
