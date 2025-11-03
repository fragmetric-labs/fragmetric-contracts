use anchor_lang::prelude::*;

use super::commands::*;

const FUND_ACCOUNT_OPERATION_COMMAND_EXPIRATION_SECONDS: i64 = 600;

#[zero_copy]
pub(super) struct OperationState {
    updated_slot: u64,
    updated_at: i64,
    expired_at: i64,

    _padding: [u8; 5],
    /// when the no_transition flag turned on, current command should not be transitioned to other command.
    /// the purpose of this flag is for internal testing by set boundary of the reset command operation.
    no_transition: u8,
    pub next_sequence: u16,
    pub num_operated: u64,

    next_command: OperationCommandEntryPod,

    _reserved: [u8; 49],
}

impl OperationState {
    /// Initialize current operation command to `reset_command` or default.
    pub fn initialize_command_if_needed(
        &mut self,
        reset_command: Option<OperationCommandEntry>,
        current_slot: u64,
        current_timestamp: i64,
    ) -> Result<()> {
        let has_reset_command = reset_command.is_some();

        if has_reset_command || self.next_command.is_none() || current_timestamp > self.expired_at {
            self.no_transition = 0;
            self.next_sequence = 0;
            self.set_command(
                reset_command.or_else(|| Some(Default::default())),
                current_slot,
                current_timestamp,
            )?;
            self.no_transition = has_reset_command as u8;
        }
        Ok(())
    }

    /// Sets next operation command and increment sequence number.
    pub fn set_command(
        &mut self,
        mut next_command: Option<OperationCommandEntry>,
        current_slot: u64,
        current_timestamp: i64,
    ) -> Result<()> {
        // deal with no_transition state, to adjust next command.
        if self.no_transition == 1 {
            let prev = self.next_command.discriminant();
            let next = next_command
                .as_ref()
                .map(|command| command.command.discriminant());
            if let (Some(prev), Some(next)) = (prev, next) {
                if prev != next {
                    // when the type of the command changes on no_transition state, ignore the next command and clear no_transition state.
                    msg!(
                        "COMMAND#{} reset due to no_transition state",
                        self.next_sequence
                    );
                    self.no_transition = 0;
                    next_command = None;
                }
                // otherwise, retaining on the same command type, still maintains no_transition state.
            } else {
                // if there is no previous command (unexpected flow) or next command, clear no_transition state.
                self.no_transition = 0;
            }
        }

        self.updated_slot = current_slot;
        self.updated_at = current_timestamp;
        self.expired_at = current_timestamp + FUND_ACCOUNT_OPERATION_COMMAND_EXPIRATION_SECONDS;
        self.num_operated += 1;
        self.next_sequence = match next_command {
            Some(_) => self.next_sequence + 1,
            None => 0,
        };
        match &next_command {
            Some(next_command) => next_command.serialize_as_pod(&mut self.next_command)?,
            None => self.next_command.set_none(),
        };
        Ok(())
    }

    #[inline(always)]
    pub fn get_next_command(&self) -> Result<Option<OperationCommandEntry>> {
        self.next_command.try_deserialize()
    }
}
