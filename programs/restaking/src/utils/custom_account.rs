use anchor_lang::{prelude::*, solana_program::program_memory::sol_memcpy};

/// A custom type that is similar to [`Account`].
pub struct CustomAccount<'a, 'info: 'a, T: AccountSerialize + AccountDeserialize + Clone> {
    account: T,
    info: &'a AccountInfo<'info>,
}

impl<'a, 'info: 'a, T: AccountSerialize + AccountDeserialize + Clone + core::fmt::Debug>
    core::fmt::Debug for CustomAccount<'a, 'info, T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Account(Custom)")
            .field("account", &self.account)
            .field("info", &self.info)
            .finish()
    }
}

impl<'a, 'info: 'a, T: AccountSerialize + AccountDeserialize + Clone> CustomAccount<'a, 'info, T> {
    pub fn new(info: &'a AccountInfo<'info>, account: T) -> Self {
        Self { info, account }
    }

    pub fn exit_with_expected_owner(
        &self,
        expected_owner: &Pubkey,
        program_id: &Pubkey,
    ) -> Result<()> {
        // Only persist if the owner is the current program and the account is not closed.
        if expected_owner == program_id && !self.is_closed() {
            let info = self.to_account_info();
            let mut data = info.try_borrow_mut_data()?;
            let dst: &mut [u8] = &mut data;
            let mut writer = BpfWriter::new(dst);
            self.account.try_serialize(&mut writer)?;
        }
        Ok(())
    }

    fn is_closed(&self) -> bool {
        self.info.owner == &System::id() && self.info.data_is_empty()
    }

    /// Reloads the account from storage. This is useful, for example, when
    /// observing side effects after CPI.
    #[allow(dead_code)]
    pub fn reload(&mut self) -> Result<()> {
        let mut data: &[u8] = &self.info.try_borrow_data()?;
        self.account = T::try_deserialize(&mut data)?;
        Ok(())
    }

    pub fn to_account_info(&self) -> AccountInfo<'info> {
        self.info.clone()
    }
}

impl<'a, 'info: 'a, T: AccountSerialize + AccountDeserialize + Owner + Clone>
    CustomAccount<'a, 'info, T>
{
    fn check_initialized(info: &'a AccountInfo<'info>) -> Result<()> {
        if info.owner == &anchor_lang::solana_program::system_program::ID && info.lamports() == 0 {
            return Err(ErrorCode::AccountNotInitialized.into());
        }

        Ok(())
    }

    fn check_owned_by_program(info: &'a AccountInfo<'info>) -> Result<()> {
        if info.owner != &T::owner() {
            return Err(Error::from(ErrorCode::AccountOwnedByWrongProgram)
                .with_pubkeys((*info.owner, T::owner())));
        }

        Ok(())
    }

    /// Deserializes the given `info` into a `CustomAccount`.
    #[inline(never)]
    pub fn try_from(info: &'a AccountInfo<'info>) -> Result<Self> {
        Self::check_initialized(info)?;
        Self::check_owned_by_program(info)?;
        let mut data: &[u8] = &info.try_borrow_data()?;
        Ok(Self::new(info, T::try_deserialize(&mut data)?))
    }

    /// Deserializes the given `info` into a `CustomAccount` without checing
    /// the account discriminator. Be careful when using this and avoid it if
    /// possible.
    #[inline(never)]
    pub fn try_from_unchecked(info: &'a AccountInfo<'info>) -> Result<Self> {
        Self::check_initialized(info)?;
        Self::check_owned_by_program(info)?;
        let mut data: &[u8] = &info.try_borrow_data()?;
        Ok(Self::new(info, T::try_deserialize_unchecked(&mut data)?))
    }
}

impl<'a, 'info: 'a, T: AccountSerialize + AccountDeserialize + Owner + Clone> AccountsExit<'info>
    for CustomAccount<'a, 'info, T>
{
    fn exit(&self, program_id: &Pubkey) -> Result<()> {
        self.exit_with_expected_owner(&T::owner(), program_id)
    }
}

impl<'a, 'info: 'a, T: AccountSerialize + AccountDeserialize + Clone> ToAccountMetas
    for CustomAccount<'a, 'info, T>
{
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        let is_signer = is_signer.unwrap_or(self.info.is_signer);
        let meta = match self.info.is_writable {
            false => AccountMeta::new_readonly(*self.info.key, is_signer),
            true => AccountMeta::new(*self.info.key, is_signer),
        };
        vec![meta]
    }
}

impl<'a, 'info: 'a, T: AccountSerialize + AccountDeserialize + Clone> ToAccountInfos<'info>
    for CustomAccount<'a, 'info, T>
{
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        vec![self.info.clone()]
    }
}

impl<'a, 'info: 'a, T: AccountSerialize + AccountDeserialize + Clone> AsRef<T>
    for CustomAccount<'a, 'info, T>
{
    fn as_ref(&self) -> &T {
        &self.account
    }
}

impl<'a, 'info: 'a, T: AccountSerialize + AccountDeserialize + Clone> std::ops::Deref
    for CustomAccount<'a, 'info, T>
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}

impl<'a, 'info: 'a, T: AccountSerialize + AccountDeserialize + Clone> std::ops::DerefMut
    for CustomAccount<'a, 'info, T>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.account
    }
}

impl<'a, 'info: 'a, T: AccountSerialize + AccountDeserialize + Clone> Key
    for CustomAccount<'a, 'info, T>
{
    fn key(&self) -> Pubkey {
        *self.info.key
    }
}

#[derive(Debug, Default)]
struct BpfWriter<T> {
    inner: T,
    pos: u64,
}

impl<T> BpfWriter<T> {
    fn new(inner: T) -> Self {
        Self { inner, pos: 0 }
    }
}

impl std::io::Write for BpfWriter<&mut [u8]> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.pos >= self.inner.len() as u64 {
            return Ok(0);
        }

        let amt = std::cmp::min(
            self.inner.len().saturating_sub(self.pos as usize),
            buf.len(),
        );
        sol_memcpy(&mut self.inner[(self.pos as usize)..], buf, amt);
        self.pos += amt as u64;
        Ok(amt)
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        if self.write(buf)? == buf.len() {
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "failed to write whole buffer",
            ))
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
