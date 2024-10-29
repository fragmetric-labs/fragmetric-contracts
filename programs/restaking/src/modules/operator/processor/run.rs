use anchor_lang::prelude::*;
use anchor_spl::token::accessor::mint;
use anchor_spl::token_interface::Mint;
use crate::modules::fund;
use crate::constants::{FUND_MANAGER_PUBKEY};

pub fn process_run<'info>(
    operator: &Signer<'info>,
    _receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    fund_account: &mut Account<'info, fund::FundAccount>,
    remaining_accounts: &'info [AccountInfo<'info>],
    _current_timestamp: i64,
    _current_slot: u64,
) -> Result<()> {
    require_eq!(operator.key(), FUND_MANAGER_PUBKEY);

    // 1. stake sol to jitoSOL
    {
        let amount_in = fund_account.get_sol_operation_reserved_amount();
        fund::stake_sol_operation_reserved(
            fund_account,
            amount_in,
            // TODO: pick required accounts for this fn
            remaining_accounts,
        );
    }

    // 2. normalize supported tokens
    // TODO: nt_opeartion_reserved_amount -> fund_account_ref.get_nt_operation_reserved_amount()
    let mut nt_opeartion_reserved_amount = 0u64;
    {
        let supported_tokens = fund_account.get_supported_tokens_iter()
            .map(|token| (token.get_mint().clone(), token.get_operation_reserved_amount()))
            .collect::<Vec<_>>();
        for (supported_token_mint, supported_token_operation_reserved_amount) in supported_tokens {
            nt_opeartion_reserved_amount += fund::normalize_lst_operation_reserved(
                fund_account,
                &supported_token_mint,
                supported_token_operation_reserved_amount,
                // TODO: pick required accounts for this fn
                remaining_accounts,
            )?;
        }
    }

    // 3. restake normalized tokens
    {
        fund::restake_nt_operation_reserved(
            fund_account,
            nt_opeartion_reserved_amount,
            // TODO: pick required accounts for this fn
            remaining_accounts,
        );
    }

    Ok(())
}

// fn pick_account<'info, T: AccountDeserialize + Clone>(key: &Pubkey, accounts: &[AccountInfo<'info>]) -> Result<Box<AccountInfo<'info>>> {
//     accounts.iter().find(|account| {
//         return account.key.eq(key);
//     }).map_or_else(Err(Error::from(ProgramError::NotEnoughAccountKeys)), |account| {
//         let b = Box::new(Account::<T>::try_from(account)?);
//         return b.as_ref();
//     })
// }