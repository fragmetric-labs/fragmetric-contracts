use anchor_lang::prelude::*;
use fragmetric_util::{RequireUpgradable, Upgradable};

#[account]
#[derive(InitSpace, RequireUpgradable)]
pub struct UserAccount {
    #[upgradable(latest = UserAccountV1, variant = V1)]
    pub data: VersionedUserAccount,
}

impl Upgradable for UserAccount {
    type LatestVersion = UserAccountV1;

    fn upgrade(&mut self) {
        match self.data {
            VersionedUserAccount::V1(_) => (),
        }
    }
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub enum VersionedUserAccount {
    V1(UserAccountV1),
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UserAccountV1 {
    #[max_len(32)]
    pub withdrawal_requests: Vec<WithdrawalRequest>,
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WithdrawalRequest {
    pub batch_id: u64,
    pub request_id: u64,
    pub receipt_token_amount: u64,
    pub timestamp: i64,
}

impl WithdrawalRequest {
    pub fn new(batch_id: u64, request_id: u64) -> Result<Self> {
        Ok(Self {
            batch_id,
            request_id,
            receipt_token_amount: 0,
            timestamp: Clock::get()?.unix_timestamp,
        })
    }
}
