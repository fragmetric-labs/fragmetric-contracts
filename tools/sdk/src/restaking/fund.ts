import * as web3 from '@solana/web3.js';
import * as spl from '@solana/spl-token';

import {RestakingProgram} from './program';

export class RestakingFund extends RestakingProgram {
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
        const restakingVaults = fundAccount?.restakingVaults.slice(fundAccount!.numRestakingVaults) ?? [];

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

            // restaking vaults
            const fundRestakingVaults = Object.fromEntries(restakingVaults.map((v, i) => [`fundRestakingVault${i}`, v.vault]));
            const fundRestakingVaultPrograms = Object.fromEntries(restakingVaults.map((v, i) => [`fundRestakingVaultProgram${i}`, v.program]));
            const fundRestakingVaultReceiptTokenMints = Object.fromEntries(restakingVaults.map((v, i) => [`fundRestakingVaultReceiptTokenMint${i}`, v.receiptTokenMint]));
            const fundReserveRestakingVaultReceiptTokens = Object.fromEntries(restakingVaults.map((v, i) => [`fundReserveRestakingVaultReceiptToken${i}`, spl.getAssociatedTokenAddressSync(v.receiptTokenMint, fund, true, v.receiptTokenProgram)]));
            const fundRestakingVaultSupportedTokenMints = Object.fromEntries(restakingVaults.map((v, i) => [`fundRestakingVaultSupportedTokenMint${i}`, v.supportedTokenMint]));
            const fundRestakingVaultReserveSupportedTokens = Object.fromEntries(restakingVaults.map((v, i) => [`fundRestakingVaultReserveSupportedToken${i}`, spl.getAssociatedTokenAddressSync(v.supportedTokenMint, v.vault, true, tokenProgram)]));

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

                fundRestakingVault0: null,
                ...fundRestakingVaults,
                fundRestakingVaultProgram0: null,
                ...fundRestakingVaultPrograms,
                fundRestakingVaultReceiptTokenMint0: null,
                ...fundRestakingVaultReceiptTokenMints,
                fundReserveRestakingVaultReceiptToken0: null,
                ...fundReserveRestakingVaultReceiptTokens,
                fundRestakingVaultSupportedTokenMint0: null,
                ...fundRestakingVaultSupportedTokenMints,
                fundRestakingVaultReserveSupportedToken0: null,
                ...fundRestakingVaultReserveSupportedTokens,
            };
        })();

        const addressBook = super.createAddressBook<keyof typeof entries>();
        const effectiveEntries = Object.fromEntries(Object.entries(entries).filter(([_, v]) => !!v));
        addressBook.addAll(effectiveEntries);
        return addressBook;
    }
}
