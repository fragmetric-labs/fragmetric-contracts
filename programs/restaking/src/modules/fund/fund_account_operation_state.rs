use std::mem::discriminant;
use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use crate::entry;

use super::command::*;

const OPERATION_COMMANDS_EXPIRATION_SECONDS: i64 = 600;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug)]
#[repr(C, align(16))]
pub(super) struct OperationState {
    updated_at: i64,
    expired_at: i64,

    _padding: [u8; 13],
    /// when the no_transition flag turned on, current command should not be transitioned to other command.
    /// the purpose of this flag is for internal testing by set boundary of the reset command operation.
    no_transition: u8,
    pub next_sequence: u16,

    next_command: OperationCommandEntryPod,
    _padding2: [u8; 8],

    _reserved: [u8; 128],
}

impl Default for OperationState {
    fn default() -> Self {
        Self::zeroed()
    }
}

impl OperationState {
    /// Initialize current operation command to `reset_command` or default.
    pub fn initialize_command_if_needed(
        &mut self,
        current_timestamp: i64,
        reset_command: Option<OperationCommandEntry>,
    ) -> Result<()> {
        let has_reset_command = reset_command.is_some();

        if has_reset_command
            || self.next_command.is_none()
            || current_timestamp > self.expired_at
        {
            self.no_transition = 0;
            self.next_sequence = 0;
            self.set_command(
                reset_command.or_else(|| Some(InitializeCommand {}.with_required_accounts([]))),
                current_timestamp,
            );
            self.no_transition = if has_reset_command { 1 } else { 0 };
        }

        Ok(())
    }

    /// Sets next operation command and increment sequence number.
    pub fn set_command(
        &mut self,
        mut command: Option<OperationCommandEntry>,
        current_timestamp: i64,
    ) {
        // deal with no_transition state, to adjust next command.
        if self.no_transition == 1 {
            let next_command: Option<OperationCommandEntry> = self.next_command.into();
            if let (Some(prev_entry), Some(next_entry)) = (next_command, &command) {
                if discriminant(&prev_entry.command) != discriminant(&next_entry.command) {
                    // when the type of the command changes on no_transition state, ignore the next command and clear no_transition state.
                    msg!(
                        "COMMAND#{} reset due to no_transition state",
                        self.next_sequence
                    );
                    self.no_transition = 0;
                    command = None;
                }
                // otherwise, retaining on the same command type, still maintains no_transition state.
            } else {
                // if there is no previous command (unexpected flow) or next command, clear no_transition state.
                self.no_transition = 0;
            }
        }

        self.updated_at = current_timestamp;
        self.expired_at = current_timestamp + OPERATION_COMMANDS_EXPIRATION_SECONDS;
        self.next_sequence = match command {
            Some(_) => self.next_sequence + 1,
            None => 0,
        };
        self.next_command = command.map(|cmd| cmd.into()).unwrap_or_default();
    }

    #[inline(always)]
    pub fn get_command(&self) -> Option<(OperationCommand, Vec<OperationCommandAccountMeta>)> {
        let opt: Option<OperationCommandEntry> = self.next_command.into();
        opt.map(|entry| (entry.command, entry.required_accounts))
    }
}
