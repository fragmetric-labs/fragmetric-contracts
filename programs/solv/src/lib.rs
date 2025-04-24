#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

mod constants;

mod instructions;

use constants::*;
use instructions::*;

#[program]
pub mod solv {
    use super::*;

    ////////////////////////////////////////////
    // VaultInitialContext
    ////////////////////////////////////////////

    pub fn initialize_vault_account(_ctx: Context<VaultAccountInitialContext>) -> Result<()> {
        todo!("holy")
    }
}
