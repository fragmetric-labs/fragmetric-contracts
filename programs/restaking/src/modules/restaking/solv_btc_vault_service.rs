use anchor_lang::prelude::*;
use anchor_spl::{token::{Mint, Token}, token_interface::TokenAccount};
use solv::states::VaultAccount;

use crate::constants::SOLV_PROGRAM_ID;

pub(in crate::modules) struct SolvBTCVaultService<'info> {
    vault_program: &'info AccountInfo<'info>,
    vault_account: AccountLoader<'info, VaultAccount>,
}

impl<'info> SolvBTCVaultService<'info> {
    pub fn new(
        vault_program: &'info AccountInfo<'info>,
        vault_account: &'info AccountInfo<'info>,
    ) -> Result<Self> {
        require_keys_eq!(SOLV_PROGRAM_ID, vault_program.key());
        require_keys_eq!(*vault_account.owner, vault_program.key());

        Ok(Self {
            vault_program,
            vault_account: AccountLoader::try_from(vault_account)?,
        })
    }

    pub fn validate_vault(
        vault_account: &'info AccountInfo<'info>,
        vault_supported_token_mint: &AccountInfo,
        vault_receipt_token_mint: &AccountInfo,
        fund_account: &AccountInfo,
    ) -> Result<()> {
        let vault_account = AccountLoader::<VaultAccount>::try_from(vault_account)?;
        let vault = vault_account.load()?;

        require_keys_eq!(vault.get_fund_manager(), fund_account.key());
        require_keys_eq!(vault.get_vst_mint(), vault_supported_token_mint.key());
        require_keys_eq!(vault.get_vrt_mint(), vault_receipt_token_mint.key());

        Ok(())
    }

    fn find_solv_event_authority_address() -> Pubkey {
        Pubkey::find_program_address(&[b"__event_authority"], &SOLV_PROGRAM_ID).0
    }

    /// * (0) vault_program
    /// * (1) vault_account(writable)
    pub fn find_accounts_to_new(
        vault_address: Pubkey,
    ) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        Ok([(SOLV_PROGRAM_ID, false), (vault_address, true)].into_iter())
    }

    /// * (0) vault_program
    /// * (1) vault_account (writable)
    /// * (2) vault_receipt_token_mint (writable)
    /// * (3) vault_supported_token_mint
    /// * (4) vault_vault_supported_token_account (writable)
    /// * (5) token_program
    /// * (6) event_authority
    fn find_accounts_to_cpi(&self) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        let vault = self.vault_account.load()?;

        // todo! return required arrays
        let vault_address = self.vault_account.key();
        let vst_mint = vault.get_vst_mint();
        let accounts = Self::find_accounts_to_new(vault_address)?.chain([
            (vault.get_vrt_mint(), true),
            (vault.get_vst_mint(), false),
            (
                anchor_spl::associated_token::get_associated_token_address(
                    &vault_address,
                    &vst_mint,
                ),
                true,
            ),
            (Token::id(), false),
            (Self::find_solv_event_authority_address(), false),
        ]);

        Ok(accounts)
    }

    pub fn get_supported_token_mint(&self) -> Result<Pubkey> {
        let vault = self.vault_account.load()?;
        Ok(vault.get_vst_mint())
    }

    /// * (0) vault_program
    /// * (1) vault_account (writable)
    /// * (2) vault_receipt_token_mint (writable)
    /// * (3) vault_supported_token_mint
    /// * (4) vault_vault_supported_token_account (writable)
    /// * (5) token_program
    /// * (6) event_authority
    pub fn find_accounts_to_deposit(&self) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        self.find_accounts_to_cpi()
    }

    /// * (0) vault_program
    /// * (1) vault_account (writable)
    /// * (2) vault_receipt_token_mint (writable)
    /// * (3) vault_supported_token_mint
    /// * (4) vault_vault_supported_token_account (writable)
    /// * (5) token_program
    /// * (6) event_authority
    pub fn find_accounts_to_request_withdrawal(
        &self,
    ) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        self.find_accounts_to_cpi()
    }

    /// * (0) vault_program
    /// * (1) vault_account (writable)
    /// * (2) vault_receipt_token_mint (writable)
    /// * (3) vault_supported_token_mint
    /// * (4) vault_vault_supported_token_account (writable)
    /// * (5) token_program
    /// * (6) event_authority
    pub fn find_accounts_to_withdraw(&self) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        self.find_accounts_to_cpi()
    }

    /// returns [payer_vault_receipt_token_account_amount, minted_vault_receipt_token_amount, deposited_supported_token_amount]
    #[inline(never)]
    pub fn deposit(
        &self,
        // fixed
        vault_receipt_token_mint: &AccountInfo<'info>,
        vault_supported_token_mint: &AccountInfo<'info>,
        vault_vault_supported_token_account: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
        event_authority: &AccountInfo<'info>,

        // variant
        fund_manager: &AccountInfo<'info>,
        fund_manager_seeds: &[&[&[u8]]],
        payer_vault_receipt_token_account: &'info AccountInfo<'info>,
        payer_vault_supported_token_account: &'info AccountInfo<'info>,
        payer: &AccountInfo<'info>,
        payer_seeds: &[&[&[u8]]],

        supported_token_amount: u64,
    ) -> Result<(u64, u64, u64)> {
        let mut signer_seeds = Vec::with_capacity(2);
        if let Some(seeds) = fund_manager_seeds.first() {
            signer_seeds.push(*seeds);
        }
        if let Some(seeds) = payer_seeds.first() {
            signer_seeds.push(*seeds);
        }

        let mut payer_vault_supported_token_account =
            InterfaceAccount::<TokenAccount>::try_from(payer_vault_supported_token_account)?;
        let payer_vault_supported_token_account_amount_before =
            payer_vault_supported_token_account.amount;
        let mut payer_vault_receipt_token_account =
            InterfaceAccount::<TokenAccount>::try_from(payer_vault_receipt_token_account)?;
        let payer_vault_receipt_token_account_amount_before =
            payer_vault_receipt_token_account.amount;

        solv::cpi::fund_manager_deposit(
            CpiContext::new_with_signer(
                self.vault_program.to_account_info(),
                solv::cpi::accounts::FundManagerContext {
                    payer: payer.to_account_info(),
                    fund_manager: fund_manager.to_account_info(),
                    vault_account: self.vault_account.to_account_info(),
                    vault_receipt_token_mint: vault_receipt_token_mint.to_account_info(),
                    vault_supported_token_mint: vault_supported_token_mint.to_account_info(),
                    payer_vault_receipt_token_account: payer_vault_receipt_token_account
                        .to_account_info(),
                    payer_vault_supported_token_account: payer_vault_supported_token_account
                        .to_account_info(),
                    vault_vault_supported_token_account: vault_vault_supported_token_account
                        .to_account_info(),
                    token_program: token_program.to_account_info(),
                    event_authority: event_authority.to_account_info(),
                    program: self.vault_program.to_account_info(),
                },
                &signer_seeds,
            ),
            supported_token_amount,
        )?;

        payer_vault_supported_token_account.reload()?;
        let payer_vault_supported_token_account_amount = payer_vault_supported_token_account.amount;
        let deposited_supported_token_amount = payer_vault_supported_token_account_amount_before
            - payer_vault_supported_token_account_amount;

        payer_vault_receipt_token_account.reload()?;
        let payer_vault_receipt_token_account_amount = payer_vault_receipt_token_account.amount;
        let minted_vault_receipt_token_amount = payer_vault_receipt_token_account_amount
            - payer_vault_receipt_token_account_amount_before;

        msg!("RESTAKE#solv deposited: vrt_mint={}, deposited_vst_amount={}, to_vrt_account_amount={}, minted_vrt_amount={}",
            vault_receipt_token_mint.key,
            deposited_supported_token_amount,
            payer_vault_receipt_token_account_amount,
            minted_vault_receipt_token_amount
        );

        Ok((
            payer_vault_receipt_token_account_amount,
            minted_vault_receipt_token_amount,
            deposited_supported_token_amount,
        ))
    }

    /// returns [payer_vault_receipt_token_account_amount, enqueued_vault_receipt_token_amount, total_withdrawal_incompleted_vault_receipt_token_amount, payer_vault_supported_token_account_amount, expected_supported_token_amount]
    #[inline(never)]
    pub fn request_withdrawal(
        &self,
        // fixed
        vault_receipt_token_mint: &AccountInfo<'info>,
        vault_supported_token_mint: &AccountInfo<'info>,
        vault_vault_supported_token_account: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
        event_authority: &AccountInfo<'info>,

        // variant
        fund_manager: &AccountInfo<'info>,
        fund_manager_seeds: &[&[&[u8]]],
        payer_vault_receipt_token_account: &'info AccountInfo<'info>,
        payer_vault_supported_token_account: &'info AccountInfo<'info>,
        payer: &AccountInfo<'info>,
        payer_seeds: &[&[&[u8]]],

        receipt_token_amount: u64,
    ) -> Result<(u64, u64, u64, u64, u64)> {
        let total_incompleted_withdrawal_vault_receipt_token_amount_before = self
            .vault_account
            .load()?
            .get_vrt_withdrawal_incompleted_amount();

        let mut payer_vault_receipt_token_account =
            InterfaceAccount::<TokenAccount>::try_from(payer_vault_receipt_token_account)?;
        let mut payer_vault_supported_token_account =
            InterfaceAccount::<TokenAccount>::try_from(payer_vault_supported_token_account)?;

        let mut signer_seeds = Vec::with_capacity(2);
        if let Some(seeds) = fund_manager_seeds.first() {
            signer_seeds.push(*seeds);
        }
        if let Some(seeds) = payer_seeds.first() {
            signer_seeds.push(*seeds);
        }

        solv::cpi::fund_manager_request_withdrawal(
            CpiContext::new_with_signer(
                self.vault_program.to_account_info(),
                solv::cpi::accounts::FundManagerContext {
                    payer: payer.to_account_info(),
                    fund_manager: fund_manager.to_account_info(),
                    vault_account: self.vault_account.to_account_info(),
                    vault_receipt_token_mint: vault_receipt_token_mint.to_account_info(),
                    vault_supported_token_mint: vault_supported_token_mint.to_account_info(),
                    payer_vault_receipt_token_account: payer_vault_receipt_token_account
                        .to_account_info(),
                    payer_vault_supported_token_account: payer_vault_supported_token_account
                        .to_account_info(),
                    vault_vault_supported_token_account: vault_vault_supported_token_account
                        .to_account_info(),
                    token_program: token_program.to_account_info(),
                    event_authority: event_authority.to_account_info(),
                    program: self.vault_program.to_account_info(),
                },
                &signer_seeds,
            ),
            receipt_token_amount,
        )?;

        payer_vault_receipt_token_account.reload()?;
        let payer_vault_receipt_token_account_amount = payer_vault_receipt_token_account.amount;

        let total_incompleted_withdrawal_vault_receipt_token_amount = self
            .vault_account
            .load()?
            .get_vrt_withdrawal_incompleted_amount();
        let enqueued_vault_receipt_token_amount =
            total_incompleted_withdrawal_vault_receipt_token_amount
                - total_incompleted_withdrawal_vault_receipt_token_amount_before;

        payer_vault_supported_token_account.reload()?;
        let payer_vault_supported_token_account_amount = payer_vault_supported_token_account.amount;

        let expected_supported_token_amount = self
            .vault_account
            .load()?
            .get_vst_estimated_amount_from_last_withdrawal_request()?;

        msg!("UNRESTAKE#solv: receipt_token_mint={}, enqueued_vault_receipt_token_account={}, from_vault_receipt_token_account_amount={}",
            vault_receipt_token_mint.key,
            enqueued_vault_receipt_token_amount,
            payer_vault_receipt_token_account_amount
        );

        Ok((
            payer_vault_receipt_token_account_amount,
            enqueued_vault_receipt_token_amount,
            total_incompleted_withdrawal_vault_receipt_token_amount,
            payer_vault_supported_token_account_amount,
            expected_supported_token_amount,
        ))
    }

    /// returns [payer_vault_supported_token_amount, unrestaked_receipt_token_amount, claimed_supported_token_amount, deducted_fee_amount]
    #[inline(never)]
    pub fn withdraw(
        &self,
        // fixed
        vault_receipt_token_mint: &AccountInfo<'info>,
        vault_supported_token_mint: &AccountInfo<'info>,
        vault_vault_supported_token_account: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
        event_authority: &AccountInfo<'info>,

        // variant
        fund_manager: &AccountInfo<'info>,
        fund_manager_seeds: &[&[&[u8]]],
        payer_vault_receipt_token_account: &'info AccountInfo<'info>,
        payer_vault_supported_token_account: &'info AccountInfo<'info>,
        payer: &AccountInfo<'info>,
        payer_seeds: &[&[&[u8]]],

        fund_supported_treasury_account: &AccountInfo<'info>,
    ) -> Result<(u64, u64, u64, u64)> {
        let mut signer_seeds = Vec::with_capacity(2);
        if let Some(seeds) = fund_manager_seeds.first() {
            signer_seeds.push(*seeds);
        }
        if let Some(seeds) = payer_seeds.first() {
            signer_seeds.push(*seeds);
        }

        let mut payer_vault_supported_token_account =
            InterfaceAccount::<TokenAccount>::try_from(payer_vault_supported_token_account)?;
        let payer_vault_receipt_token_account =
            InterfaceAccount::<TokenAccount>::try_from(payer_vault_receipt_token_account)?;
        let payer_vault_supported_token_account_amount_before =
            payer_vault_supported_token_account.amount;
        let payer_vault_receipt_token_withdrawal_completed_amount_before = self
            .vault_account
            .load()?
            .get_vrt_withdrawal_completed_amount();
        let deducted_supported_token_fee_amount =
            self.vault_account.load()?.get_vst_deducted_fee_amount();
        let extra_amount_to_claim = self.vault_account.load()?.get_vst_extra_amount_to_claim();

        solv::cpi::fund_manager_withdraw(CpiContext::new_with_signer(
            self.vault_program.to_account_info(),
            solv::cpi::accounts::FundManagerContext {
                payer: payer.to_account_info(),
                fund_manager: fund_manager.to_account_info(),
                vault_account: self.vault_account.to_account_info(),
                vault_receipt_token_mint: vault_receipt_token_mint.to_account_info(),
                vault_supported_token_mint: vault_supported_token_mint.to_account_info(),
                payer_vault_receipt_token_account: payer_vault_receipt_token_account
                    .to_account_info(),
                payer_vault_supported_token_account: payer_vault_supported_token_account
                    .to_account_info(),
                vault_vault_supported_token_account: vault_vault_supported_token_account
                    .to_account_info(),
                token_program: token_program.to_account_info(),
                event_authority: event_authority.to_account_info(),
                program: self.vault_program.to_account_info(),
            },
            &signer_seeds,
        ))?;

        // send extra amount of supported tokens to treasury
        let mint_data_ref = vault_supported_token_mint.try_borrow_data()?;
        let mint = Mint::try_deserialize(&mut mint_data_ref.as_ref())?;
        let decimals = mint.decimals;

        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: payer_vault_supported_token_account.to_account_info(),
                    to: fund_supported_treasury_account.to_account_info(),
                    mint: vault_supported_token_mint.to_account_info(),
                    authority: payer.to_account_info(),
                },
                &signer_seeds,
            ),
            extra_amount_to_claim,
            decimals,
        )?;

        let unrestaked_receipt_token_amount =
            payer_vault_receipt_token_withdrawal_completed_amount_before;

        payer_vault_supported_token_account.reload()?;
        let payer_vault_supported_token_account_amount = payer_vault_supported_token_account.amount;
        let claimed_supported_token_amount = payer_vault_supported_token_account_amount
            - payer_vault_supported_token_account_amount_before
            - extra_amount_to_claim;

        msg!("CLAIM_UNRESTAKED#solv: receipt_token_mint={}, to_vault_supported_token_account_amount={}, unrestaked_receipt_token_amount={}, claimed_supported_token_amount={}, deducted_supported_token_fee_amount={}",
            vault_receipt_token_mint.key,
            payer_vault_supported_token_account_amount,
            unrestaked_receipt_token_amount,
            claimed_supported_token_amount,
            deducted_supported_token_fee_amount
        );

        Ok((
            payer_vault_supported_token_account_amount,
            unrestaked_receipt_token_amount,
            claimed_supported_token_amount,
            deducted_supported_token_fee_amount,
        ))
    }
}
