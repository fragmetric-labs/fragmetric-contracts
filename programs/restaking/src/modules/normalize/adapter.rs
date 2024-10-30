use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};

use crate::errors::ErrorCode;
use crate::utils::PDASeeds;

use super::*;

pub struct NormalizedTokenPoolAdapter<'info> {
    pub(super) normalized_token_pool_config: Box<Account<'info, NormalizedTokenPoolConfig>>,
    pub(super) normalized_token_mint: Box<InterfaceAccount<'info, Mint>>,
    normalized_token_program: Program<'info, Token>,
    normalized_token_authority: Box<Account<'info, NormalizedTokenAuthority>>,
    normalized_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    supported_token_mint: Box<InterfaceAccount<'info, Mint>>,
    supported_token_program: Box<Interface<'info, TokenInterface>>,
    supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    supported_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>,
}

impl<'info> NormalizedTokenPoolAdapter<'info> {
    pub(super) fn new(
        normalized_token_pool_config: Account<'info, NormalizedTokenPoolConfig>,
        accounts: &'info [AccountInfo<'info>],
    ) -> Result<Self> {
        normalized_token_pool_config.validate_adapter_constructor_accounts(accounts)?;

        Ok(Self {
            normalized_token_pool_config: Box::new(normalized_token_pool_config),
            normalized_token_mint: Box::new(InterfaceAccount::try_from(&accounts[0])?),
            normalized_token_program: Program::try_from(&accounts[1])?,
            normalized_token_authority: Box::new(Account::try_from(&accounts[2])?),
            normalized_token_account: Box::new(InterfaceAccount::try_from(&accounts[3])?),
            supported_token_mint: Box::new(InterfaceAccount::try_from(&accounts[4])?),
            supported_token_program: Box::new(Interface::try_from(&accounts[5])?),
            supported_token_account: Box::new(InterfaceAccount::try_from(&accounts[6])?),
            supported_token_lock_account: Box::new(InterfaceAccount::try_from(&accounts[7])?),
        })
    }

    pub(super) fn deposit(
        &mut self,
        supported_token_authority: AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]],
        supported_token_amount: u64,
        normalized_token_mint_amount: u64,
    ) -> Result<()> {
        require_gte!(self.supported_token_account.amount, supported_token_amount);

        let supported_token_config = self
            .normalized_token_pool_config
            .get_supported_token_config_mut(self.supported_token_mint.key())?;
        supported_token_config.locked_amount = supported_token_config
            .locked_amount
            .checked_add(supported_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        token_interface::transfer_checked(
            CpiContext::new_with_signer(
                self.supported_token_program.to_account_info(),
                token_interface::TransferChecked {
                    from: self.supported_token_account.to_account_info(),
                    mint: self.supported_token_mint.to_account_info(),
                    to: self.supported_token_lock_account.to_account_info(),
                    authority: supported_token_authority,
                },
                signer_seeds,
            ),
            supported_token_amount,
            self.supported_token_mint.decimals,
        )?;

        token_interface::mint_to(
            CpiContext::new_with_signer(
                self.normalized_token_program.to_account_info(),
                token_interface::MintTo {
                    mint: self.normalized_token_mint.to_account_info(),
                    to: self.normalized_token_account.to_account_info(),
                    authority: self.normalized_token_authority.to_account_info(),
                },
                &[self.normalized_token_authority.get_signer_seeds().as_ref()],
            ),
            normalized_token_mint_amount,
        )?;

        Ok(())
    }
}
