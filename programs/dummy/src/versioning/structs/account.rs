use anchor_lang::prelude::*;
use fragmetric_util::{RequireUpgradable, Upgradable};

#[account]
#[derive(InitSpace, RequireUpgradable)]
pub struct AccountData {
    #[upgradable(latest = DataV1, variant = V1)]
    pub data: VersionedData,
    pub owner: Pubkey,
    pub created_at: i64,
}

impl Upgradable for AccountData {
    type LatestVersion = DataV1;

    fn upgrade(&mut self) {
        match self.data {
            VersionedData::V1(_) => (),
        }
    }
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub enum VersionedData {
    V1(DataV1),
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct DataV1 {
    pub field1: u64,
    #[max_len(20)]
    pub field2: String,
}
