use super::{
    ClaimUnrestakedVSTCommand, ClaimUnrestakedVSTCommandState, OperationCommand,
    OperationCommandContext, OperationCommandEntry, SelfExecutable,
};
use crate::constants::FRAGSOL_MINT_ADDRESS;
use crate::errors;
use crate::modules::fund::FundService;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::jito::JitoRestakingVault;
use crate::modules::restaking::JitoRestakingVaultService;
use crate::utils::PDASeeds;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account;
use jito_bytemuck::AccountDeserialize;
use jito_vault_core::vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct UnrestakeVRTCommand {
    #[max_len(2)]
    items: Vec<UnrestakeVSTCommandItem>,
    state: UnrestakeVRTCommandState,
}

impl From<UnrestakeVRTCommand> for OperationCommand {
    fn from(command: UnrestakeVRTCommand) -> Self {
        Self::UnrestakeVRT(command)
    }
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

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum UnrestakeVRTCommandState {
    Init,
    ReadVaultState,
    Unstake(#[max_len(4, 32)] Vec<Vec<u8>>),
}

impl SelfExecutable for UnrestakeVRTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        if let Some(item) = self.items.first() {
            let mut func_account = ctx.fund_account.clone();
            let restaking_vault = func_account.get_restaking_vault_mut(&item.vault_address)?;

            match &self.state {
                UnrestakeVRTCommandState::Init if item.sol_amount > 0 => {
                    let mut command = self.clone();
                    command.state = UnrestakeVRTCommandState::ReadVaultState;
                    match restaking_vault.receipt_token_pricing_source {
                        TokenPricingSource::JitoRestakingVault { address } => {
                            let required_accounts =
                                &mut JitoRestakingVaultService::find_accounts_for_vault(address)?;
                            required_accounts
                                .append(&mut JitoRestakingVaultService::find_withdrawal_tickets());
                            return Ok(Some(
                                command.with_required_accounts(required_accounts.to_vec()),
                            ));
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    };
                }
                UnrestakeVRTCommandState::ReadVaultState => {
                    match restaking_vault.receipt_token_pricing_source {
                        TokenPricingSource::JitoRestakingVault { address } => {
                            let [jito_vault_program, jito_vault_account, jito_vault_config, remaining_accounts @ ..] =
                                accounts
                            else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };
                            let withdrawal_tickets = &remaining_accounts[..5] else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };

                            let remaining_accounts = &remaining_accounts[5..] else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };

                            let mut withdrawal_ticket_position = 0;
                            let mut ticket_set: (Pubkey, Pubkey, Pubkey) =
                                (Pubkey::default(), Pubkey::default(), Pubkey::default());
                            let mut signer_seed = vec![];

                            for (i, withdrawal_ticket) in withdrawal_tickets.iter().enumerate() {
                                if JitoRestakingVaultService::check_withdrawal_ticket_is_empty(
                                    &withdrawal_ticket,
                                )? {
                                    let ticket_token_account = JitoRestakingVaultService::find_withdrawal_ticket_token_account(&withdrawal_ticket.key(), &restaking_vault.receipt_token_mint, &restaking_vault.receipt_token_program);
                                    withdrawal_ticket_position = i as u8;
                                    ticket_set = (
                                        JitoRestakingVaultService::find_vault_base_account(i as u8)
                                            .0,
                                        withdrawal_ticket.key(),
                                        ticket_token_account,
                                    );
                                    let (_, base_account_bump) =
                                        JitoRestakingVaultService::find_vault_base_account(i as u8);
                                    
                                    signer_seed.push(JitoRestakingVaultService::VAULT_BASE_ACCOUNT_SEED.to_vec());
                                    signer_seed.push(ctx.receipt_token_mint.key().as_ref().to_vec());
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
                            command.state =
                                UnrestakeVRTCommandState::Unstake(signer_seed);
                            return Ok(Some(command.with_required_accounts(required_accounts)));
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    };
                }
                UnrestakeVRTCommandState::Unstake(raw_signer_seed) => {
                    let [vault_program,  vault_config, vault_account,vault_receipt_token_mint, vault_receipt_token_program, vault_supported_token_mint, vault_supported_token_program, vault_supported_token_account, base_account, withdrawal_ticket_account, withdrawal_ticket_token_account, fund_receipt_token_account, associated_token_program, system_program, remaining_accounts @ ..] =
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
                    let signer_seed = raw_signer_seed.to_vec();
                    let signer_seed: Vec<&[u8]> = raw_signer_seed.iter().map(|inner_vec| inner_vec.as_slice()).collect();

                    JitoRestakingVaultService::new(
                        vault_program.to_account_info(),
                        vault_config.to_account_info(),
                        vault_account.to_account_info(),
                        vault_receipt_token_mint.to_account_info(),
                        vault_receipt_token_program.to_account_info(),
                        vault_supported_token_mint.to_account_info(),
                        vault_supported_token_program.to_account_info(),
                        vault_supported_token_account.to_account_info(),
                    )?
                    .request_withdraw(
                        &ctx.operator,
                        withdrawal_ticket_account,
                        withdrawal_ticket_token_account,
                        fund_receipt_token_account,
                        base_account,
                        associated_token_program,
                        system_program,
                        &ctx.fund_account.as_ref(),
                        &[
                            ctx.fund_account.get_seeds().as_ref(),
                            signer_seed.as_slice()
                        ],
                        need_to_withdraw_token_amount,
                    )?;
                }
                _ => (),
            }
        }
        Ok(None)
    }
}
