use anchor_lang::{
    prelude::*
    ,
    solana_program::program::invoke_signed,
};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

pub fn process_withdraw_sol_from_spl_stake_pool<'info>(
    pool_account: &AccountInfo<'info>,
    withdraw_authority: &AccountInfo<'info>,
    reserve_stake_account: &AccountInfo<'info>,
    manager_fee_account: &AccountInfo<'info>,
    sysvar_clock_program: &AccountInfo<'info>,
    sysvar_stake_history_program: &AccountInfo<'info>,
    stake_program: &AccountInfo<'info>,
    spl_stake_pool_program: &AccountInfo<'info>,
    pool_token_account_from: &InterfaceAccount<'info, TokenAccount>,
    lamport_account_to: &AccountInfo<'info>,
    spl_pool_token_mint: &InterfaceAccount<'info, Mint>,
    supported_token_program: &Interface<'info, TokenInterface>,
    signer: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    token_amount: u64,
) -> Result<()> {
    /*
    // { // stake pool program id
            //     pubkey: new anchor.web3.PublicKey("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy"),
            //     isSigner: false,
            //     isWritable: false,
            // },
            { // jito stake pool address
                pubkey: new anchor.web3.PublicKey("Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb"),
                isSigner: false,
                isWritable: true,
            },
            { // stake_pool_withdraw_authority
                pubkey: jitoStakePoolWithdrawAuthority,
                isSigner: false,
                isWritable: false,
            },
            // user_transfer_authority
            // pool_tokens_from
            { // reserve_stake_account
                pubkey: new anchor.web3.PublicKey("BgKUXdS29YcHCFrPm5M8oLHiTzZaMDjsebggjoaQ6KFL"),
                isSigner: false,
                isWritable: true,
            },
            // lamports_to
            { // manager_fee_account
                pubkey: new anchor.web3.PublicKey("feeeFLLsam6xZJFc6UQFrHqkvVt4jfmVvi2BRLkUZ4i"),
                isSigner: false,
                isWritable: true,
            },
            // pool_mint
            // token_program_id
            { // sysvar_clock_id ???
                pubkey: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                isSigner: false,
                isWritable: false,
            },
            { // sysvar_stake_history_id ???
                pubkey: anchor.web3.SYSVAR_STAKE_HISTORY_PUBKEY,
                isSigner: false,
                isWritable: false,
            },
            { // stake_program_id ???
                pubkey: anchor.web3.StakeProgram.programId,
                isSigner: false,
                isWritable: false,
            },
     */
    let withdraw_sol_ix = spl_stake_pool::instruction::withdraw_sol(
        spl_stake_pool_program.key,
        pool_account.key,
        withdraw_authority.key,
        signer.key,
        &pool_token_account_from.key(),
        reserve_stake_account.key,
        lamport_account_to.key,
        manager_fee_account.key,
        &spl_pool_token_mint.key(),
        supported_token_program.key,
        token_amount,
    );
    // msg!("&withdraw_sol_ix.accounts[2].pubkey: {}, is_signer: {}, is_writable: {}", &withdraw_sol_ix.accounts[2].pubkey, &withdraw_sol_ix.accounts[2].is_signer, &withdraw_sol_ix.accounts[2].is_writable);
    // for (i, ix_account) in withdraw_sol_ix.accounts.clone().into_iter().enumerate() {
    //     msg!("&withdraw_sol_ix.accounts[{}].pubkey: {}, is_signer: {}, is_writable: {}", i, &ix_account.pubkey, &ix_account.is_signer, &ix_account.is_writable);
    // }

    invoke_signed(
        &withdraw_sol_ix,
        &[
            pool_account.clone(),
            withdraw_authority.clone(),
            signer.clone(),
            pool_token_account_from.to_account_info(),
            reserve_stake_account.clone(),
            lamport_account_to.clone(),
            manager_fee_account.clone(),
            spl_pool_token_mint.to_account_info(),
            sysvar_clock_program.clone(),
            sysvar_stake_history_program.clone(),
            stake_program.clone(),
            supported_token_program.to_account_info(),
        ],
        signer_seeds,
    )?;

    Ok(())
}
