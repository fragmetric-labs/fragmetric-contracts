mod allocated_amount;
mod reward_pool;
mod reward_settlement;
mod user_reward_pool;
mod user_reward_settlement;

pub use allocated_amount::*;
pub use reward_pool::*;
pub use reward_settlement::*;
pub use user_reward_pool::*;
pub use user_reward_settlement::*;

pub(super) const REWARD_METADATA_NAME_MAX_LEN: usize = 16;
pub(super) const REWARD_METADATA_DESCRIPTION_MAX_LEN: usize = 128;
pub(super) const HOLDERS_MAX_LEN: usize = 8;
pub(super) const HOLDER_PUBKEYS_MAX_LEN: usize = 8;
pub(super) const TOKEN_ALLOCATED_AMOUNT_RECORD_MAX_LEN: usize = 10;
pub(super) const REWARDS_MAX_LEN: usize = 2;
pub(super) const REWARD_POOLS_MAX_LEN: usize = 2;
pub(super) const REWARD_SETTLEMENT_BLOCK_MAX_LEN: usize = 100;

#[cfg(test)]
pub mod check_account_init_space {
    use anchor_lang::Space;

    use super::*;

    pub const CHECK_REWARD_ACCOUNT_SIZE: usize = RewardAccount::INIT_SPACE; // 1624998 ~= 1.6MiB
    pub const CHECK_REWARD_POOL_SIZE: usize = RewardPool::INIT_SPACE; // 322683 ~= 320KiB
    pub const CHECK_USER_REWARD_ACCOUNT_SIZE: usize = UserRewardAccount::INIT_SPACE; // 8893 ~= 8.7KiB
    pub const CHECK_USER_REWARD_POOL_SIZE: usize = UserRewardPool::INIT_SPACE; // 1771 ~= 1.8KiB

    #[test]
    fn check_size() {
        println!("CHECK_REWARD_ACCOUNT_SIZE {}", CHECK_REWARD_ACCOUNT_SIZE);
        println!("CHECK_REWARD_POOL_SIZE {}", CHECK_REWARD_POOL_SIZE);
        println!("CHECK_USER_REWARD_ACCOUNT_SIZE {}", CHECK_USER_REWARD_ACCOUNT_SIZE);
        println!("CHECK_USER_REWARD_POOL_SIZE {}", CHECK_USER_REWARD_POOL_SIZE);
    }
}
