use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::errors::ErrorCode;

const HOLDER_NAME_MAX_LEN: usize = 14;
const HOLDER_DESCRIPTION_MAX_LEN: usize = 128;
const HOLDER_PUBKEYS_MAX_LEN_1: usize = 8;

/// Reward pool holder type.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct RewardPoolHolder {
    /// ID is determined by reward account.
    pub(super) id: u8,
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

impl RewardPoolHolder {
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
            ErrorCode::RewardExceededMaxHolderPubkeysError
        );

        self.id = id;
        self.name[..name.len()].copy_from_slice(name.as_bytes());
        self.description[..description.len()].copy_from_slice(description.as_bytes());
        self.num_pubkeys = pubkeys.len() as u8;
        self.pubkeys_1[..pubkeys.len()].copy_from_slice(pubkeys);

        Ok(())
    }

    pub(super) fn get_name(&self) -> Result<&str> {
        Ok(std::str::from_utf8(&self.name)
            .map_err(|_| crate::errors::ErrorCode::UTF8DecodingException)?
            .trim_matches('\0'))
    }

    /// How to integrate multiple fields into a single array slice or whatever...
    /// You may change the return type if needed
    #[inline(always)]
    fn get_pubkeys(&self) -> &[Pubkey] {
        &self.pubkeys_1[..self.num_pubkeys as usize]
    }

    pub(super) fn has_pubkey(&self, key: &Pubkey) -> bool {
        self.get_pubkeys().contains(key)
    }
}
