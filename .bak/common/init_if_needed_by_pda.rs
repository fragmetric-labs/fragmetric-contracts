use anchor_lang::prelude::*;
use super::{CustomAccount, SystemProgramExt};

/// Initialize the program account if needed, paid by PDA.
/// This trait is to customize anchor framework to utilize initialization with PDA as a payer.
/// Payer PDA must be owned by system program.
/// Therefore it is recommended to create a dedicated PDA for this operation.
pub trait InitIfNeededByPDA<'info> {
    /// Initialize the program account if needed, paid by PDA.
    /// This trait behaves like `init_if_needed` anchor macro.
    fn init_if_needed_by_pda<T: AccountSerialize + AccountDeserialize + Owner + Clone>(
        &self,
        account_name: &str,
        payer_pda: &AccountInfo<'info>,
        payer_pda_signer_seeds: &[&[&[u8]]],
        minimum_space: usize,
        new_account_signer_seeds: Option<&[&[&[u8]]]>,
        system_program: &Program<'info, System>,
    ) -> Result<CustomAccount<'_, 'info, T>>;
}

impl<'info> InitIfNeededByPDA<'info> for UncheckedAccount<'info> {
    #[inline(never)]
    fn init_if_needed_by_pda<T: AccountSerialize + AccountDeserialize + Owner + Clone>(
        &self,
        account_name: &str,
        payer_pda: &AccountInfo<'info>,
        payer_pda_signer_seeds: &[&[&[u8]]],
        minimum_space: usize,
        new_account_signer_seeds: Option<&[&[&[u8]]]>,
        system_program: &Program<'info, System>,
    ) -> Result<CustomAccount<'_, 'info, T>> {
        let rent = Rent::get()?;
        let actual_owner = self.owner;
        let account = if actual_owner == &anchor_lang::solana_program::system_program::ID {
            let current_lamports = self.lamports();
            if current_lamports == 0 {
                let lamports = rent.minimum_balance(minimum_space);
                system_program.create_account(
                    payer_pda,
                    Some(payer_pda_signer_seeds),
                    self,
                    new_account_signer_seeds,
                    minimum_space as u64,
                    lamports,
                    &crate::ID,
                )?;
            } else {
                require_keys_neq!(
                    payer_pda.key(),
                    self.key(),
                    ErrorCode::TryingToInitPayerAsProgramAccount,
                );
                let required_lamports = rent
                    .minimum_balance(minimum_space)
                    .max(1)
                    .saturating_sub(current_lamports);
                if required_lamports > 0 {
                    system_program.transfer_by_pda(
                        payer_pda,
                        Some(payer_pda_signer_seeds),
                        self,
                        required_lamports,
                    )?;
                }
                system_program.allocate(self, new_account_signer_seeds, minimum_space as u64)?;
                system_program.assign(self, new_account_signer_seeds, &crate::ID)?;
            }
            CustomAccount::try_from_unchecked(self)
                .map_err(|e| e.with_account_name(account_name))?
        } else {
            CustomAccount::try_from(self).map_err(|e| e.with_account_name(account_name))?
        };

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
