// import * as web3 from '@solana/web3.js';
// import * as spl from '@solana/spl-token';
// import BN from "bn.js";
//
// import {RestakingProgram, RestakingProgramAccount, RestakingProgramType} from './program';
//
// export const U64_MAX = new BN("18446744073709551615");
//
// export class RestakingFund extends RestakingProgram {
//     public static readonly fragSOLReceiptTokenMint = new web3.PublicKey('FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo');
//
//     public readonly receiptTokenMint: web3.PublicKey;
//
//     constructor({ receiptTokenMint, ...params } : ConstructorParameters<typeof RestakingProgram>[0] & {
//         receiptTokenMint: web3.PublicKey,
//     }) {
//         super(params);
//         this.receiptTokenMint = receiptTokenMint;
//         // this.addressBook = this.updateAddressBook();
//     }
//
//     private _fundAccount: RestakingProgramAccount['fundAccount'] | null = null;
//     private async getFundAccount({ refetch = false }: { refetch?: boolean } = {}): Promise<RestakingProgramAccount['fundAccount'] | null> {
//         if (!refetch && this._fundAccount) {
//             return this._fundAccount;
//         }
//         const address = web3.PublicKey.findProgramAddressSync([Buffer.from('fund'), this.receiptTokenMint.toBuffer()], this.programID)[0];
//         this._fundAccount = await this.programAccounts.fundAccount.fetchNullable(address);
//         return this._fundAccount;
//     }
//
//     public async getSupportedAssets({ refetch = false }: { refetch?: boolean } = {}) {
//         return this.getFundAccount({ refetch }).then(fundAccount => {
//             if (!fundAccount) return [];
//
//             fundAccount.oneReceiptTokenAsSol
//
//             return [
//                 new RestakingFundSupportedAsset({ assetState: fundAccount.sol, supportedToken: null }),
//                 ...fundAccount.supportedTokens.slice(0, fundAccount.numSupportedTokens).map(supportedToken => new RestakingFundSupportedAsset({ assetState: supportedToken.token, supportedToken })),
//             ];
//         })
//     }
//
//     private addressBook: ReturnType<typeof RestakingFund.prototype.updateAddressBook>;
//
//     private async updateAddressBook() {
//         const fundAccount = this.data.fundAccount ?? await this.fetchFundAccount();
//         const supportedTokens = fundAccount?.supportedTokens.slice(fundAccount!.numSupportedTokens) ?? [];
//         const normalizedToken = fundAccount?.normalizedToken;
//         const restakingVaults = fundAccount?.restakingVaults.slice(fundAccount!.numRestakingVaults) ?? [];
//
//         const entries = (() => {
//             // emit_cpi! macro
//             const programEventAuthority = web3.PublicKey.findProgramAddressSync([Buffer.from('__event_authority')], this.programID)[0];
//
//             // fund account and assets
//             const fund = web3.PublicKey.findProgramAddressSync([Buffer.from('fund'), this.receiptTokenMint.toBuffer()], this.programID)[0];
//             const fundReserve = web3.PublicKey.findProgramAddressSync([Buffer.from('fund_reserve'), this.receiptTokenMint.toBuffer()], this.programID)[0];
//             const fundTreasury = web3.PublicKey.findProgramAddressSync([Buffer.from('fund_treasury'), this.receiptTokenMint.toBuffer()], this.programID)[0];
//
//             // reward account
//             const reward = web3.PublicKey.findProgramAddressSync([Buffer.from('reward'), this.receiptTokenMint.toBuffer()], this.programID)[0];
//
//             // receipt token mint and extensions
//             const tokenProgram2022 = spl.TOKEN_2022_PROGRAM_ID;
//             const receiptTokenMint = this.receiptTokenMint;
//             const receiptTokenExtraAccountMeta = spl.getExtraAccountMetaAddress(this.receiptTokenMint, this.programID);
//             const fundReceiptTokenLock = spl.getAssociatedTokenAddressSync(this.receiptTokenMint, fund, true, tokenProgram2022);
//
//             // supported tokens
//             const tokenProgram = spl.TOKEN_PROGRAM_ID;
//             const supportedTokenMints = Object.fromEntries(supportedTokens.map((v, i) => [`supportedTokenMint${i}`, v.mint]));
//             const fundReserveSupportedTokens = Object.fromEntries(supportedTokens.map((v, i) => [`fundReserveSupportedToken${i}`, spl.getAssociatedTokenAddressSync(v.mint, fund, true, v.program)]));
//             const fundTreasurySupportedTokens = Object.fromEntries(supportedTokens.map((v, i) => [`fundTreasurySupportedToken${i}`, spl.getAssociatedTokenAddressSync(v.mint, fundTreasury, true, v.program)]));
//
//             // normalized token pool and mint
//             const normalizedTokenMint = normalizedToken?.mint ?? null;
//             const normalizedTokenPool = normalizedTokenMint ? web3.PublicKey.findProgramAddressSync([Buffer.from('nt_pool'), normalizedTokenMint.toBuffer()], this.programID)[0] : null;
//             const normalizedTokenPoolReserveSupportedTokens = normalizedTokenMint ? Object.fromEntries(supportedTokens.map((v, i) => [`normalizedTokenPoolReserveSupportedToken${i}`, spl.getAssociatedTokenAddressSync(v.mint, normalizedTokenPool!, true, v.program)])) : [];
//             const fundReserveNormalizedToken = normalizedTokenMint ? spl.getAssociatedTokenAddressSync(normalizedTokenMint, fund, true, tokenProgram) : null;
//
//             // program revenue
//             const programRevenue = this.getConstantAsPublicKey('programRevenueAddress');
//             const programRevenueSupportedTokens = Object.fromEntries(supportedTokens.map((v, i) => [`normalizedTokenPoolReserveSupportedToken${i}`, spl.getAssociatedTokenAddressSync(v.mint, programRevenue, true, v.program)]));
//
//             // restaking vaults
//             const fundRestakingVaults = Object.fromEntries(restakingVaults.map((v, i) => [`fundRestakingVault${i}`, v.vault]));
//             const fundRestakingVaultPrograms = Object.fromEntries(restakingVaults.map((v, i) => [`fundRestakingVaultProgram${i}`, v.program]));
//             const fundRestakingVaultReceiptTokenMints = Object.fromEntries(restakingVaults.map((v, i) => [`fundRestakingVaultReceiptTokenMint${i}`, v.receiptTokenMint]));
//             const fundReserveRestakingVaultReceiptTokens = Object.fromEntries(restakingVaults.map((v, i) => [`fundReserveRestakingVaultReceiptToken${i}`, spl.getAssociatedTokenAddressSync(v.receiptTokenMint, fund, true, v.receiptTokenProgram)]));
//             const fundRestakingVaultSupportedTokenMints = Object.fromEntries(restakingVaults.map((v, i) => [`fundRestakingVaultSupportedTokenMint${i}`, v.supportedTokenMint]));
//             const fundRestakingVaultReserveSupportedTokens = Object.fromEntries(restakingVaults.map((v, i) => [`fundRestakingVaultReserveSupportedToken${i}`, spl.getAssociatedTokenAddressSync(v.supportedTokenMint, v.vault, true, tokenProgram)]));
//
//             return {
//                 programEventAuthority,
//
//                 fund,
//                 fundReserve,
//                 fundTreasury,
//
//                 reward,
//
//                 tokenProgram2022,
//                 receiptTokenMint,
//                 receiptTokenExtraAccountMeta,
//                 fundReceiptTokenLock,
//
//                 tokenProgram,
//                 supportedTokenMint0: null,
//                 ...supportedTokenMints,
//                 fundReserveSupportedToken0: null,
//                 ...fundReserveSupportedTokens,
//                 fundTreasurySupportedToken0: null,
//                 ...fundTreasurySupportedTokens,
//
//                 normalizedTokenMint,
//                 normalizedTokenPool,
//                 normalizedTokenPoolReserveSupportedToken0: null,
//                 ...normalizedTokenPoolReserveSupportedTokens,
//                 fundReserveNormalizedToken,
//
//                 programRevenue,
//                 programRevenueSupportedToken0: null,
//                 ...programRevenueSupportedTokens,
//
//                 fundRestakingVault0: null,
//                 ...fundRestakingVaults,
//                 fundRestakingVaultProgram0: null,
//                 ...fundRestakingVaultPrograms,
//                 fundRestakingVaultReceiptTokenMint0: null,
//                 ...fundRestakingVaultReceiptTokenMints,
//                 fundReserveRestakingVaultReceiptToken0: null,
//                 ...fundReserveRestakingVaultReceiptTokens,
//                 fundRestakingVaultSupportedTokenMint0: null,
//                 ...fundRestakingVaultSupportedTokenMints,
//                 fundRestakingVaultReserveSupportedToken0: null,
//                 ...fundRestakingVaultReserveSupportedTokens,
//             };
//         })();
//
//         const addressBook = super.createAddressBook<keyof typeof entries>();
//         const effectiveEntries = Object.fromEntries(Object.entries(entries).filter(([_, v]) => !!v));
//         addressBook.addAll(effectiveEntries);
//         return addressBook;
//     }
// }
//
// class RestakingFundSupportedAsset {
//     public readonly data: {
//         assetState: RestakingProgramType['assetState'],
//         supportedToken: RestakingProgramType['supportedToken'] | null,
//     };
//     public readonly isSOL: boolean;
//     public readonly tokenMint: web3.PublicKey | null;
//     public readonly tokenProgram: web3.PublicKey | null;
//     public readonly decimals: number;
//     public readonly oneTokenAsSol: BN;
//     public readonly depositable: boolean;
//     public readonly withdrawable: boolean;
//     public readonly accumulatedDepositCapacityAmount: BN | null; // null means there is no cap limit
//     public readonly accumulatedDepositAmount: BN;
//
//     constructor({ assetState, supportedToken }: {
//         assetState: RestakingProgramType['assetState'],
//         supportedToken: RestakingProgramType['supportedToken'] | null,
//     }) {
//         this.data = { assetState, supportedToken };
//         this.isSOL = !supportedToken;
//         this.tokenMint = this.isSOL ? null : this.data.assetState.tokenMint;
//         this.tokenProgram = this.isSOL ? null : this.data.assetState.tokenProgram;
//         this.decimals = this.data.supportedToken?.decimals ?? 9;
//         this.oneTokenAsSol = this.data.supportedToken?.oneTokenAsSol ?? new BN(web3.LAMPORTS_PER_SOL);
//         this.depositable = this.data.assetState.depositable == 1;
//         this.withdrawable = this.data.assetState.withdrawable == 1;
//         this.accumulatedDepositCapacityAmount = this.data.assetState.accumulatedDepositCapacityAmount.eq(U64_MAX) ? null : this.data.assetState.accumulatedDepositCapacityAmount;
//         this.accumulatedDepositAmount = this.data.assetState.accumulatedDepositAmount;
//     }
// }
