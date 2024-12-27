import * as web3 from '@solana/web3.js';
import * as spl from '@solana/spl-token';

import {RestakingProgram} from './program';
import {AddressBook} from '../program';
import {BN} from '@coral-xyz/anchor';
import * as anchor from '@coral-xyz/anchor';

class RestakingFund extends RestakingProgram {
    private readonly receiptTokenMint: web3.PublicKey;
    private readonly normalizedTokenMint: web3.PublicKey | null;
    private addressBook: ReturnType<typeof this.updateAddressBook>;

    constructor({ receiptTokenMint, normalizedTokenMint, ...params } : ConstructorParameters<typeof RestakingProgram>[0] & {
        receiptTokenMint: web3.PublicKey,
        normalizedTokenMint: web3.PublicKey | null,
    }) {
        super(params);
        this.receiptTokenMint = receiptTokenMint;
        this.normalizedTokenMint = normalizedTokenMint ?? null;
        this.addressBook = this.updateAddressBook();
    }

    public async fetchFundAccount() {
        const address = web3.PublicKey.findProgramAddressSync([Buffer.from('fund'), this.receiptTokenMint.toBuffer()], this.programID)[0];
        return this.accounts.fundAccount.fetch(address);
    }

    private async updateAddressBook() {
        const fundAccount = await this.fetchFundAccount().catch(() => null);
        const supportedTokens = fundAccount?.supportedTokens.slice(fundAccount!.numSupportedTokens) ?? [];

        const entries = (() => {
            // emit_cpi! macro
            const programEventAuthority = web3.PublicKey.findProgramAddressSync([Buffer.from('__event_authority')], this.programID)[0];

            // fund account and assets
            const fund = web3.PublicKey.findProgramAddressSync([Buffer.from('fund'), this.receiptTokenMint.toBuffer()], this.programID)[0];
            const fundReserve = web3.PublicKey.findProgramAddressSync([Buffer.from('fund_reserve'), this.receiptTokenMint.toBuffer()], this.programID)[0];
            const fundTreasury = web3.PublicKey.findProgramAddressSync([Buffer.from('fund_treasury'), this.receiptTokenMint.toBuffer()], this.programID)[0];

            // reward account
            const reward = web3.PublicKey.findProgramAddressSync([Buffer.from('reward'), this.receiptTokenMint.toBuffer()], this.programID)[0];

            // receipt token mint and extensions
            const tokenProgram2022 = spl.TOKEN_2022_PROGRAM_ID;
            const receiptTokenMint = this.receiptTokenMint;
            const receiptTokenExtraAccountMeta = spl.getExtraAccountMetaAddress(this.receiptTokenMint, this.programID);
            const fundReceiptTokenLock = spl.getAssociatedTokenAddressSync(this.receiptTokenMint, fund, true, tokenProgram2022);

            // supported tokens
            const tokenProgram = spl.TOKEN_PROGRAM_ID;
            const supportedTokenMints = Object.fromEntries(supportedTokens.map((v, i) => [`supportedTokenMint${i}`, v.mint]));
            const fundReserveSupportedTokens = Object.fromEntries(supportedTokens.map((v, i) => [`fundReserveSupportedToken${i}`, spl.getAssociatedTokenAddressSync(v.mint, fund, true, v.program)]));
            const fundTreasurySupportedTokens = Object.fromEntries(supportedTokens.map((v, i) => [`fundTreasurySupportedToken${i}`, spl.getAssociatedTokenAddressSync(v.mint, fundTreasury, true, v.program)]));

            // normalized token pool and mint
            const normalizedTokenMint = this.normalizedTokenMint;
            const normalizedTokenPool = normalizedTokenMint ? web3.PublicKey.findProgramAddressSync([Buffer.from('nt_pool'), normalizedTokenMint.toBuffer()], this.programID)[0] : null;
            const normalizedTokenPoolReserveSupportedTokens = normalizedTokenMint ? Object.fromEntries(supportedTokens.map((v, i) => [`normalizedTokenPoolReserveSupportedToken${i}`, spl.getAssociatedTokenAddressSync(v.mint, normalizedTokenPool!, true, v.program)])) : [];
            const fundReserveNormalizedToken = normalizedTokenMint ? spl.getAssociatedTokenAddressSync(normalizedTokenMint, fund, true, tokenProgram) : null;

            // program revenue
            const programRevenue = this.getConstantAsPublicKey('programRevenueAddress');
            const programRevenueSupportedTokens = Object.fromEntries(supportedTokens.map((v, i) => [`normalizedTokenPoolReserveSupportedToken${i}`, spl.getAssociatedTokenAddressSync(v.mint, programRevenue, true, v.program)]));

            // jito restaking
            const jitoVaultProgram = this.getConstantAsPublicKey('jitoVaultProgramId');
            const jitoVaultConfig = this.getConstantAsPublicKey('jitoVaultConfigAddress');

            return {
                programEventAuthority,

                fund,
                fundReserve,
                fundTreasury,

                reward,

                tokenProgram2022,
                receiptTokenMint,
                receiptTokenExtraAccountMeta,
                fundReceiptTokenLock,

                tokenProgram,
                supportedTokenMint0: null,
                ...supportedTokenMints,
                fundReserveSupportedToken0: null,
                ...fundReserveSupportedTokens,
                fundTreasurySupportedToken0: null,
                ...fundTreasurySupportedTokens,

                normalizedTokenMint,
                normalizedTokenPool,
                normalizedTokenPoolReserveSupportedToken0: null,
                ...normalizedTokenPoolReserveSupportedTokens,
                fundReserveNormalizedToken,

                programRevenue,
                programRevenueSupportedToken0: null,
                ...programRevenueSupportedTokens,

                jitoVaultProgram,
                jitoVaultConfig,
            };
        })();

        const addressBook = super.createAddressBook<keyof typeof entries>();
        const effectiveEntries = Object.fromEntries(Object.entries(entries).filter(([_, v]) => v && typeof (v as any)['toBase58'] == 'function'));
        addressBook.addAll(effectiveEntries);
        return addressBook;
    }
}

/*
TODO: ... moving lookup tables.... maybe use [string, web3.PublicKey, true] params in add method to denote what have to be put into ALT
fragSOLReward,

jitoVaultProgram,
jitoVaultConfig,
jitoVaultProgramFeeWallet,
fragSOLJitoVaultProgramFeeWalletTokenAccount,
fragSOLJitoVaultAccount,
fragSOLJitoVRTMint,
fragSOLJitoVaultFeeWalletTokenAccount,
fragSOLFundJitoVRTAccount,
fragSOLJitoVaultNSOLAccount,

private _getKnownAddress() {

    // fragSOL jito VRT
    const fragSOLJitoVRTMint = this.getConstantAsPublicKey('fragsolJitoVaultReceiptTokenMintAddress');

    const fragSOLFundJitoVRTAccount = spl.getAssociatedTokenAddressSync(
        fragSOLJitoVRTMint,
        fragSOLFund,
        true,
        spl.TOKEN_PROGRAM_ID,
        spl.ASSOCIATED_TOKEN_PROGRAM_ID,
    );

    // reward
    const [fragSOLReward] = web3.PublicKey.findProgramAddressSync([Buffer.from('reward'), fragSOLTokenMintBuf], this.programId);

    // jito
    const jitoVaultProgram = this.getConstantAsPublicKey('jitoVaultProgramId');
    const jitoVaultProgramFeeWallet = this.getConstantAsPublicKey('jitoVaultProgramFeeWallet');
    const jitoVaultConfig = this.getConstantAsPublicKey('jitoVaultConfigAddress');

    // fragSOL jito vault
    const fragSOLJitoVaultAccount = this.getConstantAsPublicKey('fragsolJitoVaultAccountAddress');
    const fragSOLJitoVaultUpdateStateTracker = (slot: anchor.BN, epoch_length: anchor.BN) => {
        let ncn_epoch = slot.div(epoch_length).toBuffer('le', 8);
        return web3.PublicKey.findProgramAddressSync([Buffer.from('vault_update_state_tracker'), fragSOLJitoVaultAccount.toBuffer(), ncn_epoch], jitoVaultProgram)[0];
    };
    const fragSOLJitoVaultNSOLAccount = spl.getAssociatedTokenAddressSync(
        nSOLTokenMint,
        fragSOLJitoVaultAccount,
        true,
        spl.TOKEN_PROGRAM_ID,
        spl.ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    const fragSOLJitoVaultWithdrawalTicketAccount1 = web3.PublicKey.findProgramAddressSync([Buffer.from('vault_staker_withdrawal_ticket'), fragSOLJitoVaultAccount.toBuffer(), vaultBaseAccount1.toBuffer()], jitoVaultProgram)[0];
    const fragSOLJitoVaultWithdrawalTicketTokenAccount1 = spl.getAssociatedTokenAddressSync(
        fragSOLJitoVRTMint,
        fragSOLJitoVaultWithdrawalTicketAccount1,
        true,
        spl.TOKEN_PROGRAM_ID,
        spl.ASSOCIATED_TOKEN_PROGRAM_ID,
    )
    const fragSOLJitoVaultWithdrawalTicketAccount2 = web3.PublicKey.findProgramAddressSync([Buffer.from('vault_staker_withdrawal_ticket'), fragSOLJitoVaultAccount.toBuffer(), vaultBaseAccount2.toBuffer()], jitoVaultProgram)[0];
    const fragSOLJitoVaultWithdrawalTicketTokenAccount2 = spl.getAssociatedTokenAddressSync(
        fragSOLJitoVRTMint,
        fragSOLJitoVaultWithdrawalTicketAccount2,
        true,
        spl.TOKEN_PROGRAM_ID,
        spl.ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    const fragSOLJitoVaultProgramFeeWalletTokenAccount = spl.getAssociatedTokenAddressSync(
        fragSOLJitoVRTMint,
        jitoVaultProgramFeeWallet,
        true,
        spl.TOKEN_PROGRAM_ID,
        spl.ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    const fragSOLJitoVaultFeeWalletTokenAccount = spl.getAssociatedTokenAddressSync(
        fragSOLJitoVRTMint,
        this.keychain.getPublicKey('ADMIN'),
        false,
        spl.TOKEN_PROGRAM_ID,
        spl.ASSOCIATED_TOKEN_PROGRAM_ID,
    );
}
 */
