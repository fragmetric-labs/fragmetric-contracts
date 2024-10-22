mod processor;

pub use processor::*;

mod allocated_amount;
mod metadata;
mod reward_account;
mod reward_settlement;
mod user_reward_account;
mod user_reward_settlement;

pub use allocated_amount::*;
pub use metadata::*;
pub use reward_account::*;
pub use reward_settlement::*;
pub use user_reward_account::*;
pub use user_reward_settlement::*;

#[cfg(test)]
pub mod check_account_init_space {
    use super::*;

    pub const CHECK_REWARD_ACCOUNT_SIZE: usize = std::mem::size_of::<RewardAccount>(); // 342064 ~= 335KB
    pub const CHECK_REWARD_POOL_SIZE: usize = std::mem::size_of::<RewardPool>(); // 83440 ~= 81.5KiB
    pub const CHECK_USER_REWARD_ACCOUNT_SIZE: usize = std::mem::size_of::<UserRewardAccount>(); // 4240 ~= 4.15KiB
    pub const CHECK_USER_REWARD_POOL_SIZE: usize = std::mem::size_of::<UserRewardPool>(); // 1040 ~= 1.02KiB

    #[test]
    fn check_size() {
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
