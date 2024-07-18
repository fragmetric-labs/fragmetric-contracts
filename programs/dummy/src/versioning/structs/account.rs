use anchor_lang::prelude::*;
use fragmetric_util::{RequireUpgradable, Upgradable};

#[account]
#[derive(InitSpace, RequireUpgradable)]
pub struct AccountData {
    #[upgradable(latest = DataV2, variant = V2)]
    pub data: VersionedData,
    pub owner: Pubkey,
    pub created_at: i64,
}

impl Upgradable for AccountData {
    type LatestVersion = DataV2;

    fn upgrade(&mut self) {
        match self.data {
            VersionedData::V1(ref old) => {
                self.data = VersionedData::V2(DataV2 {
                    field1: old.field1,
                    field2: Default::default(),
                    field3: old.field2.clone(),
                    field4: Default::default(),
                });
            }
            VersionedData::V2(_) => (),
        }
    }
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub enum VersionedData {
    V1(DataV1),
    V2(DataV2),
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct DataV1 {
    pub field1: u64,
    #[max_len(20)]
    pub field2: String,
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct DataV2 {
    pub field1: u64,
    pub field2: u32,
    #[max_len(20)]
    pub field3: String,
    pub field4: bool,
}
