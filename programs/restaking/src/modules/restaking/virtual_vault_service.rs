use anchor_lang::prelude::*;
use anchor_spl::{associated_token, token_interface::Mint};

use crate::errors::ErrorCode;

pub struct VirtualVaultService<'info> {
    vault_account: &'info AccountInfo<'info>,
    vault_receipt_token_mint: &'info AccountInfo<'info>,
}

impl<'info> VirtualVaultService<'info> {
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
        vault_receipt_token_mint: &'info AccountInfo<'info>,
        fund_account: &AccountInfo,
    ) -> Result<()> {
        // validate vault address
        let (vault_address, _) = Pubkey::find_program_address(
            &[
                Self::VIRTUAL_VAULT_SEEDS,
                vault_receipt_token_mint.key().as_ref(),
                fund_account.key.as_ref(),
            ],
            &crate::ID,
        );
        require_keys_eq!(*vault_account.key, vault_address);
        require_keys_eq!(*vault_account.owner, System::id());

        // validate vrt mint
        let vault_receipt_token_mint_data =
            InterfaceAccount::<Mint>::try_from(vault_receipt_token_mint)?;

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
        token_mint: &InterfaceAccount<'info, Mint>,
        token_program: AccountInfo<'info>,
        from_vault_token_account: AccountInfo<'info>,
        to_token_account: AccountInfo<'info>,
        fund_account: AccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        require_keys_eq!(
            from_vault_token_account.key(),
            self.get_vault_token_account_address(&token_mint.key(), token_program.key)
        );

        let (_, virtual_vault_bump) = Pubkey::find_program_address(
            &[
                Self::VIRTUAL_VAULT_SEEDS,
                self.vault_receipt_token_mint.key().as_ref(),
                fund_account.key.as_ref(),
            ],
            &crate::ID,
        );

        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                token_program,
                anchor_spl::token_interface::TransferChecked {
                    from: from_vault_token_account,
                    mint: token_mint.to_account_info(),
                    to: to_token_account,
                    authority: self.vault_account.clone(),
                },
                &[&[
                    Self::VIRTUAL_VAULT_SEEDS,
                    self.vault_receipt_token_mint.key().as_ref(),
                    fund_account.key.as_ref(),
                    &[virtual_vault_bump],
                ]],
            ),
            amount,
            token_mint.decimals,
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
