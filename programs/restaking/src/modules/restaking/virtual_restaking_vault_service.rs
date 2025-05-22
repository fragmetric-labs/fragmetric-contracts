use anchor_lang::prelude::*;
use anchor_spl::{associated_token, token_interface::Mint};

use crate::errors::ErrorCode;

pub struct VirtualRestakingVaultService<'info> {
    vault_account: &'info AccountInfo<'info>,
    vault_receipt_token_mint: &'info AccountInfo<'info>,
}

impl<'info> VirtualRestakingVaultService<'info> {
    pub fn new(
        vault_account: &'info AccountInfo<'info>,
        vault_receipt_token_mint: &'info AccountInfo<'info>,
    ) -> Result<Self> {
        require_keys_eq!(*vault_account.owner, System::id());

        Ok(Self {
            vault_account,
            vault_receipt_token_mint,
        })
    }

    pub fn validate_vault(
        vault_account: &AccountInfo,
        vault_receipt_token_mint: &AccountInfo,
    ) -> Result<()> {
        require_keys_eq!(*vault_account.owner, System::id()); // do again? it is checked at new method -> but should also be here, because this method doesn't access new().

        let vault_receipt_token_mint_data =
            Mint::try_deserialize(&mut &vault_receipt_token_mint.data.borrow()[..])?;

        require_eq!(vault_receipt_token_mint_data.supply, 0);
        require!(
            vault_receipt_token_mint_data.mint_authority.is_none(),
            ErrorCode::FundRestakingVaultAuthorityNotMatchedError
        );

        Ok(())
    }

    pub const VIRTUAL_VAULT_SEEDS: &'static [u8] = b"virtual_vault";

    pub fn transfer_vault_token(
        &self,
        mint: &InterfaceAccount<'info, Mint>,
        token_program: AccountInfo<'info>,
        from_vault_token_account: AccountInfo<'info>,
        to_token_account: AccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        require_keys_eq!(
            from_vault_token_account.key(),
            self.get_vault_token_account_address(&mint.key(), token_program.key)
        );

        let (_, virtual_vault_bump) = Pubkey::find_program_address(
            &[
                Self::VIRTUAL_VAULT_SEEDS,
                self.vault_receipt_token_mint.key().as_ref(),
            ],
            &crate::ID,
        );

        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                token_program,
                anchor_spl::token_interface::TransferChecked {
                    from: from_vault_token_account,
                    mint: mint.to_account_info(),
                    to: to_token_account,
                    authority: self.vault_account.clone(),
                },
                &[&[
                    Self::VIRTUAL_VAULT_SEEDS,
                    self.vault_receipt_token_mint.key().as_ref(),
                    &[virtual_vault_bump],
                ]],
            ),
            amount,
            mint.decimals,
        )
    }

    fn get_vault_token_account_address(&self, mint: &Pubkey, token_program_id: &Pubkey) -> Pubkey {
        associated_token::get_associated_token_address_with_program_id(
            self.vault_account.key,
            mint,
            token_program_id,
        )
    }
}
