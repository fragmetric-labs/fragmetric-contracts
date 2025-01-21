use super::{ClaimUnrestakedVSTCommand, ClaimUnrestakedVSTCommandState, ClaimUnstakedSOLCommand, OperationCommand, OperationCommandContext, OperationCommandEntry, OperationCommandResult, SelfExecutable};
use crate::errors;
use crate::modules::fund::FundService;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::JitoRestakingVaultService;
use crate::utils::PDASeeds;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account;
use jito_bytemuck::AccountDeserialize;
use jito_vault_core::vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct UnrestakeVRTCommand {
    #[max_len(2)]
    items: Vec<UnrestakeVSTCommandItem>,
    state: UnrestakeVRTCommandState,
}

impl UnrestakeVRTCommand {
    pub(super) fn new_init(items: Vec<UnrestakeVSTCommandItem>) -> Self {
        Self {
            items,
            state: UnrestakeVRTCommandState::Init,
        }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct UnrestakeVSTCommandItem {
    vault_address: Pubkey,
    sol_amount: u64,
}

impl UnrestakeVSTCommandItem {
    pub(super) fn new(vault_address: Pubkey, sol_amount: u64) -> Self {
        Self {
            vault_address,
            sol_amount,
        }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum UnrestakeVRTCommandState {
    #[default]
    Init,
    ReadVaultState,
    Unstake(#[max_len(4, 32)] Vec<Vec<u8>>),
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct UnrestakeVRTCommandResult {}

impl SelfExecutable for UnrestakeVRTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if let Some(item) = self.items.first() {
            match &self.state {
                UnrestakeVRTCommandState::Init if item.sol_amount > 0 => {
                    let mut command = self.clone();
                    command.state = UnrestakeVRTCommandState::ReadVaultState;

                    let fund_accout_ref = ctx.fund_account.load()?;
                    let restaking_vault =
                        fund_accout_ref.get_restaking_vault(&item.vault_address)?;
                    match restaking_vault
                        .receipt_token_pricing_source
                        .try_deserialize()?
                    {
                        Some(TokenPricingSource::JitoRestakingVault { address }) => {
                            let required_accounts =
                                &mut JitoRestakingVaultService::find_accounts_to_new(address)?;
                            required_accounts.append(
                                &mut JitoRestakingVaultService::find_withdrawal_tickets(
                                    &restaking_vault.vault,
                                    &ctx.receipt_token_mint.key(),
                                ),
                            );
                            return Ok((
                                None,
                                Some(command.with_required_accounts(required_accounts.to_vec())),
                            ));
                        }
                        // otherwise fails
                        Some(TokenPricingSource::SPLStakePool { .. })
                        | Some(TokenPricingSource::MarinadeStakePool { .. })
                        | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
                        | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                        | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                        | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                        | None => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                        #[cfg(all(test, not(feature = "idl-build")))]
                        Some(TokenPricingSource::Mock { .. }) => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                    };
                }
                UnrestakeVRTCommandState::ReadVaultState => {
                    let fund_accout_ref = ctx.fund_account.load()?;
                    let restaking_vault =
                        fund_accout_ref.get_restaking_vault(&item.vault_address)?;

                    match restaking_vault
                        .receipt_token_pricing_source
                        .try_deserialize()?
                    {
                        Some(TokenPricingSource::JitoRestakingVault { address }) => {
                            require_keys_eq!(address, restaking_vault.vault);

                            let [jito_vault_program, jito_vault_config, jito_vault_account, remaining_accounts @ ..] =
                                accounts
                            else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };
                            let withdrawal_tickets = &remaining_accounts[..5];

                            let _remaining_accounts = &remaining_accounts[5..];

                            let mut _withdrawal_ticket_position = 0;
                            let mut ticket_set: (Pubkey, Pubkey, Pubkey) =
                                (Pubkey::default(), Pubkey::default(), Pubkey::default());
                            let mut signer_seed = vec![];

                            for (i, withdrawal_ticket) in withdrawal_tickets.iter().enumerate() {
                                if JitoRestakingVaultService::check_withdrawal_ticket_is_empty(
                                    &withdrawal_ticket,
                                )? {
                                    let ticket_token_account = JitoRestakingVaultService::find_withdrawal_ticket_token_account(&withdrawal_ticket.key(), &restaking_vault.receipt_token_mint, &restaking_vault.receipt_token_program);
                                    _withdrawal_ticket_position = i as u8;
                                    ticket_set = (
                                        JitoRestakingVaultService::find_vault_base_account(
                                            &ctx.receipt_token_mint.key(),
                                            i as u8,
                                        )
                                        .0,
                                        withdrawal_ticket.key(),
                                        ticket_token_account,
                                    );
                                    let (_, base_account_bump) =
                                        JitoRestakingVaultService::find_vault_base_account(
                                            &ctx.receipt_token_mint.key(),
                                            i as u8,
                                        );

                                    // signer_seed.push(
                                    //     JitoRestakingVaultService::VAULT_BASE_ACCOUNT_SEED.to_vec(),
                                    // );
                                    signer_seed
                                        .push(ctx.receipt_token_mint.key().as_ref().to_vec());
                                    signer_seed.push(vec![i as u8]);
                                    signer_seed.push(vec![base_account_bump]);
                                    break;
                                }
                            }
                            if ticket_set.0 == Pubkey::default() {
                                err!(errors::ErrorCode::RestakingVaultWithdrawalTicketsExhaustedError)?
                            }
                            let system_program = System::id();
                            let fund_receipt_token_account =
                                spl_associated_token_account::get_associated_token_address(
                                    &ctx.fund_account.key(),
                                    &restaking_vault.receipt_token_mint,
                                );
                            let mut required_accounts =
                                JitoRestakingVaultService::find_initialize_vault_accounts(
                                    jito_vault_program,
                                    jito_vault_config,
                                    jito_vault_account,
                                )?;
                            required_accounts.append(&mut vec![
                                (ticket_set.0, false),
                                (ticket_set.1, true),
                                (ticket_set.2, true),
                                (fund_receipt_token_account, true),
                                (anchor_spl::associated_token::ID, false),
                                (system_program, false),
                            ]);

                            let mut command = self.clone();
                            command.state = UnrestakeVRTCommandState::Unstake(signer_seed);
                            return Ok((
                                None,
                                Some(command.with_required_accounts(required_accounts)),
                            ));
                        }
                        // otherwise fails
                        Some(TokenPricingSource::SPLStakePool { .. })
                        | Some(TokenPricingSource::MarinadeStakePool { .. })
                        | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
                        | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                        | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                        | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                        | None => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                        #[cfg(all(test, not(feature = "idl-build")))]
                        Some(TokenPricingSource::Mock { .. }) => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                    };
                }
                UnrestakeVRTCommandState::Unstake(raw_signer_seed) => {
                    let [vault_program, vault_config, vault_account, vault_receipt_token_mint, vault_receipt_token_program, vault_supported_token_mint, vault_supported_token_program, vault_supported_token_account, base_account, withdrawal_ticket_account, withdrawal_ticket_token_account, fund_receipt_token_account, associated_token_program, system_program, remaining_accounts @ ..] =
                        accounts
                    else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };
                    let mut pricing_source = remaining_accounts.to_vec();
                    pricing_source.push(vault_account);
                    let pricing_service =
                        FundService::new(&mut ctx.receipt_token_mint, &mut ctx.fund_account)?
                            .new_pricing_service(pricing_source)?;

                    let need_to_withdraw_token_amount = pricing_service.get_sol_amount_as_token(
                        &vault_receipt_token_mint.key(),
                        item.sol_amount,
                    )?;
                    let signer_seed: Vec<&[u8]> = raw_signer_seed
                        .iter()
                        .map(|inner_vec| inner_vec.as_slice())
                        .collect();

                    // JitoRestakingVaultService::new(
                    //     vault_program.to_account_info(),
                    //     vault_config.to_account_info(),
                    //     vault_account.to_account_info(),
                    //     vault_receipt_token_mint.to_account_info(),
                    //     vault_receipt_token_program.to_account_info(),
                    //     vault_supported_token_mint.to_account_info(),
                    //     vault_supported_token_program.to_account_info(),
                    //     vault_supported_token_account.to_account_info(),
                    // )?
                    // .request_withdraw(
                    //     &ctx.operator,
                    //     withdrawal_ticket_account,
                    //     withdrawal_ticket_token_account,
                    //     fund_receipt_token_account,
                    //     base_account,
                    //     associated_token_program,
                    //     system_program,
                    //     &ctx.fund_account.to_account_info(),
                    //     &[
                    //         ctx.fund_account.load()?.get_seeds().as_ref(),
                    //         signer_seed.as_slice(),
                    //     ],
                    //     need_to_withdraw_token_amount,
                    // )?;
                }
                _ => (),
            }
        }
        Ok((None, Some(ClaimUnstakedSOLCommand::default().without_required_accounts())))
    }
}
