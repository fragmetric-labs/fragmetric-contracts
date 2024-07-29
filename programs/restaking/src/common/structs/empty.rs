use anchor_lang::prelude::*;

#[account]
pub struct Empty;

impl Space for Empty {
    const INIT_SPACE: usize = 0;
}
