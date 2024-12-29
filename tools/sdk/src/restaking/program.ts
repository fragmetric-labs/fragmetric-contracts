import * as web3 from '@solana/web3.js';
import BN from "bn.js";

import {Program, ProgramEvent, ProgramType, ProgramAccount} from "../program";
import idlFile from './program.idl.v0.3.3.json';
import type {Restaking} from './program.idl.v0.3.3';
import {AccountMeta} from "@solana/web3.js";

export type RestakingIDL = Restaking;
export type RestakingProgramAccount = ProgramAccount<RestakingIDL>;
export type RestakingProgramEvent = ProgramEvent<RestakingIDL>;
export type RestakingProgramType = ProgramType<RestakingIDL>;

export class RestakingProgram extends Program<RestakingIDL> {
    public static readonly programID = {
        mainnet: new web3.PublicKey('fragnAis7Bp6FTsMoa6YcH8UffhEw43Ph79qAiK3iF3'),
        devnet: new web3.PublicKey('frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ'),
        testnet: null,
        local: null,
    };

    public static readonly receiptTokenMint = {
        fragSOL: new web3.PublicKey('FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo'),
    };
    public readonly receiptTokenMint: web3.PublicKey;

    public getAddressLookupTable(): web3.PublicKey | null {
        if (this.receiptTokenMint.equals(RestakingProgram.receiptTokenMint.fragSOL)) {
            // TODO: register ALT address to the fund account
            return this.idl.getConstantAsPublicKey('fragsolAddressLookupTableAddress');
        }
        return null;
    }

    constructor({ receiptTokenMint, cluster = 'mainnet', idl = <RestakingIDL>idlFile, ...args }: Partial<ConstructorParameters<typeof Program<RestakingIDL>>[0]> & {
        receiptTokenMint: web3.PublicKey | keyof typeof RestakingProgram['receiptTokenMint'],
    }) {
        const programID = RestakingProgram.programID[cluster] ?? new web3.PublicKey(idl.address);
        super({ cluster, programID, idl, ...args });
        this.receiptTokenMint = RestakingProgram.receiptTokenMint[receiptTokenMint.toString() as keyof typeof RestakingProgram['receiptTokenMint']] ?? receiptTokenMint;
    }

    public readonly state = {
        _fund: null as any,
        fund: async (refetch = false): Promise<RestakingProgramAccount['fundAccount'] | null> => {
            if (!refetch && this.state._fund) {
                return this.state._fund;
            }
            const address = web3.PublicKey.findProgramAddressSync([Buffer.from('fund'), this.receiptTokenMint.toBuffer()], this.programID)[0];
            return this.state._fund = await this.programAccounts.fundAccount.fetchNullable(address);
        },
        _addressLookupTables: null as any,
        addressLookupTables: async (refetch = false): Promise<web3.AddressLookupTableAccount[]> => {
            if (!refetch && this.state._addressLookupTables) {
                return this.state._addressLookupTables;
            }
            let address = null;
            if (this.receiptTokenMint.equals(RestakingProgram.receiptTokenMint.fragSOL)) {
                address = this.idl.getConstantAsPublicKey('fragsolAddressLookupTableAddress');
            }
            if (address) {
                const table = await this.connection
                    .getAddressLookupTable(address, { commitment: 'confirmed' })
                    .then(res => res.value);
                if (table) {
                    return this.state._addressLookupTables = [table];
                }
            }
            return [];
        },
        _pricingSourcesAccountMeta: null as any,
        pricingSourcesAccountMeta: async (refetch = false): Promise<web3.AccountMeta[]> => {
            if (!refetch && this.state._pricingSourcesAccountMeta) {
                return this.state._pricingSourcesAccountMeta;
            }
            const addresses: web3.PublicKey[] = [];
            const fundAccount = await this.state.fund(refetch);
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
            return this.state._pricingSourcesAccountMeta = addresses.map(address => ({ pubkey: address, isSigner: false, isWritable: false }));
        }
    }

    public readonly operator = {
        updateFundPrices: async ({ operator }: { operator: web3.PublicKey | web3.Keypair }) => {
            return this.createUnsignedTransactionMessage({
                descriptions: ['update prices of the fund'],
                events: ['operatorUpdatedFundPrices'],
                instructions: await Promise.all([
                    this.programMethods
                        .operatorUpdateFundPrices()
                        .accountsPartial({
                            operator: (operator as web3.Keypair).publicKey ?? operator,
                            receiptTokenMint: this.receiptTokenMint,
                        })
                        .remainingAccounts(await this.state.pricingSourcesAccountMeta())
                        .instruction(),
                ]),
                signers: {
                    payer: operator,
                },
                addressLookupTables: await this.state.addressLookupTables(),
            });
        },
        updateNormalizedTokenPoolPrices: async ({ operator }: { operator: web3.PublicKey | web3.Keypair }) => {
            const fundAccount = await this.state.fund();
            const normalizedToken = fundAccount?.normalizedToken.enabled == 1 ? fundAccount.normalizedToken : null;
            if (!normalizedToken) {
                throw new Error(`normalized token is not enabled for the fund`);
            }
            return this.createUnsignedTransactionMessage({
                descriptions: ['update prices of the normalized token pool'],
                events: ['operatorUpdatedNormalizedTokenPoolPrices'],
                instructions: await Promise.all([
                    this.programMethods
                        .operatorUpdateNormalizedTokenPoolPrices()
                        .accountsPartial({
                            operator: (operator as web3.Keypair).publicKey ?? operator,
                            normalizedTokenMint: normalizedToken.mint,
                            normalizedTokenProgram: normalizedToken.program,
                        })
                        .remainingAccounts(await this.state.pricingSourcesAccountMeta())
                        .instruction(),
                ]),
                signers: {
                    payer: operator,
                },
                addressLookupTables: await this.state.addressLookupTables(),
            });
        },
        updateRewardPools: async ({ operator }: { operator: web3.PublicKey | web3.Keypair }) => {
            return this.createUnsignedTransactionMessage({
                descriptions: ['update reward pools of the fund'],
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
                addressLookupTables: await this.state.addressLookupTables(),
            });
        },
        donateSOLToFund: async ({ operator, amount, offsetReceivable }: { operator: web3.PublicKey | web3.Keypair, amount: BN, offsetReceivable: boolean }) => {
            return this.createUnsignedTransactionMessage({
                descriptions: [`donate SOL to the fund`, {amount, offsetReceivable}],
                events: ['operatorDonatedToFund'],
                instructions: await Promise.all([
                    this.programMethods
                        .operatorDonateSolToFund(amount, offsetReceivable)
                        .accountsPartial({
                            operator: (operator as web3.Keypair).publicKey ?? operator,
                            receiptTokenMint: this.receiptTokenMint,
                        })
                        .remainingAccounts(await this.state.pricingSourcesAccountMeta())
                        .instruction(),
                ]),
                signers: {
                    payer: operator,
                },
                addressLookupTables: await this.state.addressLookupTables(),
            });
        },
    };
}
