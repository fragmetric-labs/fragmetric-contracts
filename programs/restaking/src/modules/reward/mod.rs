mod reward;
mod reward_account;
mod reward_configuration_service;
mod reward_pool;
mod reward_service;
mod reward_settlement;
mod token_allocated_amount;
mod user_reward_account;
mod user_reward_configuration_service;
mod user_reward_pool;
mod user_reward_service;
mod user_reward_settlement;

pub use reward::*;
pub use reward_account::*;
pub use reward_configuration_service::*;
pub use reward_pool::*;
pub use reward_service::*;
pub use reward_settlement::*;
pub use token_allocated_amount::*;
pub use user_reward_account::*;
pub use user_reward_configuration_service::*;
pub use user_reward_pool::*;
pub use user_reward_service::*;
pub use user_reward_settlement::*;

#[cfg(test)]
pub mod check_account_init_space {
    use super::*;
    use crate::modules::reward::reward_pool::RewardPool;

    pub const CHECK_REWARD_ACCOUNT_SIZE: usize = 8 + std::mem::size_of::<RewardAccount>(); // 342064 ~= 335KB
    pub const CHECK_REWARD_POOL_SIZE: usize = 8 + std::mem::size_of::<RewardPool>(); // 83440 ~= 81.5KiB
    pub const CHECK_USER_REWARD_ACCOUNT_SIZE: usize = 8 + std::mem::size_of::<UserRewardAccount>(); // 4240 ~= 4.15KiB
    pub const CHECK_USER_REWARD_POOL_SIZE: usize = 8 + std::mem::size_of::<UserRewardPool>(); // 1040 ~= 1.02KiB

    #[test]
    fn check_size() {
        println!();
        println!("CHECK_REWARD_ACCOUNT_SIZE {}", CHECK_REWARD_ACCOUNT_SIZE);
        println!("CHECK_REWARD_POOL_SIZE {}", CHECK_REWARD_POOL_SIZE);
        println!(
            "CHECK_USER_REWARD_ACCOUNT_SIZE {}",
            CHECK_USER_REWARD_ACCOUNT_SIZE
        );
        println!(
            "CHECK_USER_REWARD_POOL_SIZE {}",
            CHECK_USER_REWARD_POOL_SIZE
        );
    }
}
