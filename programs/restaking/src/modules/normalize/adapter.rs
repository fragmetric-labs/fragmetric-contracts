use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};

use crate::errors::ErrorCode;
use crate::utils::PDASeeds;

use super::*;

pub struct NormalizedTokenPoolAdapter<'info> {
    normalized_token_pool_account: Box<Account<'info, NormalizedTokenPoolAccount>>,
    normalized_token_mint: Box<InterfaceAccount<'info, Mint>>,
    normalized_token_program: Program<'info, Token>,
    supported_token_mint: Box<InterfaceAccount<'info, Mint>>,
    supported_token_program: Interface<'info, TokenInterface>,
    supported_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>,
}

impl<'info> NormalizedTokenPoolAdapter<'info> {
    pub(in crate::modules) fn new(
        normalized_token_pool_account: Box<Account<'info, NormalizedTokenPoolAccount>>,
        normalized_token_mint: Box<InterfaceAccount<'info, Mint>>,
        normalized_token_program: Program<'info, Token>,
        supported_token_mint: Box<InterfaceAccount<'info, Mint>>,
        supported_token_program: Interface<'info, TokenInterface>,
        supported_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>,
    ) -> Result<Self> {
        Ok(Self {
            normalized_token_pool_account,
            normalized_token_mint,
            normalized_token_program,
            supported_token_mint,
            supported_token_program,
            supported_token_lock_account,
        })
    }

    pub(in crate::modules) fn get_denominated_amount_per_normalized_token(&self) -> Result<u64> {
        10u64
            .checked_pow(self.normalized_token_mint.decimals as u32)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }

    pub(super) fn deposit(
        &mut self,
        normalized_token_account: &InterfaceAccount<'info, TokenAccount>,
        supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        signer: AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]],
        supported_token_amount: u64,
        supported_token_amount_as_sol: u64,
        one_normalized_token_as_sol: u64,
    ) -> Result<()> {
        msg!(
            "FUCK2-1: {}, {}, {}",
            supported_token_amount_as_sol,
            self.get_denominated_amount_per_normalized_token()?,
            one_normalized_token_as_sol
        );
        let normalized_token_mint_amount = crate::utils::get_proportional_amount(
            supported_token_amount_as_sol,
            self.get_denominated_amount_per_normalized_token()?,
            one_normalized_token_as_sol,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        self.normalized_token_pool_account
            .get_supported_token_mut(self.supported_token_mint.key())?
            .lock_token(supported_token_amount)?;

        token_interface::transfer_checked(
            CpiContext::new_with_signer(
                self.supported_token_program.to_account_info(),
                token_interface::TransferChecked {
                    from: supported_token_account.to_account_info(),
                    mint: self.supported_token_mint.to_account_info(),
                    to: self.supported_token_lock_account.to_account_info(),
                    authority: signer,
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
                    to: normalized_token_account.to_account_info(),
                    authority: self.normalized_token_pool_account.to_account_info(),
                },
                &[self
                    .normalized_token_pool_account
                    .get_signer_seeds()
                    .as_ref()],
            ),
            normalized_token_mint_amount,
        )?;

        Ok(())
    }

    pub(crate) fn into_pool_account(self) -> Box<Account<'info, NormalizedTokenPoolAccount>> {
        self.normalized_token_pool_account
    }
}
