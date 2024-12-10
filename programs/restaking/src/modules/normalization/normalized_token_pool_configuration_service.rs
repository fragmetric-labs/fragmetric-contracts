use super::*;
use crate::modules::fund::FundService;
use crate::modules::pricing::{PricingService, TokenPricingSource};
use anchor_lang::prelude::*;
use anchor_spl::token::spl_token;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

pub struct NormalizedTokenPoolConfigurationService<'info: 'a, 'a> {
    normalized_token_pool_account: &'a mut Account<'info, NormalizedTokenPoolAccount>,
    normalized_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    normalized_token_program: &'a Program<'info, Token>,
}

impl Drop for NormalizedTokenPoolConfigurationService<'_, '_> {
    fn drop(&mut self) {
        self.normalized_token_pool_account.exit(&crate::ID).unwrap();
    }
}

impl<'info, 'a> NormalizedTokenPoolConfigurationService<'info, 'a> {
    pub fn new(
        normalized_token_pool_account: &'a mut Account<'info, NormalizedTokenPoolAccount>,
        normalized_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        normalized_token_program: &'a Program<'info, Token>,
    ) -> Result<Self> {
        Ok(Self {
            normalized_token_pool_account,
            normalized_token_mint,
            normalized_token_program,
        })
    }

    pub fn process_initialize_normalized_token_pool_account(
        &mut self,
        normalized_token_mint_current_authority: &Signer<'info>,
        normalized_token_pool_account_bump: u8,
    ) -> Result<()> {
        self.normalized_token_pool_account.initialize(
            normalized_token_pool_account_bump,
            self.normalized_token_mint,
        );

        // set token mint authority
        if self.normalized_token_mint.mint_authority.unwrap_or_default() != self.normalized_token_pool_account.key() {
            anchor_spl::token::set_authority(
                CpiContext::new(
                    self.normalized_token_program.to_account_info(),
                    anchor_spl::token::SetAuthority {
                        current_authority: normalized_token_mint_current_authority.to_account_info(),
                        account_or_mint: self.normalized_token_mint.to_account_info(),
                    },
                ),
                spl_token::instruction::AuthorityType::MintTokens,
                Some(self.normalized_token_pool_account.key()),
            )?;
        }

        Ok(())
    }

    pub fn process_update_normalized_token_pool_account_if_needed(&mut self) -> Result<()> {
        self.normalized_token_pool_account
            .update_if_needed(self.normalized_token_mint);
        Ok(())
    }

    pub fn process_add_supported_token(
        &mut self,
        pool_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        supported_token_mint: &InterfaceAccount<Mint>,
        supported_token_program: &Interface<'info, TokenInterface>,
        pricing_source: TokenPricingSource,
        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<()> {
        require_keys_eq!(
            pool_supported_token_account.owner,
            self.normalized_token_pool_account.key()
        );
        require_keys_eq!(
            pool_supported_token_account.mint,
            supported_token_mint.key()
        );
        require_keys_eq!(
            *supported_token_mint.to_account_info().owner,
            supported_token_program.key()
        );

        self.normalized_token_pool_account.add_new_supported_token(
            supported_token_mint.key(),
            supported_token_program.key(),
            pool_supported_token_account.key(),
            pricing_source,
        )?;

        // validate pricing source
        NormalizedTokenPoolService::new(
            self.normalized_token_pool_account,
            self.normalized_token_mint,
            self.normalized_token_program,
        )?
        .new_pricing_service(pricing_sources)?;

        Ok(())
    }
}
