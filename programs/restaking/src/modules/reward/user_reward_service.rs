use crate::modules::reward::*;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

pub struct UserRewardService<'info: 'a, 'a> {
    _receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,
    user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,

    current_slot: u64,
    _current_timestamp: i64,
}

impl<'info, 'a> UserRewardService<'info, 'a> {
    pub fn new(
        _receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
        reward_account: &'a mut AccountLoader<'info, RewardAccount>,
        user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            _receipt_token_mint,
            reward_account,
            user_reward_account,
            current_slot: clock.slot,
            _current_timestamp: clock.unix_timestamp,
        })
    }

    pub fn process_update_user_reward_pools(&self) -> Result<()> {
        self.reward_account.load_mut()?.update_user_reward_pools(
            &mut *self.user_reward_account.load_mut()?,
            self.current_slot,
        )

        // no events required practically...
        // emit!(UserUpdatedRewardPool::new(
        //     receipt_token_mint.key(),
        //     vec![update],
        // ));
    }

    pub fn process_claim_user_rewards(&self) -> Result<()> {
        unimplemented!()
    }
}
