use anchor_lang::{prelude::*, system_program::*};

pub trait SystemProgramExt<'info> {
    fn create_program_account(
        &self,
        from: &impl ToAccountInfo<'info>,
        to: &impl ToAccountInfo<'info>,
        space: u64,
        rent: u64,
        to_signer_seeds: Option<&[&[&[u8]]]>,
    ) -> Result<()>;

    fn transfer(
        &self,
        from: &impl ToAccountInfo<'info>,
        to: &impl ToAccountInfo<'info>,
        amount: u64,
        from_signer_seeds: Option<&[&[&[u8]]]>,
    ) -> Result<()>;

    fn allocate(
        &self,
        account_to_allocate: &impl ToAccountInfo<'info>,
        space: u64,
        signer_seeds: Option<&[&[&[u8]]]>,
    ) -> Result<()>;

    fn assign_to_program(
        &self,
        account_to_assign: &impl ToAccountInfo<'info>,
        signer_seeds: Option<&[&[&[u8]]]>,
    ) -> Result<()>;
}

impl<'info> SystemProgramExt<'info> for Program<'info, System> {
    fn create_program_account(
        &self,
        from: &impl ToAccountInfo<'info>,
        to: &impl ToAccountInfo<'info>,
        space: u64,
        rent: u64,
        to_signer_seeds: Option<&[&[&[u8]]]>,
    ) -> Result<()> {
        let accounts = CreateAccount {
            from: from.to_account_info(),
            to: to.to_account_info(),
        };
        let ctx = CpiContext::new_with_signer(
            self.to_account_info(),
            accounts,
            to_signer_seeds.unwrap_or_default(),
        );
        create_account(ctx, rent, space, &crate::ID)
    }

    fn transfer(
        &self,
        from: &impl ToAccountInfo<'info>,
        to: &impl ToAccountInfo<'info>,
        amount: u64,
        from_signer_seeds: Option<&[&[&[u8]]]>,
    ) -> Result<()> {
        let accounts = Transfer {
            from: from.to_account_info(),
            to: to.to_account_info(),
        };
        let ctx = CpiContext::new_with_signer(
            self.to_account_info(),
            accounts,
            from_signer_seeds.unwrap_or_default(),
        );
        transfer(ctx, amount)
    }

    fn allocate(
        &self,
        account_to_allocate: &impl ToAccountInfo<'info>,
        space: u64,
        signer_seeds: Option<&[&[&[u8]]]>,
    ) -> Result<()> {
        let accounts = Allocate {
            account_to_allocate: account_to_allocate.to_account_info(),
        };
        let ctx = CpiContext::new_with_signer(
            self.to_account_info(),
            accounts,
            signer_seeds.unwrap_or_default(),
        );
        allocate(ctx, space)
    }

    fn assign_to_program(
        &self,
        account_to_assign: &impl ToAccountInfo<'info>,
        signer_seeds: Option<&[&[&[u8]]]>,
    ) -> Result<()> {
        let accounts = Assign {
            account_to_assign: account_to_assign.to_account_info(),
        };
        let ctx = CpiContext::new_with_signer(
            self.to_account_info(),
            accounts,
            signer_seeds.unwrap_or_default(),
        );
        assign(ctx, &crate::ID)
    }
}
