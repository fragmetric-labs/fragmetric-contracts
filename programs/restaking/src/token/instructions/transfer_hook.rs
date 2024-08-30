use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::spl_token_2022::{
        extension::{
            transfer_hook::TransferHookAccount, BaseStateWithExtensionsMut,
            PodStateWithExtensionsMut,
        },
        pod::PodAccount,
    },
    token_interface::{Mint, TokenAccount},
};

use crate::{common::*, constants::*, error::ErrorCode, fund::*, reward::*, token::*, utils::*};

#[derive(Accounts)]
pub struct TokenTransferHook<'info> {
    #[account(
        token::mint = receipt_token_mint,
        token::authority = owner,
    )]
    pub source_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        token::mint = receipt_token_mint,
    )]
    pub destination_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: source token account owner, can be SystemAccount or PDA owned by another program
    pub owner: UncheckedAccount<'info>,

    /// CHECK: ExtraAccountMetaList account
    #[account(
        seeds = [b"extra-account-metas", receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    // pub whitelisted_destination_token: Account<'info, WhitelistedDestinationToken>,
    #[account(
        mut,
        seeds = [Fund::SEED, receipt_token_mint.key().as_ref()],
        bump = fund.bump,
        has_one = receipt_token_mint,
    )]
    pub fund: Box<Account<'info, Fund>>,

    #[account(
        mut,
        seeds = [UserReceipt::SEED, source_token_account.owner.as_ref(), receipt_token_mint.key().as_ref()],
        bump,
    )]
    /// CHECK: will be deserialized in runtime
    pub source_user_receipt: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [UserReceipt::SEED, destination_token_account.owner.as_ref(), receipt_token_mint.key().as_ref()],
        bump,
    )]
    /// CHECK: will be deserialized in runtime
    pub destination_user_receipt: UncheckedAccount<'info>,

    #[account(mut, address = REWARD_ACCOUNT_ADDRESS)]
    pub reward_account: Box<Account<'info, RewardAccount>>,

    #[account(
        mut,
        seeds = [UserRewardAccount::SEED, source_token_account.owner.as_ref()],
        bump,
    )]
    /// CHECK: will be deserialized in runtime
    pub source_user_reward_account: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [UserRewardAccount::SEED, destination_token_account.owner.as_ref()],
        bump,
    )]
    /// CHECK: will be deserialized in runtime
    pub destination_user_reward_account: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [PAYER_ACCOUNT_SEED],
        bump,
        owner = anchor_lang::solana_program::system_program::ID,
    )]
    /// CHECK: empty
    pub payer_account: UncheckedAccount<'info>,
}

impl<'info> TokenTransferHook<'info> {
    pub fn transfer_hook(ctx: Context<Self>, amount: u64) -> Result<()> {
        let payer_account = &ctx.accounts.payer_account;
        let receipt_token_mint = ctx.accounts.receipt_token_mint.key();
        let source_token_account_owner = ctx.accounts.source_token_account.owner;
        let destination_token_account_owner = ctx.accounts.destination_token_account.owner;
        let system_program = &ctx.accounts.system_program;

        // Custom deserialize
        let mut source_user_receipt = ctx
            .accounts
            .source_user_receipt
            .deserialize_if_exist::<UserReceipt>("source_user_receipt")?;
        if let Some(source_user_receipt) = &source_user_receipt {
            if source_user_receipt.data_version != 0 {
                require_eq!(source_user_receipt.bump, ctx.bumps.source_user_receipt);
                require_keys_eq!(source_user_receipt.user, source_token_account_owner);
                require_keys_eq!(source_user_receipt.receipt_token_mint, receipt_token_mint);
            }
        }

        let mut destination_user_receipt = ctx
            .accounts
            .destination_user_receipt
            .deserialize_if_exist::<UserReceipt>("destination_user_receipt")?;
        if let Some(destination_user_receipt) = &destination_user_receipt {
            if destination_user_receipt.data_version != 0 {
                require_eq!(
                    destination_user_receipt.bump,
                    ctx.bumps.destination_user_receipt
                );
                require_keys_eq!(
                    destination_user_receipt.user,
                    destination_token_account_owner
                );
                require_keys_eq!(
                    destination_user_receipt.receipt_token_mint,
                    receipt_token_mint
                );
            }
        }

        let mut source_user_reward_account = ctx
            .accounts
            .source_user_reward_account
            .init_if_needed_by_pda::<UserRewardAccount>(
            "source_user_reward_account",
            AsRef::as_ref(payer_account),
            &[&[PAYER_ACCOUNT_SEED, &[ctx.bumps.payer_account]]],
            8 + UserRewardAccount::INIT_SPACE,
            Some(&[&[
                UserRewardAccount::SEED,
                source_token_account_owner.as_ref(),
                &[ctx.bumps.source_user_reward_account],
            ]]),
            system_program,
        )?;
        if source_user_reward_account.data_version != 0 {
            require_eq!(
                source_user_reward_account.bump,
                ctx.bumps.source_user_reward_account
            );
            require_keys_eq!(source_user_reward_account.user, source_token_account_owner);
        }

        let mut destination_user_reward_account = ctx
            .accounts
            .destination_user_reward_account
            .init_if_needed_by_pda::<UserRewardAccount>(
            "destination_user_reward_account",
            AsRef::as_ref(payer_account),
            &[&[PAYER_ACCOUNT_SEED, &[ctx.bumps.payer_account]]],
            8 + UserRewardAccount::INIT_SPACE,
            Some(&[&[
                UserRewardAccount::SEED,
                destination_token_account_owner.as_ref(),
                &[ctx.bumps.destination_user_reward_account],
            ]]),
            system_program,
        )?;
        if destination_user_reward_account.data_version != 0 {
            require_eq!(
                destination_user_reward_account.bump,
                ctx.bumps.destination_user_reward_account
            );
            require_keys_eq!(
                destination_user_reward_account.user,
                destination_token_account_owner
            );
        }

        // Initialize
        if let Some(source_user_receipt) = &mut source_user_receipt {
            source_user_receipt.initialize_if_needed(
                ctx.bumps.source_user_receipt,
                source_token_account_owner,
                receipt_token_mint,
            );
        }
        if let Some(destination_user_receipt) = &mut destination_user_receipt {
            destination_user_receipt.initialize_if_needed(
                ctx.bumps.destination_user_receipt,
                destination_token_account_owner,
                receipt_token_mint,
            );
        }
        source_user_reward_account.initialize_if_needed(
            ctx.bumps.source_user_reward_account,
            source_token_account_owner,
        );
        destination_user_reward_account.initialize_if_needed(
            ctx.bumps.destination_user_reward_account,
            destination_token_account_owner,
        );

        Self::check_token_transferring(&ctx)?;
        Self::call_transfer_hook(
            &mut ctx.accounts.reward_account,
            receipt_token_mint,
            amount,
            &mut source_user_reward_account,
            &mut destination_user_reward_account,
        )?;

        // Update source/destination user_receipt's receipt_token_amount
        let source_token_account_total_amount = ctx.accounts.source_token_account.amount;
        let source_user_receipt = if let Some(source_user_receipt) = &mut source_user_receipt {
            source_user_receipt.set_receipt_token_amount(source_token_account_total_amount);
            source_user_receipt.exit(&crate::ID)?;
            source_user_receipt.as_ref().clone()
        } else {
            UserReceipt::dummy(
                source_token_account_owner,
                receipt_token_mint,
                source_token_account_total_amount,
            )
        };
        let destination_token_account_total_amount = ctx.accounts.destination_token_account.amount;
        let destination_user_receipt =
            if let Some(destination_user_receipt) = &mut destination_user_receipt {
                destination_user_receipt
                    .set_receipt_token_amount(destination_token_account_total_amount);
                destination_user_receipt.exit(&crate::ID)?;
                destination_user_receipt.as_ref().clone()
            } else {
                UserReceipt::dummy(
                    destination_token_account_owner,
                    receipt_token_mint,
                    destination_token_account_total_amount,
                )
            };

        emit!(UserTransferredReceiptToken {
            transferred_receipt_token_mint: ctx.accounts.receipt_token_mint.key(),
            transferred_receipt_token_amount: amount,
            source_receipt_token_account: ctx.accounts.source_token_account.key(),
            source_user: ctx.accounts.source_token_account.owner,
            source_user_receipt,
            destination_receipt_token_account: ctx.accounts.destination_token_account.key(),
            destination_user: ctx.accounts.destination_token_account.owner,
            destination_user_receipt,
        });

        // exit - flush data back to solana bpf
        source_user_reward_account.exit(&crate::ID)?;
        destination_user_reward_account.exit(&crate::ID)?;

        Ok(())
    }

    fn call_transfer_hook(
        reward_account: &mut RewardAccount,
        receipt_token_mint: Pubkey,
        amount: u64,
        source_user_reward_account: &mut UserRewardAccount,
        destination_user_reward_account: &mut UserRewardAccount,
    ) -> Result<()> {
        let current_slot = Clock::get()?.slot;

        let (from_user_update, to_user_update) = reward_account
            .update_reward_pools_token_allocation(
                receipt_token_mint,
                amount,
                None,
                Some(source_user_reward_account),
                Some(destination_user_reward_account),
                current_slot,
            )?;

        emit!(UserUpdatedRewardPool::new_from_updates(
            from_user_update,
            to_user_update
        ));

        Ok(())
    }

    fn check_token_transferring(ctx: &Context<Self>) -> Result<()> {
        let source_token_account_info = ctx.accounts.source_token_account.to_account_info();
        let mut account_data_ref = source_token_account_info.try_borrow_mut_data()?;
        let mut account = PodStateWithExtensionsMut::<PodAccount>::unpack(*account_data_ref)?;
        let account_extension = account.get_extension_mut::<TransferHookAccount>()?;

        if !bool::from(account_extension.transferring) {
            return err!(ErrorCode::TokenNotCurrentlyTransferring);
        }

        Ok(())
    }
}
