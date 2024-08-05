use anchor_lang::prelude::*;
use fragmetric_util::{RequireUpgradable, Upgradable};

#[account]
#[derive(InitSpace, RequireUpgradable)]
pub struct Fund {
    pub admin: Pubkey,
    pub receipt_token_mint: Pubkey,
    // pub receipt_token_lock_account: Pubkey,
    #[upgradable(latest = FundV1, variant = V1)]
    pub data: VersionedFund,
}

impl Upgradable for Fund {
    type LatestVersion = FundV1;

    fn upgrade(&mut self) {
        match self.data {
            VersionedFund::V1(_) => (),
        }
    }
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub enum VersionedFund {
    V1(FundV1),
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FundV1 {
    pub sol_withdrawal_fee_rate: u16, // 2
    #[max_len(20)]
    pub whitelisted_tokens: Vec<TokenInfo>,
    pub sol_amount_in: u128, // 16
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct TokenInfo {
    pub address: Pubkey,
    pub token_cap: u128,
    pub token_amount_in: u128,
}

impl TokenInfo {
    pub fn empty(address: Pubkey, token_cap: u128) -> Self {
        Self {
            address,
            token_cap,
            token_amount_in: 0,
        }
    }
}
