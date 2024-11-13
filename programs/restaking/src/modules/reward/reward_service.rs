use crate::events;
use crate::modules::reward::*;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

pub struct RewardService<'info, 'a>
where
    'info: 'a,
{
    receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,

    current_slot: u64,
    _current_timestamp: i64,
}

impl<'info, 'a> RewardService<'info, 'a> {
    pub fn new(
        receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
        reward_account: &'a mut AccountLoader<'info, RewardAccount>,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            receipt_token_mint,
            reward_account,
            current_slot: clock.slot,
            _current_timestamp: clock.unix_timestamp,
        })
    }

    pub fn process_update_reward_pools(&self) -> Result<()> {
        self.reward_account
            .load_mut()?
            .update_reward_pools(self.current_slot)?;

        self.emit_operator_updated_reward_pools_event()
    }

    fn emit_operator_updated_reward_pools_event(&self) -> Result<()> {
        emit!(events::OperatorUpdatedRewardPools {
            receipt_token_mint: self.receipt_token_mint.key(),
            reward_account_address: self.reward_account.key(),
        });

        Ok(())
    }

    pub fn process_update_user_reward_pools(
        &self,
        user_reward_account: &mut AccountLoader<UserRewardAccount>,
    ) -> Result<()> {
        self.reward_account
            .load_mut()?
            .update_user_reward_pools(&mut *user_reward_account.load_mut()?, self.current_slot)

        // no events required practically...
        // emit!(UserUpdatedRewardPool::new(
        //     receipt_token_mint.key(),
        //     vec![update],
        // ));
    }

    pub fn process_claim_user_rewards(
        &self,
        _user_reward_account: &mut AccountLoader<UserRewardAccount>,
    ) -> Result<()> {
        unimplemented!()
    }

    pub(in crate::modules) fn update_reward_pools_token_allocation(
        &self,
        from: Option<(Pubkey, &mut UserRewardAccount)>,
        to: Option<(Pubkey, &mut UserRewardAccount)>,
        amount: u64,
        contribution_accrual_rate: Option<u8>,
    ) -> Result<()> {
        let (from_key, from_account) = from.unzip();
        let (to_key, to_account) = to.unzip();

        self.reward_account
            .load_mut()?
            .update_reward_pools_token_allocation(
                self.receipt_token_mint.key(),
                amount,
                contribution_accrual_rate,
                from_account,
                to_account,
                self.current_slot,
            )?;

        emit!(events::UserUpdatedRewardPool {
            receipt_token_mint: self.receipt_token_mint.key(),
            updated_user_reward_account_addresses: vec![from_key, to_key]
                .into_iter()
                .flatten()
                .collect(),
        });

        Ok(())
    }
}
