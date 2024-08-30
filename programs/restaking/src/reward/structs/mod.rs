mod allocated_amount;
mod reward_pool;
mod settlement;
mod user_pool;

pub use allocated_amount::*;
pub use reward_pool::*;
pub use settlement::*;
pub use user_pool::*;

const NAME_MAX_LEN: usize = 16;
const DESCRIPTION_MAX_LEN: usize = 128;
const HOLDERS_MAX_LEN: usize = 10;
const HOLDER_PUBKEYS_MAX_LEN: usize = 8;
const REWARDS_MAX_LEN: usize = 20;
const REWARD_POOLS_MAX_LEN: usize = 10;
const REWARD_SETTLEMENT_BLOCK_MAX_LEN: usize = 100;
const TOKEN_ALLOCATED_AMOUNT_RECORD_MAX_LEN: usize = 10;

#[cfg(test)]
pub mod check_account_init_space {
    use anchor_lang::Space;

    use super::*;

    pub const CHECK_REWARD_ACCOUNT_SIZE: usize = RewardAccount::INIT_SPACE; // 1624998 ~= 1.6MiB
    pub const CHECK_REWARD_POOL_SIZE: usize = RewardPool::INIT_SPACE; // 322683 ~= 320KiB
    pub const CHECK_USER_REWARD_ACCOUNT_SIZE: usize = UserRewardAccount::INIT_SPACE; // 8893 ~= 8.7KiB
    pub const CHECK_USER_REWARD_POOL_SIZE: usize = UserRewardPool::INIT_SPACE; // 1771 ~= 1.8KiB
}
