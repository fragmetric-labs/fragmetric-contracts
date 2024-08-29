use anchor_lang::prelude::*;

use super::{system_program::*, CustomAccount};

/// Initialize the program account if needed, paid by user.
/// This trait is to customize anchor framework to utilize initialization with user as a payer.
pub(crate) trait InitIfNeededByUser<'info> {
    /// Initialize the program account if needed, paid by user.
    /// This trait behaves like `init_if_needed` anchor macro.
    fn init_if_needed_by_user<T: AccountSerialize + AccountDeserialize + Owner + Clone>(
        &self,
        account_name: &str,
        payer: &AccountInfo<'info>,
        minimum_space: usize,
        system_program: &Program<'info, System>,
    ) -> Result<CustomAccount<'_, 'info, T>>;
}

impl<'info> InitIfNeededByUser<'info> for UncheckedAccount<'info> {
    fn init_if_needed_by_user<T: AccountSerialize + AccountDeserialize + Owner + Clone>(
        &self,
        account_name: &str,
        payer: &AccountInfo<'info>,
        minimum_space: usize,
        system_program: &Program<'info, System>,
    ) -> Result<CustomAccount<'_, 'info, T>> {
        let rent = Rent::get()?;
        let actual_owner = self.owner;
        let account = if actual_owner == &anchor_lang::solana_program::system_program::ID {
            let current_lamports = self.lamports();
            if current_lamports == 0 {
                let lamports = rent.minimum_balance(minimum_space);
                msg!("111");
                system_program.create_program_account_from_user(
                    payer,
                    self,
                    minimum_space as u64,
                    lamports,
                )?;
            } else {
                require_keys_neq!(
                    payer.key(),
                    self.key(),
                    ErrorCode::TryingToInitPayerAsProgramAccount
                );
                let required_lamports = rent
                    .minimum_balance(minimum_space)
                    .max(1)
                    .saturating_sub(current_lamports);
                msg!("222");
                if required_lamports > 0 {
                    system_program.transfer_from_user(payer, self, required_lamports)?;
                }
                system_program.allocate_from_user(self, minimum_space as u64)?;
                system_program.assign_to_program_from_user(self)?;
            }
            msg!("333");
            CustomAccount::try_from_unchecked(self)
                .map_err(|e| e.with_account_name(account_name))?
        } else {
            msg!("444");
            CustomAccount::try_from(self).map_err(|e| e.with_account_name(account_name))?
        };
        msg!("555");

        if minimum_space != self.data_len() {
            return Err(anchor_lang::error::Error::from(ErrorCode::ConstraintSpace)
                .with_account_name(account_name)
                .with_values((minimum_space, self.data_len())));
        }

        if actual_owner != &crate::ID {
            return Err(anchor_lang::error::Error::from(ErrorCode::ConstraintOwner)
                .with_account_name(account_name)
                .with_values((*actual_owner, crate::ID)));
        }

        if !rent.is_exempt(self.lamports(), self.data_len()) {
            return Err(anchor_lang::error::Error::from(
                anchor_lang::error::ErrorCode::ConstraintRentExempt,
            )
            .with_account_name(account_name));
        }

        Ok(account)
    }
}
