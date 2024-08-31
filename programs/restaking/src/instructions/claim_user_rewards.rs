use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RewardClaimUserRewards {}

impl RewardClaimUserRewards {
    #[allow(unused_variables)]
    pub fn claim_user_rewards(ctx: Context<Self>, reward_pool_id: u8, reward_id: u8) -> Result<()> {
        unimplemented!()
    }
}
