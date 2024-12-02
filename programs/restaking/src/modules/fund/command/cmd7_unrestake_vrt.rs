use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};
use crate::modules::restaking::jito::JitoRestakingVault;
use anchor_lang::prelude::*;

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
    vault_mint: Pubkey,
    need_to_unrestake_sol_amount: u64,
}

impl UnrestakeVSTCommandItem {
    pub(super) fn new(vault_mint: Pubkey, need_to_unrestake_sol_amount: u64) -> Self {
        Self {
            vault_mint,
            need_to_unrestake_sol_amount,
        }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum UnrestakeVRTCommandState {
    Init,
    ReadVaultState,
    Unstake,
}

impl SelfExecutable for UnrestakeVRTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        if let Some(item) = self.items.first() {
            let vault = ctx.fund_account.get_restaking_vault_mut(&item.vault_mint)?;

            match &self.state {
                // UnrestakeVRTCommandState::Init if item.need_to_unrestake_vrt_amount > 0 => {
                UnrestakeVRTCommandState::Init => {
                    // TODO: convert sol to vrt
                    let mut command = self.clone();
                    command.state = UnrestakeVRTCommandState::ReadVaultState;
                    return Ok(Some(
                        command.with_required_accounts(
                            []
                                // vec![
                                //     JitoRestakingVault::find_accounts_for_vault().as_slice(),
                                //     JitoRestakingVault::find_withdrawal_tickets().as_slice(),
                                // ]
                                // .concat(),
                        ),
                    ));
                }
                UnrestakeVRTCommandState::ReadVaultState => {
                    let [jito_vault_program, jito_vault_account, jito_vault_config, withdawal_ticket_account1, withdawal_ticket_account2, withdawal_ticket_account3, withdawal_ticket_account4, withdawal_ticket_account5, remaining_accounts @ ..] =
                        accounts
                    else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    let mut command = self.clone();
                    command.state = UnrestakeVRTCommandState::Unstake;
                }
                UnrestakeVRTCommandState::Unstake => {}
                _ => (),
            }
        }
        Ok(None)
    }
}
