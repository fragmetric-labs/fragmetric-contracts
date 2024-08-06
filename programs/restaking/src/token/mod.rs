use anchor_lang::prelude::*;
use anchor_spl::{token_2022::Token2022, token_interface::TokenInterface};

mod instructions;

pub use instructions::*;

mod burn;
mod mint;
mod transfer;

pub(crate) use burn::*;
pub(crate) use mint::*;
pub(crate) use transfer::*;

trait TokenProgram<'info>: ToAccountInfo<'info> {}
impl<'info> TokenProgram<'info> for Program<'info, Token2022> {}
impl<'info> TokenProgram<'info> for Interface<'info, TokenInterface> {}
