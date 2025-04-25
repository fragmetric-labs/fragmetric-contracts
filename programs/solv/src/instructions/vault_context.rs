use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token::{spl_token, Mint, Token, TokenAccount};

use crate::constants;

#[event_cpi]
#[derive(Accounts)]
pub struct VaultAccountInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// TODO: later it will be delegated to a PDA (restaking fund)
    pub admin: Signer<'info>,

    pub delegate_reward_token_admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(mut)]
    pub receipt_token_mint: Box<Account<'info, Mint>>,

    pub supported_token_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,

    #[account(
        init,
        payer = payer,
        seeds = [VaultAccount::SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = std::cmp::min(
            solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE,
            8 + std::mem::size_of::<VaultAccount>(),
        ),
    )]
    pub vault_account: AccountLoader<'info, VaultAccount>,

    #[account(
        associated_token::mint = receipt_token_mint,
        associated_token::authority = vault_account,
        associated_token::token_program = token_program,
    )]
    pub vault_receipt_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        associated_token::mint = supported_token_mint,
        associated_token::authority = vault_account,
        associated_token::token_program = token_program,
    )]
    pub vault_supported_token_account: Box<Account<'info, TokenAccount>>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct VaultRewardDelegationContext<'info> {
    pub admin: Signer<'info>,

    /// CHECK: ...
    pub delegate: UncheckedAccount<'info>,

    pub receipt_token_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,

    #[account(
        mut,
        seeds = [VaultAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = vault_account.load()?.bump,
    )]
    pub vault_account: AccountLoader<'info, VaultAccount>,

    pub reward_token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = reward_token_mint,
        associated_token::authority = vault_account,
        associated_token::token_program = token_program,
    )]
    pub vault_reward_token_account: Box<Account<'info, TokenAccount>>,
}

const MAX_DELEGATED_REWARD_TOKEN_MINTS: usize = 30;

#[account(zero_copy)]
#[repr(C)]
pub struct VaultAccount {
    data_version: u16,
    bump: u8,
    _padding2: [u8; 13],

    admin: Pubkey,
    delegate_reward_token_admin: Pubkey,
    _reserved: [u8; 960],

    receipt_token_mint: Pubkey,
    supported_token_mint: Pubkey,
    token_program: Pubkey,
    token_decimals: u8,
    _padding3: [u8; 14],

    num_delegated_reward_token_mints: u8,
    delegated_reward_token_mints: [Pubkey; MAX_DELEGATED_REWARD_TOKEN_MINTS],

    _reserved2: [u8; 8120],
}

impl<'info> VaultAccount {
    const SEED: &'static [u8] = b"vault";

    pub fn get_bump(&self) -> u8 {
        self.bump
    }

    fn get_seeds(&self) -> [&[u8]; 3] {
        [
            Self::SEED,
            self.receipt_token_mint.as_ref(),
            std::slice::from_ref(&self.bump),
        ]
    }

    pub fn process_initialize(
        &mut self,
        vault: &AccountInfo<'info>,
        vault_bump: u8,
        admin: &Signer<'info>,
        delegate_reward_token_admin: &Signer<'info>,
        receipt_token_mint: &Account<'info, Mint>,
        supported_token_mint: &Account<'info, Mint>,
        token_program: &AccountInfo<'info>,
    ) -> Result<()> {
        require_eq!(self.data_version, 0);
        self.data_version = 1;
        self.bump = vault_bump;

        // set authority of the vault
        self.admin = admin.key();
        self.delegate_reward_token_admin = delegate_reward_token_admin.key();

        // validate mints and transfer mint authority of the vault receipt token to the vault
        self.receipt_token_mint = receipt_token_mint.key();
        self.supported_token_mint = supported_token_mint.key();
        self.token_program = *supported_token_mint.to_account_info().owner;
        self.token_decimals = supported_token_mint.decimals;

        require_eq!(
            [
                constants::ZBTC_MINT_ADDRESS,
                constants::CBBTC_MINT_ADDRESS,
                constants::WBTC_MINT_ADDRESS
            ]
            .contains(&self.supported_token_mint),
            true
        );
        require_eq!(supported_token_mint.decimals, receipt_token_mint.decimals);
        require_eq!(supported_token_mint.decimals, 8);
        require_eq!(receipt_token_mint.supply, 0);

        anchor_spl::token::set_authority(
            CpiContext::new(
                token_program.to_account_info(),
                anchor_spl::token::SetAuthority {
                    current_authority: admin.to_account_info(),
                    account_or_mint: receipt_token_mint.to_account_info(),
                },
            ),
            spl_token::instruction::AuthorityType::MintTokens,
            Some(vault.key()),
        )
    }

    pub fn process_delegate_reward_token_account(
        &self,
        vault: &AccountInfo<'info>,
        admin: &Signer<'info>,
        delegate: &AccountInfo<'info>,

        reward_token_account: &Account<'info, TokenAccount>,
        token_program: &AccountInfo<'info>,
    ) -> Result<()> {
        // validate delegation authority
        require_eq!(
            [self.admin, self.delegate_reward_token_admin].contains(admin.key),
            true
        );

        // validate eligible tokens
        require_keys_neq!(reward_token_account.mint, self.supported_token_mint);
        require_keys_neq!(reward_token_account.mint, self.receipt_token_mint);

        anchor_spl::token::approve(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                anchor_spl::token::Approve {
                    to: reward_token_account.to_account_info(),
                    delegate: delegate.to_account_info(),
                    authority: vault.to_account_info(),
                },
                &[self.get_seeds().as_ref()],
            ),
            u64::MAX,
        )
    }

    pub fn add_delegate_reward_token_account(
        &mut self,
        reward_token_account: &Account<'info, TokenAccount>,
    ) -> Result<()> {
        // remember registered reward tokens
        if self
            .delegated_reward_token_mints
            .iter()
            .take(self.num_delegated_reward_token_mints as usize)
            .find(|mint| mint.key() == reward_token_account.mint)
            .is_none()
        {
            require_neq!(
                self.num_delegated_reward_token_mints as usize,
                MAX_DELEGATED_REWARD_TOKEN_MINTS
            );
            self.delegated_reward_token_mints[self.num_delegated_reward_token_mints as usize] =
                reward_token_account.mint;
            self.num_delegated_reward_token_mints += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size() {
        const VAULT_ACCOUNT_SIZE: usize = 8 + std::mem::size_of::<VaultAccount>();
        assert_eq!(VAULT_ACCOUNT_SIZE, 1024 * 10);
    }
}
