use anchor_lang::prelude::*;
use switchboard_on_demand::{prelude::rust_decimal::Decimal, PullFeedAccountData};

use crate::constants::SWITCHBOARD_ON_DEMAND_PROGRAM_ID;
use crate::errors::ErrorCode;

pub struct SwitchboardOracleService<'a, 'info> {
    feed_account: &'a AccountInfo<'info>,
}

impl<'a, 'info> SwitchboardOracleService<'a, 'info> {
    pub fn new(feed_account: &'a AccountInfo<'info>) -> Result<Self> {
        require_keys_eq!(*feed_account.owner, SWITCHBOARD_ON_DEMAND_PROGRAM_ID);

        Ok(Self { feed_account })
    }

    #[inline(always)]
    fn borrow_account_data<'b, 'c>(
        account: &'b AccountInfo<'c>,
    ) -> Result<std::cell::Ref<'b, &'c mut [u8]>> {
        Ok(account
            .data
            .try_borrow()
            .map_err(|_| ProgramError::AccountBorrowFailed)?)
    }

    #[inline(always)]
    fn deserialize_account_data<'b, T: bytemuck::Pod>(
        data: &'b std::cell::Ref<&mut [u8]>,
    ) -> Result<&'b T> {
        bytemuck::try_from_bytes(data[8..].as_ref())
            .map_err(|_| error!(error::ErrorCode::AccountDidNotDeserialize))
    }

    pub fn get_feed_value(&self) -> Result<u128> {
        let data = &Self::borrow_account_data(self.feed_account)?;
        let feed = Self::deserialize_account_data::<PullFeedAccountData>(data)?;

        let feed_value = feed
            .get_value(
                &Clock::get()?,
                feed.max_staleness as u64,
                feed.min_sample_size as u32,
                false,
            )
            .map(|v| v.mantissa())
            .unwrap_or_else(|_| 0i128);

        Ok(feed_value.max(0) as u128)
    }
}
