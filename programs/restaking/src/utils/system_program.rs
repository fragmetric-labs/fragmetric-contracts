use anchor_lang::{prelude::*, system_program::*};

pub(crate) trait SystemProgramExt<'info> {
    /// Transfer sol `from` -> `to`
    fn transfer(
        &self,
        from: &impl ToAccountInfo<'info>,
        from_signer_seeds: Option<&[&[&[u8]]]>,
        to: &impl ToAccountInfo<'info>,
        amount: u64,
    ) -> Result<()>;

    /// Create account, paid by `payer`, owned by `owner`.
    #[allow(clippy::too_many_arguments)]
    fn create_account(
        &self,
        payer: &impl ToAccountInfo<'info>,
        payer_signer_seeds: Option<&[&[&[u8]]]>,
        account_to_create: &impl ToAccountInfo<'info>,
        account_to_create_signer_seeds: Option<&[&[&[u8]]]>,
        space: u64,
        rent: u64,
        owner: &Pubkey,
    ) -> Result<()>;

    /// Allocate `space` to an account `account_to_allocate`.
    fn allocate(
        &self,
        account_to_allocate: &impl ToAccountInfo<'info>,
        account_to_allocate_signer_seeds: Option<&[&[&[u8]]]>,
        space: u64,
    ) -> Result<()>;

    /// Assign account `account_to_assign` to owner `owner`.
    fn assign(
        &self,
        account_to_assign: &impl ToAccountInfo<'info>,
        account_to_assign_signer_seeds: Option<&[&[&[u8]]]>,
        owner: &Pubkey,
    ) -> Result<()>;
}

impl<'info> SystemProgramExt<'info> for Program<'info, System> {
    fn transfer(
        &self,
        from: &impl ToAccountInfo<'info>,
        from_signer_seeds: Option<&[&[&[u8]]]>,
        to: &impl ToAccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        let signer_seeds = from_signer_seeds.unwrap_or_default();
        let accounts = Transfer {
            from: from.to_account_info(),
            to: to.to_account_info(),
        };
        let ctx = CpiContext::new_with_signer(self.to_account_info(), accounts, signer_seeds);
        transfer(ctx, amount)
    }

    fn create_account(
        &self,
        payer: &impl ToAccountInfo<'info>,
        payer_signer_seeds: Option<&[&[&[u8]]]>,
        account_to_create: &impl ToAccountInfo<'info>,
        account_to_create_signer_seeds: Option<&[&[&[u8]]]>,
        space: u64,
        rent: u64,
        owner: &Pubkey,
    ) -> Result<()> {
        let signer_seeds = payer_signer_seeds
            .unwrap_or_default()
            .iter()
            .chain(account_to_create_signer_seeds.unwrap_or_default().iter())
            .copied()
            .collect::<Vec<_>>();

        let accounts = CreateAccount {
            from: payer.to_account_info(),
            to: account_to_create.to_account_info(),
        };
        let ctx =
            CpiContext::new_with_signer(self.to_account_info(), accounts, signer_seeds.as_slice());
        create_account(ctx, rent, space, owner)
    }

    fn allocate(
        &self,
        account_to_allocate: &impl ToAccountInfo<'info>,
        account_to_allocate_signer_seeds: Option<&[&[&[u8]]]>,
        space: u64,
    ) -> Result<()> {
        let signer_seeds = account_to_allocate_signer_seeds.unwrap_or_default();
        let accounts = Allocate {
            account_to_allocate: account_to_allocate.to_account_info(),
        };
        let ctx = CpiContext::new_with_signer(self.to_account_info(), accounts, signer_seeds);
        allocate(ctx, space)
    }

    fn assign(
        &self,
        account_to_assign: &impl ToAccountInfo<'info>,
        account_to_assign_signer_seeds: Option<&[&[&[u8]]]>,
        owner: &Pubkey,
    ) -> Result<()> {
        let signer_seeds = account_to_assign_signer_seeds.unwrap_or_default();
        let accounts = Assign {
            account_to_assign: account_to_assign.to_account_info(),
        };
        let ctx = CpiContext::new_with_signer(self.to_account_info(), accounts, signer_seeds);
        assign(ctx, owner)
    }
}
