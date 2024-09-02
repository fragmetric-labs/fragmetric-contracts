use anchor_lang::prelude::*;
use super::CustomAccount;

pub trait DeserializeIfExist<'info> {
    fn deserialize_if_exist<T: AccountSerialize + AccountDeserialize + Owner + Clone>(
        &self,
        account_name: &str,
    ) -> Result<Option<CustomAccount<'_, 'info, T>>>;
}

impl<'info> DeserializeIfExist<'info> for UncheckedAccount<'info> {
    fn deserialize_if_exist<T: AccountSerialize + AccountDeserialize + Owner + Clone>(
        &self,
        account_name: &str,
    ) -> Result<Option<CustomAccount<'_, 'info, T>>> {
        let rent = Rent::get()?;
        let actual_owner = self.owner;
        if actual_owner == &crate::ID && rent.is_exempt(self.lamports(), self.data_len()) {
            CustomAccount::try_from(self)
                .map_err(|e| e.with_account_name(account_name))
                .map(Some)
        } else {
            Ok(None)
        }
    }
}
