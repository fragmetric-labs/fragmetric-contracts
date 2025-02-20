use anchor_lang::prelude::*;

#[zero_copy]
#[repr(C)]
pub(super) struct WrappedToken {
    pub mint: Pubkey,
    pub program: Pubkey,
    pub decimals: u8,
    pub enabled: u8,
    _padding: [u8; 6],
    pub supply: u64,
    _reserved: [u8; 1984],
}

impl WrappedToken {
    pub fn initialize(
        &mut self,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        supply: u64,
    ) -> Result<()> {
        require_eq!(self.enabled, 0);

        self.enabled = 1;
        self.mint = mint;
        self.program = program;
        self.decimals = decimals;
        self.supply = supply;

        Ok(())
    }
}
