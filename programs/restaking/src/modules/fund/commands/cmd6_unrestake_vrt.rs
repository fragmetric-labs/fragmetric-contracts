use super::{ClaimUnrestakedVSTCommand, ClaimUnrestakedVSTCommandState, ClaimUnstakedSOLCommand, OperationCommand, OperationCommandContext, OperationCommandEntry, OperationCommandResult, SelfExecutable, UnstakeLSTCommandItem};
use crate::errors;
use crate::modules::fund::{FundService, WeightedAllocationParticipant, WeightedAllocationStrategy, FUND_ACCOUNT_MAX_SUPPORTED_TOKENS};
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

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct UnrestakeVSTCommandItem {
    vault_address: Pubkey,
    sol_amount: u64,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum UnrestakeVRTCommandState {
    #[default]
    New,
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
        match &self.state {
            UnrestakeVRTCommandState::New => {
                let pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                    .new_pricing_service(accounts.iter().cloned())?;
                let fund_account = ctx.fund_account.load()?;
                let unstaking_obligated_amount_as_sol = fund_account.get_total_unstaking_obligated_amount_as_sol(&pricing_service)?;

                // 1. 일단 출금 가능한 토큰들 먼저.. 출금 자체에 unrestaking이 필요하니 모르니 일단 각각 별도 수량 계산.. 볼트끼리 전략 돌려서 cut 해버려 (nSOL 볼트 포함)
                // 2. SOL 출금은? LST 쓰는 전체 볼트들 모아서 아까 컷한거 반영한 상태에 추가로 컷해버려..


                let sol_unstaking_obligated_amount = {
                    let sol_net_operation_reserved_amount =
                        fund_account.get_asset_net_operation_reserved_amount(None, true, &pricing_service)?;
                    if sol_net_operation_reserved_amount.is_negative() {
                        u64::try_from(-sol_net_operation_reserved_amount)?
                            .saturating_sub(
                                fund_account
                                    .get_supported_tokens_iter()
                                    .map(|supported_token| supported_token.pending_unstaking_amount_as_sol)
                                    .sum(),
                            )?
                    } else {
                        0
                    }
                };
                if sol_net_operation_reserved_amount.is_negative() {
                    let sol_unstaking_obligated_amount = u64::try_from(-sol_net_operation_reserved_amount)?
                        .saturating_sub(
                            fund_account
                                .get_supported_tokens_iter()
                                .map(|supported_token| supported_token.pending_unstaking_amount_as_sol)
                                .sum(),
                        );
                    let mut strategy = WeightedAllocationStrategy::<crate::modules::fund::fund_account::FUND_ACCOUNT_MAX_SUPPORTED_TOKENS>::new(
                        fund_account
                            .get_supported_tokens_iter()
                            .map(|supported_token| {
                                match supported_token.pricing_source.try_deserialize()? {
                                    Some(TokenPricingSource::SPLStakePool { .. })
                                    | Some(TokenPricingSource::MarinadeStakePool { .. })
                                    | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool {
                                               ..
                                           }) => Ok(WeightedAllocationParticipant::new(
                                        supported_token.sol_allocation_weight,
                                        pricing_service.get_token_amount_as_sol(
                                            &supported_token.mint,
                                            u64::try_from(
                                                fund_account
                                                    .get_asset_net_operation_reserved_amount(
                                                        Some(supported_token.mint),
                                                        false,
                                                        &pricing_service,
                                                    )?
                                                    .max(0),
                                            )?,
                                        )?,
                                        supported_token.sol_allocation_capacity_amount,
                                    )),
                                    // fail when supported token is not unstakable
                                    Some(TokenPricingSource::JitoRestakingVault { .. })
                                    | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                                    | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                                    | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                                    | None => {
                                        err!(ErrorCode::FundOperationCommandExecutionFailedException)?
                                    }
                                    #[cfg(all(test, not(feature = "idl-build")))]
                                    Some(TokenPricingSource::Mock { .. }) => {
                                        err!(ErrorCode::FundOperationCommandExecutionFailedException)?
                                    }
                                }
                            })
                            .collect::<Result<Vec<_>>>()?,
                    );
                    strategy.cut_greedy(sol_unstaking_obligated_amount)?;

                    let mut items = Vec::with_capacity(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS);
                    for (index, supported_token) in fund_account.get_supported_tokens_iter().enumerate() {
                        let allocated_sol_amount =
                            strategy.get_participant_last_cut_amount_by_index(index)?;
                        let allocated_token_amount = pricing_service
                            .get_sol_amount_as_token(&supported_token.mint, allocated_sol_amount)?;
                        if allocated_token_amount > 0 {
                            items.push(UnstakeLSTCommandItem {
                                token_mint: supported_token.mint,
                                allocated_token_amount,
                            });
                        }
                    }
                    drop(fund_account);

                    self.execute_prepare(ctx, accounts, items, None)
                } else {
                    Ok((None, None))
                }
            }
            _ => {},
        };

        if let Some(item) = self.items.first() {
            match &self.state {
                UnrestakeVRTCommandState::New => {

                },
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
