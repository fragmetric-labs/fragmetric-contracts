use anchor_lang::prelude::*;
use marinade_cpi::state::State;
use spl_stake_pool::state::StakePool;

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

    pub(super) fn deserialize_pool_account(
        pool_account_info: &'a AccountInfo<'info>,
    ) -> Result<State> {
        // ref: https://docs.rs/marinade-cpi/latest/marinade_cpi/state/struct.State.html
        let pool_account = State::try_deserialize(
            &mut &**pool_account_info.try_borrow_data()?,
        )
            .map_err(|_| error!(ErrorCode::AccountDidNotDeserialize))?;
        Ok(pool_account)
    }

    // TODO: ...
}