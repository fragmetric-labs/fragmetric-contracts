import * as web3 from '@solana/web3.js';
import BN from 'bn.js';

import {Program, ProgramEvent, ProgramType, ProgramAccount} from "../program";
import restakingIDL from './program.idl.json';
import type {Restaking as RestakingIDL} from './program.idl';
import {dedupe} from "../cache";
import {RestakingFundSupportedAsset} from "./state";

export type { RestakingIDL };
export type RestakingProgramAccount = ProgramAccount<RestakingIDL>;
export type RestakingProgramEvent = ProgramEvent<RestakingIDL>;
export type RestakingProgramType = ProgramType<RestakingIDL>;

export const BN_U64_MAX = new BN("18446744073709551615");

export class RestakingProgram extends Program<RestakingIDL> {
    public static readonly programID = {
        mainnet: new web3.PublicKey('fragnAis7Bp6FTsMoa6YcH8UffhEw43Ph79qAiK3iF3'),
        devnet: new web3.PublicKey('frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ'),
        testnet: null,
        local: null,
    };

    public static readonly receiptTokenMint = {
        fragSOL: new web3.PublicKey('FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo'),
        fragJTO: new web3.PublicKey('FRAGJ157KSDfGvBJtCSrsTWUqFnZhrw4aC8N8LqHuoos'),
    };
    public readonly receiptTokenMint: web3.PublicKey;

    constructor({ receiptTokenMint, cluster = 'mainnet', ...args }: Partial<Omit<ConstructorParameters<typeof Program<RestakingIDL>>[0], 'idl'|'programID'>> & {
        receiptTokenMint: web3.PublicKey | keyof typeof RestakingProgram['receiptTokenMint'],
    }) {
        const idl = <RestakingIDL>restakingIDL;
        const programID = RestakingProgram.programID[cluster] ?? new web3.PublicKey(idl.address);
        super({ ...args, cluster, idl, programID });
        this.receiptTokenMint = RestakingProgram.receiptTokenMint[receiptTokenMint.toString() as keyof typeof RestakingProgram['receiptTokenMint']] ?? receiptTokenMint;
    }

    public readonly state = {
        /* internal states */
        _fund: dedupe(async (refresh: boolean = false): Promise<RestakingProgramAccount['fundAccount'] | null> => {
            const k = '_fund';
            if (!refresh && this.cache.has(k)) return this.cache.get(k);

            const address = web3.PublicKey.findProgramAddressSync([Buffer.from('fund'), this.receiptTokenMint.toBuffer()], this.programID)[0];
            return this.cache.set(k, await this.programAccounts.fundAccount.fetchNullable(address));
        }),
        _addressLookupTables: dedupe(async (refresh: boolean = false): Promise<web3.AddressLookupTableAccount[]> => {
            const k = '_addressLookupTables';
            if (!refresh && this.cache.has(k)) return this.cache.get(k);

            const fundAccount = await this.state._fund(refresh);
            if (fundAccount?.addressLookupTableEnabled) {
                const addressLookupTable = await this.connection
                    .getAddressLookupTable(fundAccount.addressLookupTableAccount, { commitment: 'confirmed' })
                    .then(res => res.value);
                if (addressLookupTable) {
                    return this.cache.set(k, [addressLookupTable]);
                }
            }
            return [];
        }),
        _pricingSourcesAccountMeta: dedupe(async (refresh: boolean = false): Promise<web3.AccountMeta[]> => {
            const k = '_pricingSourcesAccountMeta';
            if (!refresh && this.cache.has(k)) return this.cache.get(k);

            const addresses: web3.PublicKey[] = [];
            const fundAccount = await this.state._fund(refresh);
            if (fundAccount) {
                for (const supportedToken of fundAccount.supportedTokens.slice(0, fundAccount.numSupportedTokens)) {
                    addresses.push(supportedToken.pricingSource.address);
                }

                if (fundAccount.normalizedToken.enabled == 1) {
                    addresses.push(fundAccount.normalizedToken.pricingSource.address);
                }

                for (const restakingVault of fundAccount.restakingVaults.slice(0, fundAccount.numRestakingVaults)) {
                    addresses.push(restakingVault.receiptTokenPricingSource.address);
                }
            }
            return this.cache.set(k, addresses.map(address => ({ pubkey: address, isSigner: false, isWritable: false })));
        }),

        /* user facing states */
        supportedAssets: dedupe(async (refresh: boolean = false): Promise<RestakingFundSupportedAsset[]> => {
            const k = 'supportedAssets';
            if (!refresh && this.cache.has(k)) return this.cache.get(k);

            const fundAccount = await this.state._fund(refresh);
            const supportedAssets: RestakingFundSupportedAsset[] = fundAccount ? [
                { assetState: fundAccount.sol, supportedToken: null },
                ...fundAccount.supportedTokens
                    .slice(0, fundAccount.numSupportedTokens)
                    .map(supportedToken => ({ assetState: supportedToken.token, supportedToken }))
            ].map(({assetState, supportedToken}) => {
                const oneTokenAsSOL = supportedToken?.oneTokenAsSol ?? new BN(web3.LAMPORTS_PER_SOL);
                return {
                    isNativeSOL: !supportedToken,
                    tokenMint: supportedToken ? assetState.tokenMint : null,
                    tokenProgram: supportedToken ? assetState.tokenProgram : null,
                    decimals: supportedToken?.decimals ?? 9,
                    oneTokenAsSOL,
                    oneTokenAsReceiptToken: oneTokenAsSOL.div(fundAccount.oneReceiptTokenAsSol),
                    depositable: assetState.depositable == 1,
                    withdrawable: assetState.withdrawable == 1,
                    accumulatedDepositCapacityAmount: assetState.accumulatedDepositCapacityAmount.eq(BN_U64_MAX) ? null : assetState.accumulatedDepositCapacityAmount,
                    accumulatedDepositAmount: assetState.accumulatedDepositAmount,
                };
            }) : [];
            return this.cache.set(k, supportedAssets);
        }),
    };

    public readonly operator = {
        updateFundPrices: async ({ operator }: { operator: web3.PublicKey | web3.Keypair }) => {
            return this.createTransactionMessage({
                descriptions: ['Update prices of the fund assets.'],
                events: ['operatorUpdatedFundPrices'],
                instructions: await Promise.all([
                    this.programMethods
                        .operatorUpdateFundPrices()
                        .accountsPartial({
                            operator: (operator as web3.Keypair).publicKey ?? operator,
                            receiptTokenMint: this.receiptTokenMint,
                        })
                        .remainingAccounts(await this.state._pricingSourcesAccountMeta())
                        .instruction(),
                ]),
                signers: {
                    payer: operator,
                },
                addressLookupTables: await this.state._addressLookupTables(),
            });
        },
        updateNormalizedTokenPoolPrices: async ({ operator }: { operator: web3.PublicKey | web3.Keypair }) => {
            const fundAccount = await this.state._fund();
            const normalizedToken = fundAccount?.normalizedToken.enabled == 1 ? fundAccount.normalizedToken : null;
            if (!normalizedToken) {
                throw new Error(`normalized token is not enabled for the fund`);
            }
            return this.createTransactionMessage({
                descriptions: ['Update prices of the normalized token pool assets.'],
                events: ['operatorUpdatedNormalizedTokenPoolPrices'],
                instructions: await Promise.all([
                    this.programMethods
                        .operatorUpdateNormalizedTokenPoolPrices()
                        .accountsPartial({
                            operator: (operator as web3.Keypair).publicKey ?? operator,
                            normalizedTokenMint: normalizedToken.mint,
                            normalizedTokenProgram: normalizedToken.program,
                        })
                        .remainingAccounts(await this.state._pricingSourcesAccountMeta())
                        .instruction(),
                ]),
                signers: {
                    payer: operator,
                },
                addressLookupTables: await this.state._addressLookupTables(),
            });
        },
        updateRewardPools: async ({ operator }: { operator: web3.PublicKey | web3.Keypair }) => {
            return this.createTransactionMessage({
                descriptions: ['Update the reward pools.'],
                events: ['operatorUpdatedRewardPools'],
                instructions: await Promise.all([
                    this.programMethods
                        .operatorUpdateRewardPools()
                        .accountsPartial({
                            operator: (operator as web3.Keypair).publicKey ?? operator,
                            receiptTokenMint: this.receiptTokenMint,
                        })
                        .instruction(),
                ]),
                signers: {
                    payer: operator,
                },
                addressLookupTables: await this.state._addressLookupTables(),
            });
        },
        donateSOLToFund: async ({ operator, amount, offsetReceivable }: { operator: web3.PublicKey | web3.Keypair, amount: BN, offsetReceivable: boolean }) => {
            return this.createTransactionMessage({
                descriptions: [`WARNING: Donate SOL to the fund for testing.`, { amount, offsetReceivable }],
                events: ['operatorDonatedToFund'],
                instructions: await Promise.all([
                    this.programMethods
                        .operatorDonateSolToFund(amount, offsetReceivable)
                        .accountsPartial({
                            operator: (operator as web3.Keypair).publicKey ?? operator,
                            receiptTokenMint: this.receiptTokenMint,
                        })
                        .remainingAccounts(await this.state._pricingSourcesAccountMeta())
                        .instruction(),
                ]),
                signers: {
                    payer: operator,
                },
                addressLookupTables: await this.state._addressLookupTables(),
            });
        },
    };
}
