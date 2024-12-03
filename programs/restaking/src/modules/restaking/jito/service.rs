use crate::constants::*;
use crate::modules::restaking::jito::{
    JitoRestakingVault, JitoRestakingVaultContext, JitoVaultOperatorDelegation,
};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::associated_token;
use anchor_spl::token_interface::TokenAccount;
use jito_bytemuck::AccountDeserialize;
use jito_vault_core::{
    config::Config, vault::Vault, vault_operator_delegation::VaultOperatorDelegation,
    vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket,
    vault_update_state_tracker::VaultUpdateStateTracker,
};

impl JitoRestakingVault {
    pub const VAULT_BASE_ACCOUNT_SEED: &'static [u8] = b"vault_base_account";
    pub const VAULT_WITHDRAWAL_TICKET_SEED: &'static [u8] = b"vault_staker_withdrawal_ticket";
    pub const VAULT_BASE_ACCOUNT1_SEED: &'static [u8] = b"vault_base_account1";
    pub const VAULT_BASE_ACCOUNT2_SEED: &'static [u8] = b"vault_base_account2";
    pub fn get_vault_operator_delegation_key(
        ctx: JitoRestakingVaultContext,
        operator: &Pubkey,
    ) -> Pubkey {
        let (vault_operator_delegation_key, _, _) = VaultOperatorDelegation::find_program_address(
            ctx.vault_program.key,
            ctx.vault.key,
            operator,
        );
        vault_operator_delegation_key
    }

    pub fn get_status(
        vault: AccountInfo,
        vault_operators_delegation: &[AccountInfo],
    ) -> Result<JitoRestakingVault> {
        let vault_data_ref = vault.data.borrow();
        let vault_data = vault_data_ref.as_ref();
        let vault = Vault::try_from_slice_unchecked(vault_data)?;

        // Vault's status
        let vault_receipt_token_withdrawal_pending_amount = vault.vrt_cooling_down_amount();
        let vault_receipt_token_enqueued_for_withdrawal_amount =
            vault.vrt_enqueued_for_cooldown_amount();
        let vault_receipt_token_claimable_amount = vault.vrt_ready_to_claim_amount();
        let vault_receipt_token_unrestake_pending_amount =
            vault_receipt_token_withdrawal_pending_amount
                .checked_add(vault_receipt_token_enqueued_for_withdrawal_amount)
                .unwrap();
        let vault_supported_token_restaked_amount = vault.tokens_deposited();

        // Vault::DelegationState's status
        let vault_delegation_state = vault.delegation_state;
        let supported_token_cooling_down_amount = vault_delegation_state.cooling_down_amount();
        let supported_token_enqueued_for_undelegate_amount =
            vault_delegation_state.enqueued_for_cooldown_amount();
        let supported_token_undelegate_pending_amount = supported_token_cooling_down_amount
            .checked_add(supported_token_enqueued_for_undelegate_amount)
            .unwrap();
        let supported_token_delegated_amount = vault_delegation_state.staked_amount();

        // Check Validation
        let operator_count = vault.operator_count();
        if vault_operators_delegation.len() != operator_count as usize {
            msg!("the number of vault operators delegation does not match the operator count in the vault.");
            return Err(ProgramError::InvalidAccountData.into());
        }

        // VaultOperatorDelegation's status
        let mut vault_operators_delegation_status = Vec::new();
        for vault_operator_delegation in vault_operators_delegation {
            if vault_operator_delegation.owner != &vault.operator_admin {
                msg!("owner and operator admin do not match.");
                return Err(ProgramError::InvalidAccountData.into());
            }
            let vault_operator_delegation_data_ref = vault_operator_delegation.data.borrow();
            let vault_operator_delegation_data = vault_operator_delegation_data_ref.as_ref();
            let vault_operator_delegation =
                VaultOperatorDelegation::try_from_slice_unchecked(vault_operator_delegation_data)?;

            let vault_operator_delegation_state = vault_operator_delegation.delegation_state;
            let operator_supported_token_delegated_amount =
                vault_operator_delegation_state.staked_amount();
            let operator_supported_token_cooling_down_amount =
                vault_operator_delegation_state.cooling_down_amount();
            let operator_supported_token_enqueued_for_undelegate_amount =
                vault_operator_delegation_state.enqueued_for_cooldown_amount();
            let operator_supported_token_undelegate_pending_amount =
                operator_supported_token_cooling_down_amount
                    .checked_add(operator_supported_token_enqueued_for_undelegate_amount)
                    .unwrap();

            vault_operators_delegation_status.push(JitoVaultOperatorDelegation {
                operator_supported_token_delegated_amount,
                operator_supported_token_undelegate_pending_amount,
            });
        }

        Ok(Self {
            vault_receipt_token_claimable_amount,
            vault_receipt_token_unrestake_pending_amount,
            vault_supported_token_restaked_amount,
            supported_token_undelegate_pending_amount,
            supported_token_delegated_amount,
            vault_operators_delegation_status,
        })
    }

    pub fn find_accounts_for_vault(vault: Pubkey) -> Result<Vec<(Pubkey, bool)>> {
        require_eq!(vault, FRAGSOL_JITO_VAULT_ACCOUNT_ADDRESS);
        Ok(vec![
            (JITO_VAULT_PROGRAM_ID, false),
            (FRAGSOL_JITO_VAULT_ACCOUNT_ADDRESS, false),
            (FRAGSOL_JITO_VAULT_CONFIG_ADDRESS, false),
        ])
    }

    pub fn find_accounts_for_restaking_vault(
        fund_account: &AccountInfo,
        jito_vault_program: &AccountInfo,
        jito_vault_account: &AccountInfo,
        jito_vault_config: &AccountInfo,
    ) -> Result<Vec<(Pubkey, bool)>> {
        let vault_data_ref = jito_vault_account.data.borrow();
        let vault_data = vault_data_ref.as_ref();
        let vault = Vault::try_from_slice_unchecked(vault_data)?;

        let vault_vrt_mint = vault.vrt_mint;
        let vault_vst_mint = vault.supported_mint;
        let token_program = anchor_spl::token::ID;
        let system_program = System::id();
        let fund_supported_token_account =
            anchor_spl::associated_token::get_associated_token_address(
                fund_account.key,
                &vault_vst_mint,
            );
        let clock = Clock::get()?;
        let vault_update_state_tracker =
            Self::get_vault_update_state_tracker(jito_vault_config, clock.slot, false)?;

        let vault_update_state_tracker_prepare_for_delaying =
            Self::get_vault_update_state_tracker(jito_vault_config, clock.slot, true)?;

        let vault_fee_wallet_token_account =
            associated_token::get_associated_token_address_with_program_id(
                &ADMIN_PUBKEY,
                &vault_vrt_mint,
                &token_program,
            );
        let fund_receipt_token_account =
            associated_token::get_associated_token_address_with_program_id(
                &ADMIN_PUBKEY,
                &fund_account.key(),
                &token_program,
            );

        Ok(vec![
            (*jito_vault_program.key, false),
            (*jito_vault_account.key, false),
            (*jito_vault_config.key, false),
            (vault_update_state_tracker.key(), false),
            (vault_update_state_tracker_prepare_for_delaying.key(), false),
            (vault_vrt_mint, false),
            (vault_vst_mint, false),
            (fund_supported_token_account, false),
            (fund_receipt_token_account, false),
            (vault_fee_wallet_token_account, false),
            (token_program, false),
            (system_program, false),
        ])
    }

    pub fn find_accounts_for_unstaking_vault(
        fund_account: &AccountInfo,
        jito_vault_program: &AccountInfo,
        jito_vault_account: &AccountInfo,
        jito_vault_config: &AccountInfo,
    ) -> Result<Vec<(Pubkey, bool)>> {
        let vault_data_ref = jito_vault_account.data.borrow();
        let vault_data = vault_data_ref.as_ref();
        let vault = Vault::try_from_slice_unchecked(vault_data)?;

        let vault_vrt_mint = vault.vrt_mint;
        let vault_vst_mint = vault.supported_mint;
        let token_program = anchor_spl::token::ID;
        let system_program = System::id();
        let fund_supported_token_account =
            anchor_spl::associated_token::get_associated_token_address(
                fund_account.key,
                &vault_vst_mint,
            );
        let clock = Clock::get()?;
        let vault_update_state_tracker =
            Self::get_vault_update_state_tracker(jito_vault_config, clock.slot, false)?;

        let vault_update_state_tracker_prepare_for_delaying =
            Self::get_vault_update_state_tracker(jito_vault_config, clock.slot, true)?;

        let vault_fee_wallet_token_account =
            associated_token::get_associated_token_address_with_program_id(
                &ADMIN_PUBKEY,
                &vault_vrt_mint,
                &token_program,
            );
        let fund_receipt_token_account =
            associated_token::get_associated_token_address_with_program_id(
                &ADMIN_PUBKEY,
                &fund_account.key(),
                &token_program,
            );
        let vault_supported_token_account =
            associated_token::get_associated_token_address_with_program_id(
                &jito_vault_account.key(),
                &vault.supported_mint,
                &token_program,
            );

        Ok(vec![
            (*jito_vault_program.key, false),
            (*jito_vault_account.key, false),
            (*jito_vault_config.key, false),
            (vault_update_state_tracker.key(), false),
            (vault_update_state_tracker_prepare_for_delaying.key(), false),
            (vault_vrt_mint, false),
            (vault_vst_mint, false),
            (fund_supported_token_account, false),
            (fund_receipt_token_account, false),
            (vault_supported_token_account, false),
            (vault_fee_wallet_token_account, false),
            (token_program, false),
            (system_program, false),
        ])
    }

    pub fn get_vault_update_state_tracker(
        jito_vault_config: &AccountInfo,
        current_slot: u64,
        delayed: bool,
    ) -> Result<Pubkey> {
        let data = jito_vault_config
            .try_borrow_data()
            .map_err(|e| Error::from(e).with_account_name("jito_vault_update_state_tracker"))?;
        let config = Config::try_from_slice_unchecked(&data)
            .map_err(|e| Error::from(e).with_account_name("jito_vault_update_state_tracker"))?;
        let mut ncn_epoch = current_slot
            .checked_div(config.epoch_length())
            .ok_or_else(|| error!(crate::errors::ErrorCode::CalculationArithmeticException))?;

        if delayed {
            ncn_epoch += 1
        }

        let (vault_update_state_tracker, _) = Pubkey::find_program_address(
            &[
                b"vault_update_state_tracker",
                FRAGSOL_JITO_VAULT_ACCOUNT_ADDRESS.as_ref(),
                &ncn_epoch.to_le_bytes(),
            ],
            &JITO_VAULT_PROGRAM_ID,
        );

        Ok(vault_update_state_tracker)
    }

    pub fn find_current_vault_update_state_tracker<'info>(
        vault_staker_withdrawal_ticket: &'info AccountInfo<'info>,
        next_vault_staker_withdrawal_ticket: &'info AccountInfo<'info>,
    ) -> Result<&'info AccountInfo<'info>> {
        let data = vault_staker_withdrawal_ticket
            .try_borrow_data()
            .map_err(|e| Error::from(e).with_account_name("jito_vault_update_state_tracker"))?;
        let tracker = VaultUpdateStateTracker::try_from_slice_unchecked(&data)
            .map_err(|e| Error::from(e).with_account_name("jito_vault_update_state_tracker"))?;

        if Clock::get()?.epoch == tracker.ncn_epoch() {
            Ok(vault_staker_withdrawal_ticket)
        } else {
            Ok(next_vault_staker_withdrawal_ticket)
        }
    }

    pub fn find_withdrawal_tickets() -> Vec<(Pubkey, bool)> {
        const BASE_ACCOUNTS_LENGTH: u8 = 5;
        let mut withdrawal_tickets = Vec::with_capacity(BASE_ACCOUNTS_LENGTH as usize);

        for i in 1..=BASE_ACCOUNTS_LENGTH {
            withdrawal_tickets.push((
                Self::find_withdrawal_ticket_account(&Self::find_vault_base_account(i)),
                false,
            ));
        }
        withdrawal_tickets
    }

    pub fn find_vault_base_account(index: u8) -> Pubkey {
        let base = Pubkey::find_program_address(
            &[
                Self::VAULT_BASE_ACCOUNT_SEED,
                (FRAGSOL_MINT_ADDRESS as Pubkey).as_ref(),
                &[index],
            ],
            &JITO_VAULT_PROGRAM_ID,
        )
        .0;
        base
    }
    pub fn find_withdrawal_ticket_account(base: &Pubkey) -> Pubkey {
        let vault_account: Pubkey = FRAGSOL_JITO_VAULT_ACCOUNT_ADDRESS;
        let withdrawal_ticket_account = Pubkey::find_program_address(
            &[
                Self::VAULT_WITHDRAWAL_TICKET_SEED,
                vault_account.as_ref(),
                base.as_ref(),
            ],
            &JITO_VAULT_PROGRAM_ID,
        );
        withdrawal_ticket_account.0
    }

    pub fn find_withdrawal_ticket_token_account(withdrawal_ticket_account: &Pubkey) -> Pubkey {
        let withdrawal_ticket_token_account = associated_token::get_associated_token_address(
            &withdrawal_ticket_account,
            &NSOL_MINT_ADDRESS,
        );
        withdrawal_ticket_token_account
    }
}

pub fn deposit<'info>(
    ctx: &JitoRestakingVaultContext<'info>,
    supported_token_account: &AccountInfo<'info>,
    supported_token_amount_in: u64,
    vault_fee_receipt_token_account: &AccountInfo<'info>,
    vault_receipt_token_account: &'info AccountInfo<'info>,
    vault_receipt_token_min_amount_out: u64,
    signer: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
) -> Result<u64> {
    let mut vault_receipt_token_account_parsed =
        InterfaceAccount::<TokenAccount>::try_from(vault_receipt_token_account)?;
    let vault_receipt_token_account_amount_before = vault_receipt_token_account_parsed.amount;

    let mint_to_ix = jito_vault_sdk::sdk::mint_to(
        ctx.vault_program.key,
        ctx.vault_config.key,
        ctx.vault.key,
        ctx.vault_receipt_token_mint.key,
        signer.key,
        supported_token_account.key,
        ctx.vault_supported_token_account.key,
        vault_receipt_token_account.key,
        // TODO: read fee admin ata from vault state account
        vault_fee_receipt_token_account.key,
        None,
        supported_token_amount_in,
        vault_receipt_token_min_amount_out,
    );

    // ctx.vault_supported_token_account.

    invoke_signed(
        &mint_to_ix,
        &[
            ctx.vault_program.clone(),
            ctx.vault_config.clone(),
            ctx.vault.clone(),
            ctx.vault_receipt_token_mint.clone(),
            signer.clone(),
            supported_token_account.clone(),
            ctx.vault_supported_token_account.clone(),
            vault_receipt_token_account.clone(),
            vault_fee_receipt_token_account.clone(),
            ctx.vault_receipt_token_program.clone(),
        ],
        signer_seeds,
    )?;

    vault_receipt_token_account_parsed.reload()?;
    let vault_receipt_token_account_amount = vault_receipt_token_account_parsed.amount;
    let minted_vrt_amount =
        vault_receipt_token_account_amount - vault_receipt_token_account_amount_before;

    require_gte!(minted_vrt_amount, supported_token_amount_in);

    Ok(minted_vrt_amount)
}

pub fn delegation<'info>(
    ctx: &JitoRestakingVaultContext<'info>,
    vault_operator: &AccountInfo<'info>,
    vault_operator_delegation: &AccountInfo<'info>,
    signer: &AccountInfo<'info>, // jito_vault_delegation_admin
    signer_seeds: &[&[&[u8]]],
    supported_token_delegation_amount_in: u64,
) -> Result<()> {
    let add_delegation_ix = jito_vault_sdk::sdk::add_delegation(
        ctx.vault_program.key,
        ctx.vault_config.key,
        ctx.vault.key,
        vault_operator.key,
        vault_operator_delegation.key,
        signer.key,
        supported_token_delegation_amount_in,
    );

    invoke_signed(
        &add_delegation_ix,
        &[
            ctx.vault_program.clone(),
            ctx.vault_config.clone(),
            ctx.vault.clone(),
            vault_operator.clone(),
            vault_operator_delegation.clone(),
            signer.clone(),
        ],
        signer_seeds,
    )?;

    Ok(())
}

pub fn undelegation<'info>(
    ctx: &JitoRestakingVaultContext<'info>,
    vault_operator: &AccountInfo<'info>,
    vault_operator_delegation: &AccountInfo<'info>,
    signer: &AccountInfo<'info>, // jito_vault_delegation_admin
    signer_seeds: &[&[&[u8]]],
    supported_token_delegation_amount_out: u64,
) -> Result<()> {
    let cooldown_delegation_ix = jito_vault_sdk::sdk::cooldown_delegation(
        ctx.vault_program.key,
        ctx.vault_config.key,
        ctx.vault.key,
        vault_operator.key,
        vault_operator_delegation.key,
        signer.key,
        supported_token_delegation_amount_out,
    );

    invoke_signed(
        &cooldown_delegation_ix,
        &[
            ctx.vault_program.clone(),
            ctx.vault_config.clone(),
            ctx.vault.clone(),
            vault_operator.clone(),
            vault_operator_delegation.clone(),
            signer.clone(),
        ],
        signer_seeds,
    )?;

    Ok(())
}

pub fn request_withdraw<'info>(
    ctx: &JitoRestakingVaultContext<'info>,
    operator: &Signer<'info>,
    vault_withdrawal_ticket: &AccountInfo<'info>,
    vault_withdrawal_ticket_token_account: &AccountInfo<'info>,
    vault_receipt_token_account: &AccountInfo<'info>,
    vault_base_account: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    associated_token_program: &AccountInfo<'info>,
    signer: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    vrt_token_amount_out: u64,
) -> Result<()> {
    anchor_spl::associated_token::create(CpiContext::new(
        associated_token_program.clone(),
        anchor_spl::associated_token::Create {
            payer: operator.to_account_info(),
            associated_token: vault_withdrawal_ticket_token_account.clone(),
            authority: vault_withdrawal_ticket.clone(),
            mint: ctx.vault_receipt_token_mint.clone(),
            system_program: system_program.clone(),
            token_program: ctx.vault_receipt_token_program.clone(),
        },
    ))?;

    let rent = Rent::get()?;
    let current_lamports = vault_withdrawal_ticket.lamports();
    let space = 8 + std::mem::size_of::<VaultStakerWithdrawalTicket>();
    let required_lamports = rent
        .minimum_balance(space)
        .max(1)
        .saturating_sub(current_lamports);

    if required_lamports > 0 {
        anchor_lang::system_program::transfer(
            CpiContext::new(
                system_program.clone(),
                anchor_lang::system_program::Transfer {
                    from: operator.to_account_info(),
                    to: vault_withdrawal_ticket.clone(),
                },
            ),
            required_lamports,
        )?;
    }

    let enqueue_withdraw_ix = jito_vault_sdk::sdk::enqueue_withdrawal(
        ctx.vault_program.key,
        ctx.vault_config.key,
        ctx.vault.key,
        vault_withdrawal_ticket.key,
        vault_withdrawal_ticket_token_account.key,
        signer.key,
        vault_receipt_token_account.key,
        vault_base_account.key,
        vrt_token_amount_out,
    );

    invoke_signed(
        &enqueue_withdraw_ix,
        &[
            ctx.vault_program.clone(),
            ctx.vault_config.clone(),
            ctx.vault.clone(),
            vault_withdrawal_ticket.clone(),
            vault_withdrawal_ticket_token_account.clone(),
            signer.clone(),
            vault_receipt_token_account.clone(),
            vault_base_account.clone(),
            ctx.vault_receipt_token_program.clone(),
            system_program.clone(),
        ],
        signer_seeds,
    )?;

    Ok(())
}

pub fn withdraw<'info>(
    ctx: &JitoRestakingVaultContext<'info>,
    vault_withdrawal_ticket: &AccountInfo<'info>,
    vault_withdrawal_ticket_token_account: &AccountInfo<'info>,
    fund_vault_supported_token_account: &AccountInfo<'info>,
    vault_fee_receipt_token_account: &AccountInfo<'info>,
    vault_program_fee_wallet_vrt_account: &AccountInfo<'info>,
    signer: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
) -> Result<()> {
    let enqueue_withdraw_ix = jito_vault_sdk::sdk::burn_withdrawal_ticket(
        ctx.vault_program.key,
        ctx.vault_config.key,
        ctx.vault.key,
        ctx.vault_supported_token_account.key,
        ctx.vault_receipt_token_mint.key,
        signer.key,
        fund_vault_supported_token_account.key,
        vault_withdrawal_ticket.key,
        vault_withdrawal_ticket_token_account.key,
        vault_fee_receipt_token_account.key,
        vault_program_fee_wallet_vrt_account.key,
    );

    invoke_signed(
        &enqueue_withdraw_ix,
        &[
            ctx.vault_program.clone(),
            ctx.vault_config.clone(),
            ctx.vault.clone(),
            ctx.vault_supported_token_account.clone(),
            ctx.vault_receipt_token_mint.clone(),
            signer.clone(),
            fund_vault_supported_token_account.clone(),
            vault_withdrawal_ticket.clone(),
            vault_withdrawal_ticket_token_account.clone(),
            vault_fee_receipt_token_account.clone(),
            vault_program_fee_wallet_vrt_account.clone(),
            ctx.vault_receipt_token_program.clone(),
            system_program.clone(),
        ],
        &[],
    )?;

    Ok(())
}
