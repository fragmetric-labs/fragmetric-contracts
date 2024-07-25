use anchor_lang::prelude::*;
use fragmetric_util::{RequireUpgradable, Upgradable};

#[account]
#[derive(InitSpace, RequireUpgradable)]
pub struct ReceiptTokenAuthority {
    pub authority: Pubkey,
    #[upgradable(latest = ReceiptTokenAuthorityV0, variant = V0)]
    pub data: VersionedReceiptTokenAuthority,
}

impl Upgradable for ReceiptTokenAuthority {
    type LatestVersion = ReceiptTokenAuthorityV0;

    fn upgrade(&mut self) {
        match self.data {
            VersionedReceiptTokenAuthority::V0(_) => (),
        }
    }
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub enum VersionedReceiptTokenAuthority {
    V0(ReceiptTokenAuthorityV0),
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ReceiptTokenAuthorityV0;
impl Space for ReceiptTokenAuthorityV0 {
    const INIT_SPACE: usize = 0;
}
