mod fund_manager_updated_fund;
mod fund_manager_updated_reward_pool;
mod operator_claimed_remaining_reward;
mod operator_donated_to_fund;
mod operator_ran_fund_command;
mod operator_updated_fund_prices;
mod operator_updated_normalized_token_pool_prices;
mod operator_updated_reward_pools;
mod user_canceled_withdrawal_request_from_fund;
mod user_claimed_reward;
mod user_created_or_updated_fund_account;
mod user_created_or_updated_reward_account;
mod user_delegated_reward_account;
mod user_deposited_to_fund;
mod user_requested_withdrawal_from_fund;
mod user_transferred_receipt_token;
mod user_unwrapped_receipt_token;
mod user_updated_reward_pool;
mod user_withdrew_from_fund;
mod user_wrapped_receipt_token;

pub use fund_manager_updated_fund::*;
pub use fund_manager_updated_reward_pool::*;
pub use operator_claimed_remaining_reward::*;
pub use operator_donated_to_fund::*;
pub use operator_ran_fund_command::*;
pub use operator_updated_fund_prices::*;
pub use operator_updated_normalized_token_pool_prices::*;
pub use operator_updated_reward_pools::*;
pub use user_canceled_withdrawal_request_from_fund::*;
pub use user_claimed_reward::*;
pub use user_created_or_updated_fund_account::*;
pub use user_created_or_updated_reward_account::*;
pub use user_delegated_reward_account::*;
pub use user_deposited_to_fund::*;
pub use user_requested_withdrawal_from_fund::*;
pub use user_transferred_receipt_token::*;
pub use user_unwrapped_receipt_token::*;
pub use user_updated_reward_pool::*;
pub use user_withdrew_from_fund::*;
pub use user_wrapped_receipt_token::*;

use anchor_lang::prelude::*;

pub fn emit_cpi<'info>(
    event_authority_info: &'info AccountInfo<'info>,
    program_info: &'info AccountInfo<'info>,
    event: &impl anchor_lang::Event,
) -> Result<()> {
    let (event_authority_address, event_authority_bump) =
        Pubkey::find_program_address(&[b"__event_authority"], &crate::ID);
    require_keys_eq!(event_authority_info.key(), event_authority_address);
    require_keys_eq!(program_info.key(), crate::ID);

    let disc = anchor_lang::event::EVENT_IX_TAG_LE;
    let inner_data = event.data();
    let ix_data = disc.iter().copied().chain(inner_data).collect::<Vec<_>>();

    let ix = anchor_lang::solana_program::instruction::Instruction::new_with_bytes(
        crate::ID,
        &ix_data,
        vec![AccountMeta::new_readonly(*event_authority_info.key, true)],
    );

    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[event_authority_info.to_account_info()],
        &[&[b"__event_authority", &[event_authority_bump]]],
    )
    .map_err(anchor_lang::error::Error::from)
}
