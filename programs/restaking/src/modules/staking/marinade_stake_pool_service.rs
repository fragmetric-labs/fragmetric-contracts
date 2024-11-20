use anchor_lang::prelude::*;

pub struct MarinadeStakePoolService<'info: 'a, 'a> {
    pub marinade_stake_pool_program: &'a AccountInfo<'info>,
    pub pool_account: &'a AccountInfo<'info>,
    pub pool_token_mint: &'a AccountInfo<'info>,
    pub pool_token_program: &'a AccountInfo<'info>,
}

impl<'info, 'a> MarinadeStakePoolService<'info, 'a> {
    pub fn new(
        marinade_stake_pool_program: &'a AccountInfo<'info>,
        pool_account: &'a AccountInfo<'info>,
        pool_token_mint: &'a AccountInfo<'info>,
        pool_token_program: &'a AccountInfo<'info>,
    ) -> Result<Self> {
        require_eq!(marinade_cpi::ID, marinade_stake_pool_program.key());

        Ok(Self {
            marinade_stake_pool_program,
            pool_account,
            pool_token_mint,
            pool_token_program,
        })
    }

    // TODO: ...
}
