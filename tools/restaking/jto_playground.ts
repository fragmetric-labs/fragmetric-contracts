import * as anchor from "@coral-xyz/anchor";
import {BN, web3} from "@coral-xyz/anchor";
// @ts-ignore
import * as splTokenMetadata from "@solana/spl-token-metadata";
import * as splStakePool from "@solana/spl-stake-pool";
// @ts-ignore
import * as spl from "@solana/spl-token-3.x";
import * as mpl from "@metaplex-foundation/mpl-token-metadata";
import * as umi from "@metaplex-foundation/umi";
import * as umi2 from "@metaplex-foundation/umi-bundle-defaults";
// @ts-ignore
import {AnchorPlayground, AnchorPlaygroundConfig, getLogger} from "../lib";
import {Restaking} from "../../target/types/restaking";
import {getKeychain, KEYCHAIN_ENV, KEYCHAIN_KEYS} from "./keychain";
import {IdlTypes} from "@coral-xyz/anchor/dist/cjs/program/namespace/types";
import * as ed25519 from "ed25519";

const {logger, LOG_PAD_SMALL, LOG_PAD_LARGE} = getLogger("restaking");

const MAX_CAPACITY = "18,446,744,073,709,551,615".replace(/,/g, '');

export class RestakingPlayground extends AnchorPlayground<Restaking, KEYCHAIN_KEYS> {
    public static create(env: KEYCHAIN_ENV, args?: Pick<AnchorPlaygroundConfig<Restaking, any>, "provider">) {
        return getKeychain(env).then((keychain) => {
            return new RestakingPlayground({
                keychain,
                provider: new anchor.AnchorProvider(args?.provider?.connection ?? new web3.Connection(RestakingPlayground.clusterURL[env]), new anchor.Wallet(keychain.wallet)),
            });
        });
    }

    private static readonly clusterURL: { [env in KEYCHAIN_ENV]: string } = {
        local: "http://0.0.0.0:8899",
        devnet: "https://api.devnet.solana.com",
        mainnet: "https://api.mainnet-beta.solana.com",
    };

    public get isLocalnet(): boolean {
        return this.connection.rpcEndpoint == RestakingPlayground.clusterURL.local;
    }

    public get isDevnet(): boolean {
        return this.connection.rpcEndpoint == RestakingPlayground.clusterURL.devnet;
    }

    public get isMainnet(): boolean {
        return this.connection.rpcEndpoint == RestakingPlayground.clusterURL.mainnet;
    }

    private constructor(args: Pick<AnchorPlaygroundConfig<Restaking, any>, "provider" | "keychain">) {
        super({
            provider: args.provider,
            keychain: args.keychain,
            idl: require("../../target/idl/restaking.json") as Restaking,
        });
    }

    public get initializeSteps() {
        if (this._initializeSteps) return this._initializeSteps;
        return (this._initializeSteps = this._getInitializeSteps());
    }

    private _initializeSteps: ReturnType<typeof this._getInitializeSteps>;

    private _getInitializeSteps() {
        return [
            () => this.runAdminInitializeFragJTOTokenMint(), // 0
            () => this.runAdminInitializeOrUpdateFundAccount(), // 1
            () => this.runAdminInitializeOrUpdateRewardAccount(), // 2
            () => this.runAdminInitializeFragJTOExtraAccountMetaList(), // 3
            () => this.runFundManagerInitializeRewardPools(), // 4
            () => this.runFundManagerSettleReward({ poolName: "bonus", rewardName: "fPoint", amount: new BN(0) }), // 5
            () => this.runFundManagerInitializeFundSupportedTokens(), // 6
            () => this.runFundManagerInitializeFundJitoRestakingVaults(), // 7
            () => this.runFundManagerUpdateFundConfigurations(), // 8
            () => this.runAdminInitializeWfragJTOTokenMint(), // 9
            () => this.runAdminInitializeOrUpdateFundWrapAccountRewardAccount(), // 10
            () => this.runFundManagerInitializeFundWrappedToken(), // 11
            () => this.runFundManagerAddRestakingVaultCompoundingRewardTokens(), // 12
            () => this.runFundManagerAddTokenSwapStrategies(), // 13
            () => this.runFundManagerAddJitoRestakingVaultDelegations(), // 14
        ];
    }

    public async getOrCreateKnownAddressLookupTable() {
        if (this._knownAddressLookupTableAddress) {
            return this._knownAddressLookupTableAddress;
        }

        const existinglookupTableAddress = this.getConstantAsPublicKey('fragjtoAddressLookupTableAddress');
        const existingLookupTable = await this.connection.getAccountInfo(existinglookupTableAddress).catch(() => null);
        if (existingLookupTable) {
            this._knownAddressLookupTableAddress = existinglookupTableAddress;
        } else {
            const authority = this.keychain.getKeypair('ADMIN').publicKey;
            const payer = this.wallet.publicKey;
            const recentSlot = await this.connection.getSlot({commitment: 'recent'});
            const [createIx, lookupTableAddress] = web3.AddressLookupTableProgram.createLookupTable({
                authority,
                payer,
                recentSlot,
            });
            await this.run({
                instructions: [createIx],
                signerNames: ['ADMIN']
            });
            logger.notice('created a lookup table for known addresses:'.padEnd(LOG_PAD_LARGE), lookupTableAddress.toString());
            this._knownAddressLookupTableAddress = lookupTableAddress;
            await this.sleep(2);
        }

        await this.updateKnownAddressLookupTable();
        await this.setAddressLookupTableAddresses([this._knownAddressLookupTableAddress]);
    }

    public async updateKnownAddressLookupTable() {
        const authority = this.keychain.getKeypair('ADMIN').publicKey;
        const payer = this.wallet.publicKey;
        const lookupTable = await this.connection
            .getAddressLookupTable(this._knownAddressLookupTableAddress, { commitment: 'confirmed' })
            .then(res => res.value);
        const existingAddresses = new Set(lookupTable.state.addresses.map(a => a.toString()));
        logger.info("current lookup table addresses", lookupTable.state.addresses);

        // prepare update (knownAddress + pricing source)
        const addresses = (() => {
            const addresses = [
                ...(Object.values(this.knownAddress).filter(address => typeof address != 'function').flat() as web3.PublicKey[]),
                ...this.pricingSourceAccounts.map(meta => meta.pubkey),
            ]
            .map(address => address.toString())
            .filter(address => !existingAddresses.has(address));
            return [...new Set(addresses)].map(address => new web3.PublicKey(address));
        })();

        // do update
        const listOfAddressList = addresses.reduce((listOfAddressList, address) =>  {
            if (listOfAddressList[0].length == 27) { // 27 (addresses) + 5 (admin/authority, payer, alt_program, alt, system_program)
                listOfAddressList.unshift([address]);
            } else {
                listOfAddressList[0].push(address);
            }
            return listOfAddressList;
        }, [[]] as web3.PublicKey[][]);
        logger.info("newly added lookup table addresses", addresses);

        for (let addresses of listOfAddressList) {
            if (addresses.length == 0) continue;
            await this.run({
                instructions: [
                    web3.AddressLookupTableProgram.extendLookupTable({
                        lookupTable: this._knownAddressLookupTableAddress,
                        authority,
                        payer,
                        addresses,
                    }),
                ],
                signerNames: ['ADMIN'],
            });
        }

        if (addresses.length > 0) {
            logger.notice('updated a lookup table for known addresses:'.padEnd(LOG_PAD_LARGE), this._knownAddressLookupTableAddress.toString());
        }
    }

    private _knownAddressLookupTableAddress?: web3.PublicKey;

    public get knownAddress() {
        if (this._knownAddress) return this._knownAddress;
        return (this._knownAddress = this._getKnownAddress());
    }

    private _knownAddress: ReturnType<typeof this._getKnownAddress>;

    private _getKnownAddress() {
        // for emit_cpi!
        const programEventAuthority = web3.PublicKey.findProgramAddressSync([Buffer.from("__event_authority")], this.programId)[0];

        // fragJTO
        const fragJTOTokenMint = this.getConstantAsPublicKey("fragjtoMintAddress");
        const fragJTOTokenMintBuf = fragJTOTokenMint.toBuffer();
        const fragJTOExtraAccountMetasAccount = spl.getExtraAccountMetaAddress(fragJTOTokenMint, this.programId);

        // wfragJTO
        const wfragJTOTokenMint = this.getConstantAsPublicKey('fragjtoWrappedTokenMintAddress');

        // JTO
        const jtoTokenMint = this.supportedTokenMetadata['JTO'].mint;

        // fragJTO jito JTO VRT
        const fragJTOJitoJTOVRTMint = this.getConstantAsPublicKey('fragjtoJitoJtoVaultReceiptTokenMintAddress');

        // fragJTO fund & ATAs
        const [fragJTOFund] = web3.PublicKey.findProgramAddressSync([Buffer.from("fund"), fragJTOTokenMintBuf], this.programId);
        const [fragJTOFundReserveAccount] = web3.PublicKey.findProgramAddressSync([Buffer.from("fund_reserve"), fragJTOTokenMintBuf], this.programId);
        const fragJTOFundSupportedTokenReserveAccount = (symbol: keyof typeof this.supportedTokenMetadata) =>
            spl.getAssociatedTokenAddressSync(this.supportedTokenMetadata[symbol].mint, fragJTOFundReserveAccount, true, this.supportedTokenMetadata[symbol].program);
        const fragJTOFundSupportedTokenReserveAccounts = Object.keys(this.supportedTokenMetadata).reduce((obj, symbol) => ({
            [`fragJTOFundSupportedTokenReserveAccount_${symbol}`]: fragJTOFundSupportedTokenReserveAccount(symbol as any),
            ...obj,
        }), {} as { string: web3.PublicKey });
        const [fragJTOFundTreasuryAccount] = web3.PublicKey.findProgramAddressSync([Buffer.from("fund_treasury"), fragJTOTokenMintBuf], this.programId);
        const fragJTOFundSupportedTokenTreasuryAccount = (symbol: keyof typeof this.supportedTokenMetadata) =>
            spl.getAssociatedTokenAddressSync(this.supportedTokenMetadata[symbol].mint, fragJTOFundTreasuryAccount, true, this.supportedTokenMetadata[symbol].program);
        const fragJTOFundSupportedTokenTreasuryAccounts = Object.keys(this.supportedTokenMetadata).reduce((obj, symbol) => ({
            [`fragJTOFundSupportedTokenTreasuryAccount_${symbol}`]: fragJTOFundSupportedTokenTreasuryAccount(symbol as any),
            ...obj,
        }), {} as { string: web3.PublicKey });
        const [fragJTOFundWrapAccount] = web3.PublicKey.findProgramAddressSync([Buffer.from("fund_wrap"), fragJTOTokenMintBuf], this.programId);
        const fragJTOFundReceiptTokenWrapAccount = spl.getAssociatedTokenAddressSync(
            fragJTOTokenMint,
            fragJTOFundWrapAccount,
            true,
            spl.TOKEN_2022_PROGRAM_ID,
        );

        const fragJTOFundReceiptTokenLockAccount = spl.getAssociatedTokenAddressSync(
            fragJTOTokenMint,
            fragJTOFund,
            true,
            spl.TOKEN_2022_PROGRAM_ID,
        );

        const fragJTOFundJitoJTOVRTAccount = spl.getAssociatedTokenAddressSync(
            fragJTOJitoJTOVRTMint,
            fragJTOFundReserveAccount,
            true,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );

        const fragJTOUserFund = (user: web3.PublicKey) => web3.PublicKey.findProgramAddressSync([Buffer.from("user_fund"), fragJTOTokenMintBuf, user.toBuffer()], this.programId)[0];
        const fragJTOUserTokenAccount = (user: web3.PublicKey) => spl.getAssociatedTokenAddressSync(fragJTOTokenMint, user, false, spl.TOKEN_2022_PROGRAM_ID);
        const wfragJTOUserTokenAccount = (user: web3.PublicKey) => spl.getAssociatedTokenAddressSync(wfragJTOTokenMint, user, false);
        const userSupportedTokenAccount = (user: web3.PublicKey, symbol: keyof typeof this.supportedTokenMetadata) =>
            spl.getAssociatedTokenAddressSync(this.supportedTokenMetadata[symbol].mint, user, false, this.supportedTokenMetadata[symbol].program);

        // reward
        const [fragJTOReward] = web3.PublicKey.findProgramAddressSync([Buffer.from("reward"), fragJTOTokenMintBuf], this.programId);
        const fragJTOUserReward = (user: web3.PublicKey) => web3.PublicKey.findProgramAddressSync([Buffer.from("user_reward"), fragJTOTokenMintBuf, user.toBuffer()], this.programId)[0];
        const fragJTOFundWrapAccountReward = fragJTOUserReward(fragJTOFundWrapAccount);

        // jito
        const jitoVaultProgram = this.getConstantAsPublicKey('jitoVaultProgramId');
        const jitoVaultProgramFeeWallet = this.getConstantAsPublicKey('jitoVaultProgramFeeWallet');
        const jitoVaultFeeWallet = this.keychain.getPublicKey('ADMIN');
        const jitoVaultConfig = this.getConstantAsPublicKey('jitoVaultConfigAddress');
        const jitoRestakingProgram = this.getConstantAsPublicKey('jitoRestakingProgramId');
        const jitoRestakingConfig = this.getConstantAsPublicKey('jitoRestakingConfigAddress');

        // fragJTO jito vault
        const fragJTOJitoVaultUpdateStateTracker = (vaultAccount: web3.PublicKey) => {
            return (slot: BN, epochLength: BN) => {
                let ncnEpoch = slot.div(epochLength).toBuffer('le', 8);
                return web3.PublicKey.findProgramAddressSync([Buffer.from('vault_update_state_tracker'), vaultAccount.toBuffer(), ncnEpoch], jitoVaultProgram)[0];
            };
        }

        // fragJTO jito JTO vault
        const fragJTOJitoJTOVaultAccount = this.getConstantAsPublicKey('fragjtoJitoJtoVaultAccountAddress');
        const fragJTOJitoJTOVaultUpdateStateTracker = fragJTOJitoVaultUpdateStateTracker(fragJTOJitoJTOVaultAccount);
        const fragJTOJitoJTOVaultTokenAccount = spl.getAssociatedTokenAddressSync(
            jtoTokenMint,
            fragJTOJitoJTOVaultAccount,
            true,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        const fragJTOJitoJTOVaultProgramFeeWalletTokenAccount = spl.getAssociatedTokenAddressSync(
            fragJTOJitoJTOVRTMint,
            jitoVaultProgramFeeWallet,
            true,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        const fragJTOJitoJTOVaultFeeWalletTokenAccount = spl.getAssociatedTokenAddressSync(
            fragJTOJitoJTOVRTMint,
            jitoVaultFeeWallet,
            false,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );

        // Revenue
        const programRevenueAccount = new web3.PublicKey(this.getConstant('programRevenueAddress'));
        const programSupportedTokenRevenueAccount = (symbol: keyof typeof this.supportedTokenMetadata) =>
            spl.getAssociatedTokenAddressSync(this.supportedTokenMetadata[symbol].mint, programRevenueAccount, true, this.supportedTokenMetadata[symbol].program);
        const programSupportedTokenRevenueAccounts = Object.keys(this.supportedTokenMetadata).reduce((obj, symbol) => ({
            [`programSupportedTokenRevenueAccount_${symbol}`]: programSupportedTokenRevenueAccount(symbol as any),
            ...obj,
        }), {} as { string: web3.PublicKey });

        return {
            // for emit_cpi!
            programEventAuthority,
            // fragJTO
            fragJTOTokenMint,
            fragJTOExtraAccountMetasAccount,
            // wfragJTO
            wfragJTOTokenMint,
            // JTO
            jtoTokenMint,
            // fragJTO jito JTO VRT
            fragJTOJitoJTOVRTMint,
            // fragJTO fund & ATAs
            fragJTOFund,
            fragJTOFundReserveAccount,
            fragJTOFundSupportedTokenReserveAccount,
            ...fragJTOFundSupportedTokenReserveAccounts,
            fragJTOFundTreasuryAccount,
            fragJTOFundSupportedTokenTreasuryAccount,
            ...fragJTOFundSupportedTokenTreasuryAccounts,
            fragJTOFundWrapAccount,
            fragJTOFundReceiptTokenWrapAccount,
            fragJTOFundReceiptTokenLockAccount,
            fragJTOFundJitoJTOVRTAccount,
            fragJTOUserFund,
            fragJTOUserTokenAccount,
            wfragJTOUserTokenAccount,
            userSupportedTokenAccount,
            // reward
            fragJTOReward,
            fragJTOUserReward,
            fragJTOFundWrapAccountReward,
            // jito
            jitoVaultProgram,
            jitoVaultProgramFeeWallet,
            jitoVaultFeeWallet,
            jitoVaultConfig,
            jitoRestakingProgram,
            jitoRestakingConfig,
            // fragJTO jito JTO vault
            fragJTOJitoJTOVaultAccount,
            fragJTOJitoJTOVaultUpdateStateTracker,
            fragJTOJitoJTOVaultTokenAccount,
            fragJTOJitoJTOVaultProgramFeeWalletTokenAccount,
            fragJTOJitoJTOVaultFeeWalletTokenAccount,
            // program revenue
            programRevenueAccount,
            programSupportedTokenRevenueAccount,
            ...programSupportedTokenRevenueAccounts,

            tokenProgram: spl.TOKEN_PROGRAM_ID,
            token2022Program: spl.TOKEN_2022_PROGRAM_ID,
            systemProgram: web3.SystemProgram.programId,
        };
    }

    public readonly fragJTODecimals = 9;
    public readonly wfragJTODecimals = 9;
    public readonly vrtDecimals = 9;

    public get supportedTokenMetadata() {
        if (this._supportedTokenMetadata) return this._supportedTokenMetadata;
        return (this._supportedTokenMetadata = this._getSupportedTokenMetadata());
    }

    private _supportedTokenMetadata: ReturnType<typeof this._getSupportedTokenMetadata>;

    private _getSupportedTokenMetadata() {
        if (this.isDevnet) {
            return {
                JTO: {
                    mint: this.getConstantAsPublicKey("devnetJtoMintAddress"),
                    program: spl.TOKEN_PROGRAM_ID,
                    pricingSourceAddress: this.getConstantAsPublicKey("devnetJtoLiquidityPoolAddress"),
                    pricingSource: {
                        orcaDexLiquidityPool: {
                            address: this.getConstantAsPublicKey("devnetJtoLiquidityPoolAddress"),
                        },
                    },
                    decimals: 9,
                },
            };
        } else {
            // for 'localnet', it would be cloned from mainnet
            return {
                JTO: {
                    mint: this.getConstantAsPublicKey("mainnetJtoMintAddress"),
                    program: spl.TOKEN_PROGRAM_ID,
                    pricingSourceAddress: this.getConstantAsPublicKey("mainnetJtoLiquidityPoolAddress"),
                    pricingSource: {
                        orcaDexLiquidityPool: {
                            address: this.getConstantAsPublicKey("mainnetJtoLiquidityPoolAddress"),
                        },
                    },
                    decimals: 9,
                },
            };
        }
    }

    public get restakingVaultMetadata() {
        if (this._restakingVaultMetadata) return this._restakingVaultMetadata;
        return (this._restakingVaultMetadata = this._getRestakingVaultMetadata());
    }

    private _restakingVaultMetadata: ReturnType<typeof this._getRestakingVaultMetadata>;

    private _getRestakingVaultMetadata() {
        if (this.isDevnet) {
            return {
                jitoJTOVault: {
                    VSTMint: this.knownAddress.jtoTokenMint,
                    VRTMint: this.knownAddress.fragJTOJitoJTOVRTMint,
                    vault: this.getConstantAsPublicKey("fragjtoJitoJtoVaultAccountAddress"),
                    operators: {},
                    program: this.getConstantAsPublicKey("jitoVaultProgramId"),
                    programFeeWalletTokenAccount: this.knownAddress.fragJTOJitoJTOVaultProgramFeeWalletTokenAccount,
                    feeWalletTokenAccount: this.knownAddress.fragJTOJitoJTOVaultFeeWalletTokenAccount,
                    vaultTokenAccount: this.knownAddress.fragJTOJitoJTOVaultTokenAccount,
                    fundVRTAccount: this.knownAddress.fragJTOFundJitoJTOVRTAccount,
                    compoundingRewards: [
                        this.rewardTokenMetadata["jitoSOL"].mint,
                    ],
                },
            };
        } else {
            // for 'localnet', it would be cloned from mainnet
            return {
                jitoJTOVault: {
                    VSTMint: this.knownAddress.jtoTokenMint,
                    VRTMint: this.knownAddress.fragJTOJitoJTOVRTMint,
                    vault: this.getConstantAsPublicKey("fragjtoJitoJtoVaultAccountAddress"),
                    operators: {
                        Everstake: new web3.PublicKey("FzZ9EXmHv7ANCXijpALUBzCza6wYNprnsfaEHuoNx9sE"),
                        Luganodes: new web3.PublicKey("LKFpfXtBkH5b7D9mo8dPcjCLZCZpmLQC9ELkbkyVdah"),
                        PierTwo: new web3.PublicKey("GZxp4e2Tm3Pw9GyAaxuF6odT3XkRM96jpZkp3nxhoK4Y"),
                        Temporal: new web3.PublicKey("CA8PaNSoFWzvbCJ2oK3QxBEutgyHSTT5omEptpj8YHPY"),
                        ChorusOne: new web3.PublicKey("7yofWXChEHkPTSnyFdKx2Smq5iWVbGB4P1dkdC6zHWYR"),
                        KILN: new web3.PublicKey("29rxXT5zbTR1ctiooHtb1Sa1TD4odzhQHsrLz3D78G5w"),
                        Helius: new web3.PublicKey("BFEsrxFPsBcY2hR5kgyfKnpwgEc8wYQdngvRukLQXwG2"),
                        Hashkey: new web3.PublicKey("2sHNuid4rus4sK2EmndLeZcPNKkgzuEoc8Vro3PH2qop"),
                        InfStones: new web3.PublicKey("5TGRFaLy3eF93pSNiPamCgvZUN3gzdYcs7jA3iCAsd1L"),
                    },
                    program: this.getConstantAsPublicKey("jitoVaultProgramId"),
                    programFeeWalletTokenAccount: this.knownAddress.fragJTOJitoJTOVaultProgramFeeWalletTokenAccount,
                    feeWalletTokenAccount: this.knownAddress.fragJTOJitoJTOVaultFeeWalletTokenAccount,
                    vaultTokenAccount: this.knownAddress.fragJTOJitoJTOVaultTokenAccount,
                    fundVRTAccount: this.knownAddress.fragJTOFundJitoJTOVRTAccount,
                    compoundingRewards: [
                        this.rewardTokenMetadata["jitoSOL"].mint,
                    ],
                },
            };
        }
    }

    public get rewardTokenMetadata() {
        if (this._rewardTokenMetadata) return this._getRewardTokenMetadata();
        return (this._rewardTokenMetadata = this._getRewardTokenMetadata());
    }
    private _rewardTokenMetadata: ReturnType<typeof this._getRewardTokenMetadata>;
    private _getRewardTokenMetadata() {
        if (this.isDevnet) {
            return {
                jitoSOL: {
                    mint: this.getConstantAsPublicKey("devnetJitosolMintAddress"),
                    program: spl.TOKEN_PROGRAM_ID,
                    decimals: 9,
                }
            }
        } else {
            return {
                jitoSOL: {
                    mint: this.getConstantAsPublicKey("mainnetJitosolMintAddress"),
                    program: spl.TOKEN_PROGRAM_ID,
                    decimals: 9,
                    // for test
                    airdropSource: {
                        splStakePool: {
                            address: this.getConstantAsPublicKey("mainnetJitosolStakePoolAddress"),
                        }
                    }
                }
            }
        }
    }

    public get pricingSourceAccounts() {
        if (this._pricingSourceAccounts) return this._pricingSourceAccounts;
        return (this._pricingSourceAccounts = this._getPricingSourceAccounts());
    }

    private _pricingSourceAccounts: ReturnType<typeof this._getPricingSourceAccounts>;

    private _getPricingSourceAccounts(): web3.AccountMeta[] {
        return [
            ...Object.values(this.supportedTokenMetadata).map((v) => {
                return {
                    pubkey: v.pricingSourceAddress,
                    isSigner: false,
                    isWritable: false,
                };
            }),
            ...Object.values(this.restakingVaultMetadata).map((v) => {
                return {
                    pubkey: v.vault,
                    isSigner: false,
                    isWritable: false,
                }
            }),
        ];
    }

    public get tokenSwapStrategies() {
        if (this._tokenSwapStrategies) return this._tokenSwapStrategies;
        return (this._tokenSwapStrategies = this._getTokenSwapStrategies());
    }
    private _tokenSwapStrategies: ReturnType<typeof this._getTokenSwapStrategies>;
    private _getTokenSwapStrategies() {
        return [
            {
                fromTokenMint: new web3.PublicKey("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn"),
                toTokenMint: this.knownAddress.jtoTokenMint,
                tokenSwapSource: {
                    orcaDexLiquidityPool: {
                        address: new web3.PublicKey("G2FiE1yn9N9ZJx5e1E2LxxMnHvb1H3hCuHLPfKJ98smA"),
                    }
                }
            }
        ]
    }

    public async tryAirdropJTO(account: web3.PublicKey, lamports: BN = new BN(100 * web3.LAMPORTS_PER_SOL)) {
        const symbol = 'JTO'
        const token = this.supportedTokenMetadata[symbol];
        const ata = spl.getAssociatedTokenAddressSync(
            token.mint,
            account,
            false,
            token.program,
        );

        await this.run({
            instructions: [
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    ata,
                    account,
                    token.mint,
                    token.program,
                ),
                spl.createMintToInstruction(
                    token.mint,
                    ata,
                    this.keychain.getPublicKey('ALL_MINT_AUTHORITY'),
                    lamports.toNumber(),
                    [],
                    token.program,
                ),
            ],
            signerNames: ["ALL_MINT_AUTHORITY"],
        })

        const balance = await this.getUserSupportedTokenAccount(account, symbol).then(a => new BN(a.amount.toString()));
        logger.debug(`JTO airdropped (+${this.lamportsToX(lamports, token.decimals, symbol)}): ${this.lamportsToX(balance, token.decimals, 'JTO')}`.padEnd(LOG_PAD_LARGE), ata.toString());
    }

    public async tryAirdropRewardToken(vault: web3.PublicKey, symbol: keyof typeof this.rewardTokenMetadata, lamports: BN) {
        const token = this.rewardTokenMetadata[symbol];
        let balanceBefore: BN;

        await (async () => {
            const ata = await spl.getOrCreateAssociatedTokenAccount(
                this.connection,
                this.wallet,
                token.mint,
                vault,
                true,
                "confirmed",
                {
                    skipPreflight: false,
                    commitment: "confirmed",
                },
                token.program
            );
            balanceBefore = new BN(ata.amount.toString());

            // If reward token is SPL stake pool token
            const splStakePoolAddress: web3.PublicKey | null = token.airdropSource["splStakePool"]?.address ?? null;
            if (splStakePoolAddress) {
                const {instructions, signers} = await splStakePool.depositSol(this.connection, splStakePoolAddress, this.wallet.publicKey, lamports.toNumber(), ata.address);
                return await this.run({instructions, signers});
            }

            // As a fallback, mint by authority
            return await this.run({
                instructions: [
                    spl.createMintToInstruction(
                        token.mint,
                        ata.address,
                        this.keychain.getPublicKey("ALL_MINT_AUTHORITY"),
                        lamports.toNumber(),
                        [],
                        token.program,
                    ),
                ],
                signerNames: ["ALL_MINT_AUTHORITY"],
            })
        })();

        const ata = await (async () => {
            const ata = spl.getAssociatedTokenAddressSync(
                token.mint,
                vault,
                true,
                token.program,
            );
            return await spl.getAccount(this.connection, ata, "confirmed", token.program);
        })();
        const balance = new BN(ata.amount.toString());
        const balanceDiff = balance.sub(balanceBefore);
        logger.debug(`${symbol} reward deposited (+${this.lamportsToX(balanceDiff, token.decimals, symbol)}): ${this.lamportsToX(balance, token.decimals, symbol)}`.padEnd(LOG_PAD_LARGE), ata.address.toString());
    }

    public getUserSupportedTokenAccount(user: web3.PublicKey, symbol: keyof typeof this.supportedTokenMetadata) {
        return spl.getAccount(
            this.connection,
            this.knownAddress.userSupportedTokenAccount(user, symbol),
            "confirmed",
            this.supportedTokenMetadata[symbol].program
        );
    }

    public getUserFragJTOAccount(user: web3.PublicKey) {
        return spl.getAccount(
            this.connection,
            this.knownAddress.fragJTOUserTokenAccount(user),
            "confirmed",
            spl.TOKEN_2022_PROGRAM_ID
        );
    }

    public async getOrCreateUserFragJTOAccount(user: web3.PublicKey) {
        return await spl.getOrCreateAssociatedTokenAccount(
            this.connection,
            this.wallet,
            this.knownAddress.fragJTOTokenMint,
            user,
            false,
            'confirmed',
            {
                commitment: 'confirmed',
            },
            spl.TOKEN_2022_PROGRAM_ID,
        )
    }

    public getUserWfragJTOAccount(user: web3.PublicKey) {
        return spl.getAccount(
            this.connection,
            this.knownAddress.wfragJTOUserTokenAccount(user),
            "confirmed",
        )
    }

    public getFragJTOFundSupportedTokenTreasuryAccountBalance(symbol: keyof typeof this.supportedTokenMetadata) {
        return spl.getAccount(
            this.connection,
            this.knownAddress.fragJTOFundSupportedTokenTreasuryAccount(symbol),
            "confirmed",
            this.supportedTokenMetadata[symbol].program
        ).then(v => new BN(v.amount.toString()));
    }

    public getProgramSupportedTokenRevenueAccountBalance(symbol: keyof typeof this.supportedTokenMetadata) {
        return spl.getAccount(
            this.connection,
            this.knownAddress.programSupportedTokenRevenueAccount(symbol),
            "confirmed",
            this.supportedTokenMetadata[symbol].program
        ).then(v => new BN(v.amount.toString()));
    }

    public getProgramRevenueAccountBalance() {
        return this.connection.getAccountInfo(this.knownAddress.programRevenueAccount).then(v => new BN(v.lamports));
    }

    public getFragJTOFundSupportedTokenReserveAccount(symbol: keyof typeof this.supportedTokenMetadata) {
        return spl.getAccount(
            this.connection,
            this.knownAddress.fragJTOFundSupportedTokenReserveAccount(symbol),
            "confirmed",
            this.supportedTokenMetadata[symbol].program
        );
    }

    public getFragJTOFundSupportedTokenReserveAccountByMintAddress(mint: web3.PublicKey) {
        for (const [symbol, token] of Object.entries(this.supportedTokenMetadata)) {
            if (mint.toString() != token.mint.toString()) continue;
            return spl.getAccount(
                this.connection,
                this.knownAddress.fragJTOFundSupportedTokenReserveAccount(symbol as any),
                "confirmed",
                token.program,
            );
        }
        throw new Error("fund supported token account not found")
    }

    public getFragJTOFundVRTAccount(symbol: keyof typeof this.restakingVaultMetadata) {
            let vault = this.restakingVaultMetadata[symbol];
            let account = spl.getAssociatedTokenAddressSync(
                vault.VRTMint,
                this.knownAddress.fragJTOFundReserveAccount,
                true,
                spl.TOKEN_PROGRAM_ID,
                spl.ASSOCIATED_TOKEN_PROGRAM_ID,
            );
            return spl.getAccount(
                this.connection,
                account,
                "confirmed",
                spl.TOKEN_PROGRAM_ID,
            );
    }

    public getFragJTOFundReceiptTokenWrapAccount() {
        return spl.getAccount(
            this.connection,
            this.knownAddress.fragJTOFundReceiptTokenWrapAccount,
            "confirmed",
            spl.TOKEN_2022_PROGRAM_ID
        );
    }

    public getFragJTOFundReceiptTokenLockAccount() {
        return spl.getAccount(
            this.connection,
            this.knownAddress.fragJTOFundReceiptTokenLockAccount,
            "confirmed",
            spl.TOKEN_2022_PROGRAM_ID
        );
    }

    public getUserFragJTOFundAccount(user: web3.PublicKey) {
        return this.account.userFundAccount.fetch(this.knownAddress.fragJTOUserFund(user), "confirmed");
    }

    public getUserFragJTORewardAccount(user: web3.PublicKey) {
        return this.account.userRewardAccount.fetch(this.knownAddress.fragJTOUserReward(user), "confirmed");
    }

    public getFragJTOFundWrapAccountRewardAccount() {
        return this.getUserFragJTORewardAccount(this.knownAddress.fragJTOFundWrapAccount);
    }

    public getFragJTORewardAccount() {
        return this.account.rewardAccount.fetch(this.knownAddress.fragJTOReward, "confirmed");
    }

    public getFragJTOFundAccount() {
        return this.account.fundAccount.fetch(this.knownAddress.fragJTOFund, "confirmed");
    }

    public getFragJTOFundReserveAccountBalance() {
        return this.connection.getBalance(this.knownAddress.fragJTOFundReserveAccount, "confirmed")
            .then(v => new BN(v));
    }

    public getFragJTOFundTreasuryAccountBalance() {
        return this.connection.getBalance(this.knownAddress.fragJTOFundTreasuryAccount, "confirmed")
            .then(v => new BN(v));
    }

    public getFragJTOJitoJTOVaultTokenAccountBalance() {
        return this.connection.getTokenAccountBalance(this.knownAddress.fragJTOJitoJTOVaultTokenAccount, "confirmed")
            .then(v => new BN(v.value.amount));
    }

    public getFragJTOTokenMint() {
        return spl.getMint(
            // @ts-ignore
            this.connection,
            this.knownAddress.fragJTOTokenMint,
            "confirmed",
            spl.TOKEN_2022_PROGRAM_ID
        );
    }

    public async getJitoRestakingVault(symbol: keyof typeof this.restakingVaultMetadata) {
        const restakingVault = this.restakingVaultMetadata[symbol];
        let vaultData = await this.connection.getAccountInfo(restakingVault.vault).then(a => a.data);
        let expected = vaultData.length;

        const checkLengthDecreased = (decreased: number, msg: string) => {
            expected -= decreased;
            if (vaultData.length != expected) {
                throw msg;
            }
        }
        const parsePubkey = (msg?: string) => {
            const key = new web3.PublicKey(Uint8Array.from(vaultData.subarray(0, 32)));
            vaultData = vaultData.subarray(32);
            checkLengthDecreased(32, msg ?? "");
            return key;
        };
        const parseValue = (msg?: string) => {
            const value = vaultData.readBigUInt64LE();
            vaultData = vaultData.subarray(8);
            checkLengthDecreased(8, msg ?? "");
            return value;
        };
        const parseBps = (msg?: string) => {
            const bps = vaultData.readUInt16LE();
            vaultData = vaultData.subarray(2);
            checkLengthDecreased(2, msg ?? "");
            return bps;
        };
        const parseByte = (msg? : string) => {
            const byte = vaultData[0];
            vaultData = vaultData.subarray(1);
            checkLengthDecreased(1, msg ?? "");
            return byte;
        };
        
        parseValue();
        const base = parsePubkey();
        const vrtMint = parsePubkey();
        const supportedMint = parsePubkey();
        const vrtSupply = parseValue();
        const tokensDeposited = parseValue();
        const depositCapacity = parseValue();
        const stakedAmount = parseValue();
        const enqueuedForCooldownAmount = parseValue();
        const coolingDownAmount = parseValue();
        const delegationState = {stakedAmount, enqueuedForCooldownAmount, coolingDownAmount};
        vaultData = vaultData.subarray(256);
        checkLengthDecreased(256, "reserve");
        const additionalAssetsNeedUnstaking = parseValue();
        const vrtEnqueuedForCooldownAmount = parseValue();
        const vrtCoolingDownAmount = parseValue();
        const vrtReadyToClaimAmount = parseValue();
        const admin = parsePubkey();
        const delegationAdmin = parsePubkey();
        const operatorAdmin = parsePubkey();
        const ncnAdmin = parsePubkey();
        const slasherAdmin = parsePubkey();
        const capacityAdmin = parsePubkey();
        const feeAdmin = parsePubkey();
        const delegateAssetAdmin = parsePubkey();
        const feeWallet = parsePubkey();
        const mintBurnAdmin = parsePubkey();
        const metadataAdmin = parsePubkey();
        const vaultIndex = parseValue();
        const ncnCount = parseValue();
        const operatorCount = parseValue();
        const slasherCount = parseValue();
        const lastFeeChangeSlot = parseValue();
        const lastFullStateUpdateSlot = parseValue();
        const depositFeeBps = parseBps();
        const withdrawalFeeBps = parseBps();
        const nextWithdrawalFeeBps = parseBps();
        const rewardFeeBps = parseBps();
        const programFeeBps = parseBps();
        const bump = parseByte();
        const isPaused = (parseByte() > 0);
        const lastStartStateUpdateSlot = parseValue();
        vaultData = vaultData.subarray(251);
        checkLengthDecreased(251, "done");

        return {
            base,
            vrtMint,
            supportedMint,
            vrtSupply,
            tokensDeposited,
            depositCapacity,
            delegationState,
            additionalAssetsNeedUnstaking,
            vrtEnqueuedForCooldownAmount,
            vrtCoolingDownAmount,
            vrtReadyToClaimAmount,
            admin,
            delegationAdmin,
            operatorAdmin,
            ncnAdmin,
            slasherAdmin,
            capacityAdmin,
            feeAdmin,
            delegateAssetAdmin,
            feeWallet,
            mintBurnAdmin,
            metadataAdmin,
            vaultIndex,
            ncnCount,
            operatorCount,
            slasherCount,
            lastFeeChangeSlot,
            lastFullStateUpdateSlot,
            depositFeeBps,
            withdrawalFeeBps,
            nextWithdrawalFeeBps,
            rewardFeeBps,
            programFeeBps,
            bump,
            isPaused,
            lastStartStateUpdateSlot,
        }
    }

    public async getJitoVaultOperatorDelegation(
        symbol: keyof typeof this.restakingVaultMetadata,
        operator: web3.PublicKey | string
    ) {
        const restakingVault = this.restakingVaultMetadata[symbol];
        const restakingOperator = operator instanceof web3.PublicKey ? operator : new web3.PublicKey(operator);
        const [vaultOperatorDelegation] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("vault_operator_delegation"), restakingVault.vault.toBuffer(), restakingOperator.toBuffer()],
            this.knownAddress.jitoVaultProgram,
        );
        logger.debug(`vault_operator_delegation`.padEnd(LOG_PAD_LARGE), vaultOperatorDelegation);

        let data = await this.connection.getAccountInfo(vaultOperatorDelegation).then(a => a.data);
        let expectedLength = data.length;

        const checkLengthDecreased = (decreased: number, msg: string) => {
            expectedLength -= decreased;
            if (data.length != expectedLength) {
                throw msg;
            }
        };
        const parsePubkey = (msg?: string) => {
            const key = new web3.PublicKey(Uint8Array.from(data.subarray(0, 32)));
            data = data.subarray(32);
            checkLengthDecreased(32, msg ?? "");
            return key;
        };
        const parseValue = (msg?: string) => {
            const value = data.readBigUInt64LE();
            data = data.subarray(8);
            checkLengthDecreased(8, msg ?? "");
            return value;
        };
        const parseByte = (msg?: string) => {
            const byte = data[0];
            data = data.subarray(1);
            checkLengthDecreased(1, msg ?? "");
            return byte;
        };

        parseValue();
        const vault = parsePubkey();
        const operatorKey = parsePubkey();
        const stakedAmount = parseValue();
        const enqueuedForCooldownAmount = parseValue();
        const coolingDownAmount = parseValue();
        const delegationState = {stakedAmount, enqueuedForCooldownAmount, coolingDownAmount};
        data = data.subarray(256);
        checkLengthDecreased(256, "reserve");
        const lastUpdateSlot = parseValue();
        const index = parseValue();
        const bump = parseByte();
        data = data.subarray(263);
        checkLengthDecreased(263, "done");

        return {
            vault,
            operator: operatorKey,
            delegationState,
            lastUpdateSlot,
            index,
            bump,
        };
    }

    private fragJTOImageURI = "https://fragmetric-assets.s3.ap-northeast-2.amazonaws.com/fragjto.png";
    private fragJTOMetadataURI = "https://quicknode.quicknode-ipfs.com/ipfs/QmQyCKdba9f6dpxc43pGwQ66DvjpPFbE6S8rPrKDh1Sz72";
    private fragJTOMetadata: splTokenMetadata.TokenMetadata = {
        mint: this.keychain.getPublicKey("FRAGJTO_MINT"),
        name: "Fragmetric Staked JTO",
        symbol: "fragJTO",
        uri: this.fragJTOMetadataURI,
        additionalMetadata: [["description", `fragJTO is the staked Jito governance token that provides optimized restaking rewards.`]],
        updateAuthority: this.keychain.getPublicKey("ADMIN"),
    };

    public async runAdminInitializeFragJTOTokenMint() {
        const fileForMetadataURI = JSON.stringify(
            {
                name: this.fragJTOMetadata.name,
                symbol: this.fragJTOMetadata.symbol,
                description: this.fragJTOMetadata.additionalMetadata[0][1],
                image: this.fragJTOImageURI,
                // attributes: [],
            },
            null,
            0
        );
        logger.debug(`fragJTO metadata file:\n> ${this.fragJTOMetadata.uri}\n> ${fileForMetadataURI}`);

        const mintInitialSize = spl.getMintLen([spl.ExtensionType.TransferHook, spl.ExtensionType.MetadataPointer]);
        const mintMetadataExtensionSize = spl.TYPE_SIZE + spl.LENGTH_SIZE + splTokenMetadata.pack(this.fragJTOMetadata).length;
        const mintTotalSize = mintInitialSize + mintMetadataExtensionSize;
        const lamports = await this.connection.getMinimumBalanceForRentExemption(mintTotalSize);

        await this.run({
            instructions: [
                web3.SystemProgram.createAccount({
                    fromPubkey: this.wallet.publicKey,
                    newAccountPubkey: this.knownAddress.fragJTOTokenMint,
                    lamports: lamports,
                    space: mintInitialSize,
                    programId: spl.TOKEN_2022_PROGRAM_ID,
                }),
                spl.createInitializeTransferHookInstruction(this.knownAddress.fragJTOTokenMint, this.keychain.getPublicKey("ADMIN"), this.programId, spl.TOKEN_2022_PROGRAM_ID),
                spl.createInitializeMetadataPointerInstruction(this.knownAddress.fragJTOTokenMint, this.keychain.getPublicKey("ADMIN"), this.knownAddress.fragJTOTokenMint, spl.TOKEN_2022_PROGRAM_ID),
                spl.createInitializeMintInstruction(
                    this.knownAddress.fragJTOTokenMint,
                    this.fragJTODecimals,
                    this.keychain.getPublicKey("ADMIN"),
                    null, // freeze authority to be null
                    spl.TOKEN_2022_PROGRAM_ID
                ),
                splTokenMetadata.createInitializeInstruction({
                    programId: spl.TOKEN_2022_PROGRAM_ID,
                    mint: this.knownAddress.fragJTOTokenMint,
                    metadata: this.knownAddress.fragJTOTokenMint,
                    name: this.fragJTOMetadata.name,
                    symbol: this.fragJTOMetadata.symbol,
                    uri: this.fragJTOMetadata.uri,
                    mintAuthority: this.keychain.getPublicKey("ADMIN"),
                    updateAuthority: this.fragJTOMetadata.updateAuthority,
                }),
                ...this.fragJTOMetadata.additionalMetadata.map(([field, value]) =>
                    splTokenMetadata.createUpdateFieldInstruction({
                        programId: spl.TOKEN_2022_PROGRAM_ID,
                        metadata: this.knownAddress.fragJTOTokenMint,
                        updateAuthority: this.fragJTOMetadata.updateAuthority,
                        field,
                        value,
                    })
                ),
            ],
            signerNames: ["ADMIN", "FRAGJTO_MINT"],
        });
        const fragJTOMint = await spl.getMint(
            // @ts-ignore
            this.connection,
            this.knownAddress.fragJTOTokenMint,
            "confirmed",
            spl.TOKEN_2022_PROGRAM_ID
        );
        logger.notice("fragJTO token mint created with extensions".padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOTokenMint.toString());
        return {fragJTOMint};
    }

    public async runAdminInitializeWfragJTOTokenMint(createMetadata = false) {
        const mintSize = spl.getMintLen([]);
        const lamports = await this.connection.getMinimumBalanceForRentExemption(mintSize);

        await this.run({
            instructions: [
                web3.SystemProgram.createAccount({
                    fromPubkey: this.wallet.publicKey,
                    newAccountPubkey: this.knownAddress.wfragJTOTokenMint,
                    lamports: lamports,
                    space: mintSize,
                    programId: spl.TOKEN_PROGRAM_ID,
                }),
                spl.createInitializeMintInstruction(
                    this.knownAddress.wfragJTOTokenMint,
                    this.wfragJTODecimals,
                    this.keychain.getPublicKey("ADMIN"),
                    null, // freeze authority to be null
                    spl.TOKEN_PROGRAM_ID
                ),
            ],
            signerNames: ["FRAGJTO_WRAPPED_TOKEN_MINT"],
        });

        if (this.isLocalnet) {
            const txSig = await this.connection.requestAirdrop(this.keychain.getKeypair("ADMIN").publicKey, 1_000_000_000)
            await this.connection.confirmTransaction(txSig, 'confirmed');
        }

        if (createMetadata) {
            const umiInstance = umi2.createUmi(this.connection.rpcEndpoint).use(mpl.mplTokenMetadata());
            const keypair = this.keychain.getKeypair('FRAGJTO_WRAPPED_TOKEN_MINT');
            const umiKeypair = umiInstance.eddsa.createKeypairFromSecretKey(keypair.secretKey);
            const mint = umi.createSignerFromKeypair(umiInstance, umiKeypair);

            const authKeypair = umiInstance.eddsa.createKeypairFromSecretKey(this.keychain.getKeypair("ADMIN").secretKey);
            const authority = umi.createSignerFromKeypair(umiInstance, authKeypair);
            umiInstance.use(umi.signerIdentity(authority));

            await mpl.createV1(umiInstance, {
                mint,
                authority,
                name: "Wrapped Fragmetric Staked JTO",
                symbol: "wfragJTO",
                decimals: this.wfragJTODecimals,
                uri: "https://quicknode.quicknode-ipfs.com/ipfs/QmS4bSyX4v9tCWMnchJ7jhpWHy1YYKsD5nsMDAs45yX5hZ",
                sellerFeeBasisPoints: umi.percentAmount(0),
                tokenStandard: mpl.TokenStandard.Fungible,
            }).sendAndConfirm(umiInstance);

            const assets = await mpl.fetchAllDigitalAssetByUpdateAuthority(umiInstance, authority.publicKey);
            logger.notice("wfragJTO token mint metadata created".padEnd(LOG_PAD_LARGE), assets);
        }

        const wfragJTOMint = await spl.getMint(this.connection, this.knownAddress.wfragJTOTokenMint, "confirmed", spl.TOKEN_PROGRAM_ID);
        logger.notice("wfragJTO token mint created".padEnd(LOG_PAD_LARGE), this.knownAddress.wfragJTOTokenMint.toString());
        return {wfragJTOMint};
    }

    public async runAdminUpdateTokenMetadata() {
        const fragJTOTokenMetadataAddress = this.knownAddress.fragJTOTokenMint;

        let tokenMetadata = await spl.getTokenMetadata(this.connection, fragJTOTokenMetadataAddress, undefined, spl.TOKEN_2022_PROGRAM_ID);
        logger.debug(`current token metadata:\n> ${JSON.stringify(tokenMetadata, null, 0)}`);

        const updatedFileForMetadataURI = JSON.stringify(
            {
                name: tokenMetadata.name,
                symbol: tokenMetadata.symbol,
                description: tokenMetadata.additionalMetadata[0][1],
                image: this.fragJTOImageURI,
                // attributes: [],
            },
            null,
            0
        );
        logger.debug(`fragJTO metadata file:\n> ${updatedFileForMetadataURI}`);

        const updatedMetadata = spl.updateTokenMetadata(tokenMetadata, splTokenMetadata.Field.Uri, this.fragJTOMetadataURI);
        logger.debug(`will update token metadata:\n> ${JSON.stringify(updatedMetadata, null, 0)}`);

        await this.run({
            instructions: [
                splTokenMetadata.createUpdateFieldInstruction({
                    programId: spl.TOKEN_2022_PROGRAM_ID,
                    metadata: this.knownAddress.fragJTOTokenMint,
                    updateAuthority: tokenMetadata.updateAuthority,
                    field: splTokenMetadata.Field.Uri,
                    value: this.fragJTOMetadataURI,
                }),
            ],
            signerNames: ["ADMIN"],
        });

        tokenMetadata = await spl.getTokenMetadata(this.connection, fragJTOTokenMetadataAddress, "confirmed", spl.TOKEN_2022_PROGRAM_ID);
        logger.notice(`updated token metadata:\n> ${JSON.stringify(tokenMetadata, null, 2)}`);
    }

    public async runAdminInitializeOrUpdateRewardAccount(batchSize = 35) {
        const currentVersion = await this.connection
            .getAccountInfo(this.knownAddress.fragJTOReward)
            .then((a) => a.data.readInt16LE(8))
            .catch((err) => 0);

        const targetVersion = parseInt(this.getConstant("rewardAccountCurrentVersion"));
        const instructions = [
            ...(currentVersion == 0 ? [this.program.methods.adminInitializeRewardAccount().accounts({
                payer: this.wallet.publicKey,
                receiptTokenMint: this.knownAddress.fragJTOTokenMint,
            }).instruction()] : []),
            ...new Array(targetVersion - currentVersion).fill(null).map((_, index, arr) => this.program.methods.adminUpdateRewardAccountIfNeeded(null).accounts({
                payer: this.wallet.publicKey,
                receiptTokenMint: this.knownAddress.fragJTOTokenMint,
            }).instruction()),
        ];
        if (instructions.length > 0) {
            for (let i = 0; i < instructions.length / batchSize; i++) {
                const batchedInstructions = [];
                for (let j = i * batchSize; j < instructions.length && batchedInstructions.length < batchSize; j++) {
                    batchedInstructions.push(instructions[j]);
                }
                logger.debug(`running batched instructions`.padEnd(LOG_PAD_LARGE), `${i * batchSize + batchedInstructions.length}/${instructions.length}`);
                await this.run({
                    instructions: batchedInstructions,
                    signerNames: ["ADMIN"],
                });
            }
        }

        const fragJTORewardAccount = await this.account.rewardAccount.fetch(this.knownAddress.fragJTOReward, "confirmed");
        logger.notice(`updated reward account version from=${currentVersion}, to=${fragJTORewardAccount.dataVersion}, target=${targetVersion}`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOReward.toString());

        return {fragJTORewardAccount};
    }

    public async runAdminInitializeOrUpdateFundAccount(batchSize = 35) {
        const knownAddressLookupTableAddress = await this.getOrCreateKnownAddressLookupTable();

        const currentVersion = await this.connection
            .getAccountInfo(this.knownAddress.fragJTOFund)
            .then((a) => a.data.readInt16LE(8))
            .catch((err) => 0);

        const targetVersion = parseInt(this.getConstant("fundAccountCurrentVersion"));
        const instructions = [
            spl.createAssociatedTokenAccountIdempotentInstruction(
                this.wallet.publicKey,
                this.knownAddress.fragJTOFundReceiptTokenLockAccount,
                this.knownAddress.fragJTOFund,
                this.knownAddress.fragJTOTokenMint,
                spl.TOKEN_2022_PROGRAM_ID,
            ),
            ...(currentVersion == 0 ? [this.program.methods.adminInitializeFundAccount().accounts({
                payer: this.wallet.publicKey,
                receiptTokenMint: this.knownAddress.fragJTOTokenMint,
            }).instruction()] : []),
            ...new Array(targetVersion - currentVersion).fill(null).map((_, index, arr) => this.program.methods.adminUpdateFundAccountIfNeeded(null).accounts({
                payer: this.wallet.publicKey,
                receiptTokenMint: this.knownAddress.fragJTOTokenMint,
            }).instruction()),
            this.methods.adminSetAddressLookupTableAccount(knownAddressLookupTableAddress).accountsPartial({
                payer: this.wallet.publicKey,
                receiptTokenMint: this.knownAddress.fragJTOTokenMint,
            }).instruction(),
        ];
        if (instructions.length > 0) {
            for (let i = 0; i < instructions.length / batchSize; i++) {
                const batchedInstructions = [];
                for (let j = i * batchSize; j < instructions.length && batchedInstructions.length < batchSize; j++) {
                    batchedInstructions.push(instructions[j]);
                }
                logger.debug(`running batched instructions`.padEnd(LOG_PAD_LARGE), `${i * batchSize + batchedInstructions.length}/${instructions.length}`);
                await this.run({
                    instructions: batchedInstructions,
                    signerNames: ["ADMIN"],
                });
            }
        }

        const [fragJTOMint, fragJTOFundAccount] = await Promise.all([
            spl.getMint(this.connection, this.knownAddress.fragJTOTokenMint, "confirmed", spl.TOKEN_2022_PROGRAM_ID),
            this.account.fundAccount.fetch(this.knownAddress.fragJTOFund, "confirmed"),
        ]);
        logger.notice(`updated fund account version from=${currentVersion}, to=${fragJTOFundAccount.dataVersion}, target=${targetVersion}`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOFund.toString());

        return {fragJTOMint: fragJTOMint, fragJTOFundAccount: fragJTOFundAccount};
    }

    public async runAdminInitializeOrUpdateFundWrapAccountRewardAccount() {
        const fragJTOFundWrapAccountAddress = this.knownAddress.fragJTOFundWrapAccount;
        const currentRewardVersion = await this.getFragJTOFundWrapAccountRewardAccount()
            .then(a => a.dataVersion)
            .catch(err => 0);
        
        const targetRewardVersion = parseInt(this.getConstant("userRewardAccountCurrentVersion"));

        await this.run({
            instructions: [
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.fragJTOFundReceiptTokenWrapAccount,
                    fragJTOFundWrapAccountAddress,
                    this.knownAddress.fragJTOTokenMint,
                    spl.TOKEN_2022_PROGRAM_ID,
                ),
                ...(currentRewardVersion == 0 ? [
                    this.program.methods.adminInitializeFundWrapAccountRewardAccount()
                        .accountsPartial({
                            payer: this.wallet.publicKey,
                            receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        })
                        .instruction(),
                    ]
                    : [
                        ...new Array(targetRewardVersion - currentRewardVersion).fill(null).map((_, index, arr) =>
                            this.program.methods
                                .adminUpdateFundWrapAccountRewardAccountIfNeeded(null)
                                .accountsPartial({
                                    payer: this.wallet.publicKey,
                                    receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                                })
                                .instruction(),
                        )
                    ]),
            ],
            signerNames: ['ADMIN'],
        });

        const fragJTOFundWrapAccountRewardAccount = await this.getFragJTOFundWrapAccountRewardAccount();
        logger.notice(`created fund wrap account reward account`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOFundWrapAccountReward.toString());

        return {fragJTOFundWrapAccountRewardAccount};
    }

    public async runFundManagerInitializeFundWrappedToken() {
        await this.run({
            instructions: [
                this.program.methods.fundManagerInitializeFundWrappedToken()
                    .accountsPartial({
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        wrappedTokenMint: this.knownAddress.wfragJTOTokenMint,
                    })
                    .instruction(),
            ],
            signerNames: ['ADMIN', 'FUND_MANAGER'],
            events: ['fundManagerUpdatedFund']
        });

        const [wfragJTOMint, fragJTOFundAccount] = await Promise.all([
            spl.getMint(this.connection, this.knownAddress.wfragJTOTokenMint, "confirmed", spl.TOKEN_PROGRAM_ID),
            this.getFragJTOFundAccount(),
        ]);
        logger.notice('set fragJTO fund wrapped token'.padEnd(LOG_PAD_LARGE), this.knownAddress.wfragJTOTokenMint.toString());

        return {wfragJTOMint, fragJTOFundAccount};
    }

    public async runFundManagerInitializeFundJitoRestakingVaults() {
        const {event, error} = await this.run({
            instructions: Object.entries(this.restakingVaultMetadata).flatMap(([symbol, v]) => {
                return [
                    // TODO v0.3/restaking: adjust authority of fee wallet
                    spl.createAssociatedTokenAccountIdempotentInstruction(
                        this.wallet.publicKey,
                        v.feeWalletTokenAccount,
                        this.knownAddress.jitoVaultFeeWallet,
                        v.VRTMint,
                    ),
                    spl.createAssociatedTokenAccountIdempotentInstruction(
                        this.wallet.publicKey,
                        v.vaultTokenAccount,
                        v.vault,
                        v.VSTMint,
                    ),
                    spl.createAssociatedTokenAccountIdempotentInstruction(
                        this.wallet.publicKey,
                        v.fundVRTAccount,
                        this.knownAddress.fragJTOFundReserveAccount,
                        v.VRTMint,
                    ),
                    spl.createAssociatedTokenAccountIdempotentInstruction(
                        this.wallet.publicKey,
                        v.programFeeWalletTokenAccount,
                        this.knownAddress.jitoVaultProgramFeeWallet,
                        v.VRTMint,
                    ),
                    this.program.methods.fundManagerInitializeFundJitoRestakingVault()
                        .accountsPartial({
                            receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                            vaultAccount: v.vault,
                            vaultReceiptTokenMint: v.VRTMint,
                            vaultSupportedTokenMint: v.VSTMint,
                        })
                        .remainingAccounts(this.pricingSourceAccounts)
                        .instruction(),
                ]
            }),
            signerNames: ["FUND_MANAGER"],
            events: ["fundManagerUpdatedFund"],
        });

        logger.notice(`initialized fragJTO fund jito restaking vaults`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOFund.toString());
        const fragJTOFund = await this.account.fundAccount.fetch(this.knownAddress.fragJTOFund, 'confirmed');
        return {event, error, fragJTOFund};
    }

    public async runFundManagerAddJitoRestakingVault(symbol: keyof typeof this.restakingVaultMetadata) {
        const vault = this.restakingVaultMetadata[symbol];
        await this.run({
            instructions: [
                // TODO v0.3/restaking: adjust authority of fee wallet
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    vault.feeWalletTokenAccount,
                    this.knownAddress.jitoVaultFeeWallet,
                    vault.VRTMint,
                ),
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    vault.vaultTokenAccount,
                    vault.vault,
                    vault.VSTMint,
                ),
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    vault.fundVRTAccount,
                    this.knownAddress.fragJTOFundReserveAccount,
                    vault.VRTMint,
                ),
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    vault.programFeeWalletTokenAccount,
                    this.knownAddress.jitoVaultProgramFeeWallet,
                    vault.VRTMint,
                ),
                this.program.methods.fundManagerInitializeFundJitoRestakingVault()
                    .accountsPartial({
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        vaultAccount: vault.vault,
                        vaultReceiptTokenMint: vault.VRTMint,
                        vaultSupportedTokenMint: vault.VSTMint,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signerNames: ["FUND_MANAGER"],
            events: ['fundManagerUpdatedFund'],
        });

        const [
            fragJTOFundJitoFeeWalletTokenAccount,
            fragJTOJitoVaultTokenAccount,
            fragJTOFundJitoVRTAccount,
            fragJTOJitoVaultProgramFeeWalletTokenAccount,
            fragJTOFundAccount,
        ] = await Promise.all([
            spl.getAccount(this.connection, vault.feeWalletTokenAccount, 'confirmed'),
            spl.getAccount(this.connection, vault.vaultTokenAccount, 'confirmed'),
            spl.getAccount(this.connection, vault.fundVRTAccount, 'confirmed'),
            spl.getAccount(this.connection, vault.programFeeWalletTokenAccount, 'confirmed'),
            this.getFragJTOFundAccount(),
        ]);
        logger.notice("jito VRT fee account created".padEnd(LOG_PAD_LARGE), vault.feeWalletTokenAccount.toString());
        logger.notice("jito vault VST account created".padEnd(LOG_PAD_LARGE), vault.vaultTokenAccount.toString());
        logger.notice("jito VRT account created".padEnd(LOG_PAD_LARGE), vault.fundVRTAccount.toString());
        logger.notice("jito VRT account (of program fee wallet) created".padEnd(LOG_PAD_LARGE), vault.programFeeWalletTokenAccount.toString());

        return {fragJTOFundJitoVRTAccount, fragJTOJitoVaultTokenAccount, fragJTOFundJitoFeeWalletTokenAccount, fragJTOJitoVaultProgramFeeWalletTokenAccount, fragJTOFundAccount};
    }

    public async runFundManagerAddJitoRestakingVaultDelegations() {
        const instructions = Object.values(this.restakingVaultMetadata).flatMap(vault =>
            Object.values(vault.operators).map(operator =>
                this.methods.fundManagerInitializeFundJitoRestakingVaultDelegation()
                    .accountsPartial({
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        vaultAccount: vault.vault,
                        vaultOperator: operator as web3.PublicKey,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            )
        );
        const {event, error} = (instructions.length > 0) ? await this.run({
            instructions,
            signerNames: ["FUND_MANAGER"],
            events: ["fundManagerUpdatedFund"],
        }) : {event: null, error: null};

        logger.notice(`initialized fragJTO jito vaults operator delegations`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOFund.toString());
        const fragJTOFund = await this.getFragJTOFundAccount();
        return {event, error, fragJTOFund};
    }

    public async runFundManagerAddJitoRestakingVaultDelegation(vault: web3.PublicKey, operator: web3.PublicKey) {
        const {event, error} = await this.run({
            instructions: [
                this.methods.fundManagerInitializeFundJitoRestakingVaultDelegation()
                    .accountsPartial({
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        vaultAccount: vault,
                        vaultOperator: operator,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signerNames: ["FUND_MANAGER"],
            events: ['fundManagerUpdatedFund'],
        });

        logger.notice("initialized fragJTO jito vault operator delegation".padEnd(LOG_PAD_LARGE), operator.toString());
        const fragJTOFund = await this.getFragJTOFundAccount();
        return {event, error, fragJTOFund};
    }

    // below functions are auxillary functions for jito instructions.
    private getPublicKey(authority: web3.Keypair | web3.PublicKey | KEYCHAIN_KEYS) {
        if (authority instanceof web3.Keypair)  return authority.publicKey;
        if (authority instanceof web3.PublicKey) return authority;
        return this.keychain.getPublicKey(authority);
    }
    private getSigners(...keypairOrNames: (web3.Keypair | KEYCHAIN_KEYS)[]) {
        const signers = keypairOrNames.filter(keypairOrName => keypairOrName instanceof web3.Keypair) as web3.Signer[];
        const signerNames = keypairOrNames.filter(keypairOrName => !(keypairOrName instanceof web3.Keypair)) as KEYCHAIN_KEYS[];
        return { signers, signerNames };
    }
    private getAccountMeta(
        account: web3.Keypair | web3.PublicKey | KEYCHAIN_KEYS,
        ...signerOrWritable: ("SIGNER" | "WRITABLE")[]
    ) {
        const pubkey = this.getPublicKey(account);
        const isSigner = signerOrWritable.includes("SIGNER");
        const isWritable = signerOrWritable.includes("WRITABLE");
        return {pubkey, isSigner, isWritable};
    }

    // need for operation & test environment setup - create jito vault
    public async runAdminCreateJitoRestakingVault(
        vstMint: web3.PublicKey,
        authority: web3.Keypair | KEYCHAIN_KEYS = "ADMIN",
        vrtMint = web3.Keypair.generate(),
        base = web3.Keypair.generate(),
        depositFeeBps = 0,
        withdrawalFeeBps = 0,
        rewardFeeBps = 0,
        vstDecimals = 9,
        initializeTokenAmount = BigInt(0),
    ) {
        const [vaultPublicKey] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("vault"), base.publicKey.toBuffer()],
            this.knownAddress.jitoVaultProgram
        );
        const [burnVaultPublicKey] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("burn_vault"), base.publicKey.toBuffer()],
            this.knownAddress.jitoVaultProgram,
        );
        const authorityVstTokenAccount = spl.getAssociatedTokenAddressSync(vstMint, this.getPublicKey(authority), true);
        const vaultVstTokenAccount = spl.getAssociatedTokenAddressSync(vstMint, vaultPublicKey, true);
        const burnVaultVrtTokenAccount = spl.getAssociatedTokenAddressSync(vrtMint.publicKey, burnVaultPublicKey, true);
        logger.debug("vst mint".padEnd(LOG_PAD_LARGE), vstMint.toString());
        logger.debug("vrt mint".padEnd(LOG_PAD_LARGE), vrtMint.publicKey.toString());

        const InitializeVaultInstructionDataSize = {
            discriminator: 1, // u8
            depositFeeBps: 2, // u16
            withdrawalFeeBps: 2, // u16
            rewardFeeBps: 2, /// u16
            decimals: 1, // u8
            initializeTokenAmount: 8, // u64
        };

        const discriminator = 1;
        const data = Buffer.alloc(Object.values(InitializeVaultInstructionDataSize).reduce((x, y) => x + y));

        let offset = 0;
        data.writeUInt8(discriminator, offset); offset += InitializeVaultInstructionDataSize.discriminator;
        data.writeUInt16LE(depositFeeBps, offset); offset += InitializeVaultInstructionDataSize.depositFeeBps;
        data.writeUInt16LE(withdrawalFeeBps, offset); offset += InitializeVaultInstructionDataSize.withdrawalFeeBps;
        data.writeUint16LE(rewardFeeBps, offset); offset += InitializeVaultInstructionDataSize.rewardFeeBps;
        data.writeUInt8(vstDecimals, offset); offset += InitializeVaultInstructionDataSize.decimals;
        data.writeBigUInt64LE(initializeTokenAmount, offset); offset += InitializeVaultInstructionDataSize.initializeTokenAmount;

        const ix = new web3.TransactionInstruction({
            programId: this.knownAddress.jitoVaultProgram,
            keys: [
                this.getAccountMeta(this.knownAddress.jitoVaultConfig, "WRITABLE"),
                this.getAccountMeta(vaultPublicKey, "WRITABLE"),
                this.getAccountMeta(vrtMint, "SIGNER", "WRITABLE"),
                this.getAccountMeta(vstMint),
                this.getAccountMeta(authorityVstTokenAccount, "WRITABLE"),
                this.getAccountMeta(vaultVstTokenAccount, "WRITABLE"),
                this.getAccountMeta(burnVaultPublicKey),
                this.getAccountMeta(burnVaultVrtTokenAccount, "WRITABLE"),
                this.getAccountMeta(authority, "SIGNER", "WRITABLE"),
                this.getAccountMeta(base, "SIGNER"),
                this.getAccountMeta(web3.SystemProgram.programId),
                this.getAccountMeta(spl.TOKEN_PROGRAM_ID),
                this.getAccountMeta(spl.ASSOCIATED_TOKEN_PROGRAM_ID),
            ],
            data,
        });

        await this.run({
            instructions: [ix],
            ...this.getSigners(vrtMint, authority, base),
        });

        logger.notice("created jito restaking vault".padEnd(LOG_PAD_LARGE), vaultPublicKey.toString);

        return { vault: vaultPublicKey };
    }

    // need for test environment setup - create jito operator
    public async runAdminCreateJitoRestakingOperator(
        authority: web3.Keypair | KEYCHAIN_KEYS = "ADMIN",
        base = web3.Keypair.generate(),
        operatorFeeBps = 0,
    ) {
        const [operatorPublicKey] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("operator"), base.publicKey.toBuffer()],
            this.knownAddress.jitoRestakingProgram,
        );

        const InitializeOperatorInstructionDataSize = {
            discriminator: 1, // u8
            operatorFeeBps: 2, // u16
        };

        const discriminator = 2;
        const data = Buffer.alloc(Object.values(InitializeOperatorInstructionDataSize).reduce((x, y) => x + y));

        let offset = 0;
        data.writeUInt8(discriminator, offset); offset += InitializeOperatorInstructionDataSize.discriminator;
        data.writeUInt16LE(operatorFeeBps, offset); offset += InitializeOperatorInstructionDataSize.operatorFeeBps;

        const ix = new web3.TransactionInstruction({
            programId: this.knownAddress.jitoRestakingProgram,
            keys: [
                this.getAccountMeta(this.knownAddress.jitoRestakingConfig, "WRITABLE"),
                this.getAccountMeta(operatorPublicKey, "WRITABLE"),
                this.getAccountMeta(authority, "SIGNER", "WRITABLE"),
                this.getAccountMeta(base, "SIGNER"),
                this.getAccountMeta(web3.SystemProgram.programId),
            ],
            data,
        });

        await this.run({
            instructions: [ix],
            ...this.getSigners(authority, base),
        });

        logger.notice("created jito restaking operator".padEnd(LOG_PAD_LARGE), operatorPublicKey.toString());

        return { operator: operatorPublicKey };
    }

    // no use
    public async runAdminCreateJitoRestakingNcn(
        authority: web3.Keypair | KEYCHAIN_KEYS = "ADMIN",
        base = web3.Keypair.generate(),
    ) {
        const [ncnPublicKey] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("ncn"), base.publicKey.toBuffer()],
            this.knownAddress.jitoRestakingProgram,
        );

        const InitializeNcnInstructionDataSize = {
            discriminator: 1, // u8
        };

        const discriminator = 1;
        const data = Buffer.alloc(InitializeNcnInstructionDataSize.discriminator);

        let offset = 0;
        data.writeUInt8(discriminator, offset);

        const ix = new web3.TransactionInstruction({
            programId: this.knownAddress.jitoRestakingProgram,
            keys: [
                this.getAccountMeta(this.knownAddress.jitoRestakingConfig, "WRITABLE"),
                this.getAccountMeta(ncnPublicKey, "WRITABLE"),
                this.getAccountMeta(authority, "SIGNER", "WRITABLE"),
                this.getAccountMeta(base, "SIGNER"),
                this.getAccountMeta(web3.SystemProgram.programId),
            ],
            data,
        });

        await this.run({
            instructions: [ix],
            ...this.getSigners(authority, base),
        });

        logger.notice("created jito restaking ncn".padEnd(LOG_PAD_LARGE), ncnPublicKey.toString());

        return { ncn: ncnPublicKey };
    }

    // need for test environment setup - initialize operator_vault_ticket
    public async runAdminInitializeJitoOperatorVaultTicket(
        operator: web3.PublicKey,
        vault: web3.PublicKey,
        authority: web3.Keypair | KEYCHAIN_KEYS = "ADMIN",
    ) {
        const [operatorVaultTicketPublicKey] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("operator_vault_ticket"), operator.toBuffer(), vault.toBuffer()],
            this.knownAddress.jitoRestakingProgram,
        );
        logger.debug(`operator`.padEnd(LOG_PAD_LARGE), operator.toString());
        logger.debug(`vault`.padEnd(LOG_PAD_LARGE), vault.toString());

        const InitializeOperatorVaultTicketInstructionDataSize = {
            discriminator: 1, // u8
        };

        const discriminator = 5;
        const data = Buffer.alloc(InitializeOperatorVaultTicketInstructionDataSize.discriminator);

        let offset = 0;
        data.writeUInt8(discriminator, offset);

        const ix = new web3.TransactionInstruction({
            programId: this.knownAddress.jitoRestakingProgram,
            keys: [
                this.getAccountMeta(this.knownAddress.jitoRestakingConfig),
                this.getAccountMeta(operator, "WRITABLE"),
                this.getAccountMeta(vault),
                this.getAccountMeta(operatorVaultTicketPublicKey, "WRITABLE"),
                this.getAccountMeta(authority, "SIGNER"),
                this.getAccountMeta(this.wallet, "SIGNER", "WRITABLE"),
                this.getAccountMeta(web3.SystemProgram.programId),
            ],
            data,
        });

        await this.run({
            instructions: [ix],
            ...this.getSigners(authority),
        });

        logger.notice(`initialized jito restaking operator_vault_ticket`.padEnd(LOG_PAD_LARGE), operatorVaultTicketPublicKey.toString());

        return { operatorVaultTicket: operatorVaultTicketPublicKey };
    }

    // need for operation & test environment setup - initialize vault_operator_delegation
    public async runAdminInitializeJitoVaultOperatorDelegation(
        vault: web3.PublicKey,
        operator: web3.PublicKey,
        authority: web3.Keypair | KEYCHAIN_KEYS = "ADMIN",
    ) {
        const [operatorVaultTicketPublicKey] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("operator_vault_ticket"), operator.toBuffer(), vault.toBuffer()],
            this.knownAddress.jitoRestakingProgram,
        );
        const [vaultOperatorDelegationPublicKey] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("vault_operator_delegation"), vault.toBuffer(), operator.toBuffer()],
            this.knownAddress.jitoVaultProgram,
        );
        logger.debug(`vault`.padEnd(LOG_PAD_LARGE), vault.toString());
        logger.debug(`operator`.padEnd(LOG_PAD_LARGE), operator.toString());

        const InitializeVaultOperatorDelegationInstructionDataSize = {
            discriminator: 1, // u8
        };

        const discriminator = 3;
        const data = Buffer.alloc(InitializeVaultOperatorDelegationInstructionDataSize.discriminator);

        let offset = 0;
        data.writeUInt8(discriminator, offset);

        const ix = new web3.TransactionInstruction({
            programId: this.knownAddress.jitoVaultProgram,
            keys: [
                this.getAccountMeta(this.knownAddress.jitoVaultConfig),
                this.getAccountMeta(vault, "WRITABLE"),
                this.getAccountMeta(operator, "WRITABLE"),
                this.getAccountMeta(operatorVaultTicketPublicKey),
                this.getAccountMeta(vaultOperatorDelegationPublicKey, "WRITABLE"),
                this.getAccountMeta(authority, "SIGNER"),
                this.getAccountMeta(this.wallet, "SIGNER", "WRITABLE"),
                this.getAccountMeta(web3.SystemProgram.programId),
            ],
            data,
        });

        await this.run({
            instructions: [ix],
            ...this.getSigners(authority),
        });

        logger.notice(`initialized jito restaking vault_operator_delegation`.padEnd(LOG_PAD_LARGE), vaultOperatorDelegationPublicKey.toString());

        return { vaultOperatorDelegation: vaultOperatorDelegationPublicKey };
    }

    // need for operation & test environment setup - set secondary admin to fund_account
    public async runAdminSetSecondaryAdminForJitoVault(
        vault: web3.PublicKey,
        secondaryAdminRole: "DELEGATION_ADMIN",
        newAuthority = this.knownAddress.fragJTOFund,
        authority: web3.Keypair | KEYCHAIN_KEYS = "ADMIN",
    ) {
        logger.debug(`new ${secondaryAdminRole.replace(/_/g, ' ').toLowerCase()}`.padEnd(LOG_PAD_LARGE), newAuthority.toString());

        const SetSecondaryAdminInstructionDataSize = {
            discriminator: 1, // u8
            vaultAdminRole: 1, // enum VaultAdminRole
        };

        const discriminator = 22;
        const vaultAdminRole = (() => {
            switch (secondaryAdminRole) {
                case "DELEGATION_ADMIN":
                    return 0; // enum 0
            }
        })();
        const data = Buffer.alloc(Object.values(SetSecondaryAdminInstructionDataSize).reduce((x, y) => x + y));

        let offset = 0;
        data.writeUInt8(discriminator, offset); offset += SetSecondaryAdminInstructionDataSize.discriminator;
        data.writeUInt8(vaultAdminRole, offset); offset += SetSecondaryAdminInstructionDataSize.vaultAdminRole;

        const ix = new web3.TransactionInstruction({
            programId: this.knownAddress.jitoVaultProgram,
            keys: [
                this.getAccountMeta(this.knownAddress.jitoVaultConfig),
                this.getAccountMeta(vault, "WRITABLE"),
                this.getAccountMeta(authority, "SIGNER"),
                this.getAccountMeta(newAuthority),
            ],
            data,
        });

        await this.run({
            instructions: [ix],
            ...this.getSigners(authority),
        });

        logger.notice(`changed jito restaking vault secondary admin`.padEnd(LOG_PAD_LARGE), vault.toString());
    }

    // need for operation & test - delegate vault_token_account to fund_account
    public async runAdminDelegateJitoVaultTokenAccount(
        vault: web3.PublicKey,
        tokenMint: web3.PublicKey,
        delegate = this.knownAddress.fragJTOFund,
        delegateAssetAdmin: web3.Keypair | KEYCHAIN_KEYS = "ADMIN",
    ) {
        const vaultTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
            this.connection,
            this.wallet,
            tokenMint,
            vault,
            true,
            "confirmed",
            {
                skipPreflight: false,
                commitment: "confirmed",
            },
            spl.TOKEN_PROGRAM_ID,
        );

        logger.debug(`delegate token mint`.padEnd(LOG_PAD_LARGE), tokenMint.toString());
        logger.debug(`delegate to`.padEnd(LOG_PAD_LARGE), delegate.toString());

        const data = Buffer.from([20]);
        const ix = new web3.TransactionInstruction({
            programId: this.knownAddress.jitoVaultProgram,
            keys: [
                this.getAccountMeta(this.knownAddress.jitoVaultConfig),
                this.getAccountMeta(vault),
                this.getAccountMeta(delegateAssetAdmin, "SIGNER"),
                this.getAccountMeta(tokenMint),
                this.getAccountMeta(vaultTokenAccount.address, "WRITABLE"),
                this.getAccountMeta(delegate),
                this.getAccountMeta(spl.TOKEN_PROGRAM_ID),
            ],
            data,
        });

        await this.run({
            instructions: [ix],
            ...this.getSigners(delegateAssetAdmin),
        });

        logger.notice(`delegated jito vault token account`.padEnd(LOG_PAD_LARGE), vault.toString());
    }

    // no use
    public async runAdminInitializeJitoNcnOperatorState(
        ncn: web3.PublicKey,
        operator: web3.PublicKey,
        authority: web3.Keypair | KEYCHAIN_KEYS = "ADMIN",
    ) {
        const [ncnOperatorStatePublicKey] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("ncn_operator_state"), ncn.toBuffer(), operator.toBuffer()],
            this.knownAddress.jitoRestakingProgram,
        );
        logger.debug(`ncn`.padEnd(LOG_PAD_LARGE), ncn.toString());
        logger.debug(`operator`.padEnd(LOG_PAD_LARGE), operator.toString());

        const InitializeNcnOperatorStateInstructionDataSize = {
            discriminator: 1, // u8
        };

        const discriminator = 6;
        const data = Buffer.alloc(InitializeNcnOperatorStateInstructionDataSize.discriminator);

        const offset = 0;
        data.writeUInt8(discriminator, offset);

        const ix = new web3.TransactionInstruction({
            programId: this.knownAddress.jitoRestakingProgram,
            keys: [
                this.getAccountMeta(this.knownAddress.jitoRestakingConfig),
                this.getAccountMeta(ncn, "WRITABLE"),
                this.getAccountMeta(operator, "WRITABLE"),
                this.getAccountMeta(ncnOperatorStatePublicKey, "WRITABLE"),
                this.getAccountMeta(authority, "SIGNER"),
                this.getAccountMeta(this.wallet, "SIGNER", "WRITABLE"),
                this.getAccountMeta(web3.SystemProgram.programId),
            ],
            data,
        });

        await this.run({
            instructions: [ix],
            ...this.getSigners(authority),
        });

        logger.notice(`initialized jito restaking ncn_operator_state`.padEnd(LOG_PAD_LARGE), ncnOperatorStatePublicKey.toString());

        return { ncnOperatorState: ncnOperatorStatePublicKey };
    }

    // no use
    public async runAdminInitializeJitoNcnVaultTicket(
        ncn: web3.PublicKey,
        vault: web3.PublicKey,
        authority: web3.Keypair | KEYCHAIN_KEYS = "ADMIN",
    ) {
        const [ncnVaultTicketPublicKey] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("ncn_vault_ticket"), ncn.toBuffer(), vault.toBuffer()],
            this.knownAddress.jitoRestakingProgram,
        );
        logger.debug(`ncn`.padEnd(LOG_PAD_LARGE), ncn.toString());
        logger.debug(`vault`.padEnd(LOG_PAD_LARGE), vault.toString());

        const InitializeNcnVaultTicketInstructionDataSize = {
            discriminator: 1, // u8
        };

        const discriminator = 4;
        const data = Buffer.alloc(InitializeNcnVaultTicketInstructionDataSize.discriminator);

        let offset = 0;
        data.writeUInt8(discriminator, offset);

        const ix = new web3.TransactionInstruction({
            programId: this.knownAddress.jitoRestakingProgram,
            keys: [
                this.getAccountMeta(this.knownAddress.jitoRestakingConfig),
                this.getAccountMeta(ncn, "WRITABLE"),
                this.getAccountMeta(vault),
                this.getAccountMeta(ncnVaultTicketPublicKey, "WRITABLE"),
                this.getAccountMeta(authority, "SIGNER"),
                this.getAccountMeta(this.wallet, "SIGNER", "WRITABLE"),
                this.getAccountMeta(web3.SystemProgram.programId),
            ],
            data,
        });

        await this.run({
            instructions: [ix],
            ...this.getSigners(authority),
        });

        logger.notice(`initialized jito restaking ncn_vault_ticket`.padEnd(LOG_PAD_LARGE), ncnVaultTicketPublicKey.toString());

        return { ncnVaultTicket: ncnVaultTicketPublicKey };
    }

    // need for operation - initialize vault_ncn_ticket
    public async runAdminInitializeJitoVaultNcnTicket(
        vault: web3.PublicKey,
        ncn: web3.PublicKey,
        authority: web3.Keypair | KEYCHAIN_KEYS = "ADMIN",
    ) {
        const [ncnVaultTicketPublicKey] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("ncn_vault_ticket"), ncn.toBuffer(), vault.toBuffer()],
            this.knownAddress.jitoRestakingProgram,
        );
        const [vaultNcnTicketPublicKey] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("vault_ncn_ticket"), vault.toBuffer(), ncn.toBuffer()],
            this.knownAddress.jitoVaultProgram,
        );
        logger.debug(`vault`.padEnd(LOG_PAD_LARGE), vault.toString());
        logger.debug(`ncn`.padEnd(LOG_PAD_LARGE), ncn.toString());

        const InitializeVaultNcnTicketInstructionDataSize = {
            discriminator: 1, // u8
        };

        const discriminator = 4;
        const data = Buffer.alloc(InitializeVaultNcnTicketInstructionDataSize.discriminator);

        let offset = 0;
        data.writeUInt8(discriminator, offset);

        const ix = new web3.TransactionInstruction({
            programId: this.knownAddress.jitoVaultProgram,
            keys: [
                this.getAccountMeta(this.knownAddress.jitoVaultConfig),
                this.getAccountMeta(vault, "WRITABLE"),
                this.getAccountMeta(ncn),
                this.getAccountMeta(ncnVaultTicketPublicKey),
                this.getAccountMeta(vaultNcnTicketPublicKey, "WRITABLE"),
                this.getAccountMeta(authority, "SIGNER"),
                this.getAccountMeta(this.wallet, "SIGNER", "WRITABLE"),
                this.getAccountMeta(web3.SystemProgram.programId),
            ],
            data,
        });

        await this.run({
            instructions: [ix],
            ...this.getSigners(authority),
        });

        logger.notice(`initialized jito restaking vault_ncn_ticket`.padEnd(LOG_PAD_LARGE), vaultNcnTicketPublicKey.toString());

        return { vaultNcnTicket: vaultNcnTicketPublicKey };
    }

    public async runAdminInitializeFragJTOExtraAccountMetaList() {
        await this.run({
            instructions: [
                this.program.methods.adminInitializeExtraAccountMetaList().accounts({
                    payer: this.wallet.publicKey,
                    receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                }).instruction(),
            ],
            signerNames: ["ADMIN"],
        });
        const fragJTOExtraAccountMetasAccount = await this.connection.getAccountInfo(spl.getExtraAccountMetaAddress(this.knownAddress.fragJTOTokenMint, this.programId)).then((acc) => spl.getExtraAccountMetas(acc));
        logger.notice(`initialized fragJTO extra account meta list`.padEnd(LOG_PAD_LARGE));

        return {fragJTOExtraAccountMetasAccount};
    }

    public async runAdminUpdateFragJTOExtraAccountMetaList() {
        await this.run({
            instructions: [
                this.program.methods.adminUpdateExtraAccountMetaListIfNeeded().accounts({
                    payer: this.wallet.publicKey,
                    receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                }).instruction(),
            ],
            signerNames: ["ADMIN"],
        });
        const fragJTOExtraAccountMetasAccount = await this.connection.getAccountInfo(spl.getExtraAccountMetaAddress(this.knownAddress.fragJTOTokenMint, this.programId)).then((acc) => spl.getExtraAccountMetas(acc));
        logger.notice(`updated fragJTO extra account meta list`.padEnd(LOG_PAD_LARGE));

        return {fragJTOExtraAccountMetasAccount};
    }

    public async runFundManagerAddRestakingVaultCompoundingRewardTokens() {
        const {event, error} = await this.run({
            instructions: Object.values(this.restakingVaultMetadata).flatMap(vault =>
                (vault.compoundingRewards ?? []).map(rewardTokenMint =>
                    this.methods.fundManagerAddRestakingVaultCompoundingRewardToken(
                            vault.vault,
                            rewardTokenMint
                        ).accountsPartial({
                            receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        }).instruction())),
            signerNames: ["FUND_MANAGER"],
            events: ["fundManagerUpdatedFund"],
        });

        logger.notice(`added fragJTO restaking vaults compounding reward tokens`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOFund.toString());
        const fragJTOFund = await this.account.fundAccount.fetch(this.knownAddress.fragJTOFund, 'confirmed');
        return {event, error, fragJTOFund};
    }

    public async runFundManagerAddRestakingVaultCompoundingRewardToken(vault: web3.PublicKey, rewardTokenMint: web3.PublicKey) {
        const {event, error} = await this.run({
            instructions: [
                this.methods.fundManagerAddRestakingVaultCompoundingRewardToken(vault, rewardTokenMint)
                    .accountsPartial({
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                    })
                    .instruction(),
            ],
            signerNames: ["FUND_MANAGER"],
            events: ["fundManagerUpdatedFund"],
        });

        logger.notice(`added compounding reward token`.padEnd(LOG_PAD_LARGE), rewardTokenMint);
        const fragJTOFund = await this.account.fundAccount.fetch(this.knownAddress.fragJTOFund, 'confirmed');
        return {event, error, fragJTOFund};
    }

    public async runFundManagerAddTokenSwapStrategies() {
        const {event, error} = await this.run({
            instructions: this.tokenSwapStrategies.map(strategy => {
                return this.methods.fundManagerAddTokenSwapStrategy(
                    strategy.fromTokenMint,
                    strategy.toTokenMint,
                    strategy.tokenSwapSource,
                )
                .accountsPartial({
                    receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                })
                .instruction();
            }),
            signerNames: ["FUND_MANAGER"],
            events: ["fundManagerUpdatedFund"],
        });

        logger.notice(`added fragJTO token swap strategies`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOFund.toString());
        const fragJTOFund = await this.account.fundAccount.fetch(this.knownAddress.fragJTOFund, 'confirmed');
        return {event, error, fragJTOFund};
    }

    public async runFundManagerAddTokenSwapStrategy(
        fromTokenMint: web3.PublicKey,
    ) {
        const strategy = this.tokenSwapStrategies.find(strategy => strategy.fromTokenMint.equals(fromTokenMint));
        const {event, error} = await this.run({
            instructions: [
                this.methods.fundManagerAddTokenSwapStrategy(
                    strategy.fromTokenMint,
                    strategy.toTokenMint,
                    strategy.tokenSwapSource,
                )
                .accountsPartial({
                    receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                })
                .instruction(),
            ],
            signerNames: ["FUND_MANAGER"],
            events: ["fundManagerUpdatedFund"],
        });

        logger.notice(`added token swap strategy`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOFund.toString());
        const fragJTOFund = await this.account.fundAccount.fetch(this.knownAddress.fragJTOFund, 'confirmed');
        return {event, error, fragJTOFund};
    }

    public async runFundManagerInitializeFundSupportedTokens() {
        const {event, error} = await this.run({
            instructions: Object.entries(this.supportedTokenMetadata).flatMap(([symbol, v]) => {
                return [
                    spl.createAssociatedTokenAccountIdempotentInstruction(
                        this.wallet.publicKey,
                        this.knownAddress.fragJTOFundSupportedTokenReserveAccount(symbol as any),
                        this.knownAddress.fragJTOFundReserveAccount,
                        v.mint,
                        v.program,
                    ),
                    spl.createAssociatedTokenAccountIdempotentInstruction(
                        this.wallet.publicKey,
                        this.knownAddress.fragJTOFundSupportedTokenTreasuryAccount(symbol as any),
                        this.knownAddress.fragJTOFundTreasuryAccount,
                        v.mint,
                        v.program,
                    ),
                    this.program.methods
                        .fundManagerAddSupportedToken(v.pricingSource)
                        .accountsPartial({
                            receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                            supportedTokenMint: v.mint,
                            supportedTokenProgram: v.program,
                        })
                        .remainingAccounts(this.pricingSourceAccounts)
                        .instruction(),
                ];
            }),
            signerNames: ["FUND_MANAGER"],
            events: ["fundManagerUpdatedFund"],
        });

        logger.notice(`initialized fragJTO fund configuration`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOFund.toString());
        const fragJTOFund = await this.account.fundAccount.fetch(this.knownAddress.fragJTOFund, 'confirmed');
        return {event, error, fragJTOFund};
    }

    public get targetFragJTOFundConfiguration() {
        return {
            depositEnabled: this.isDevnet ? true : (this.isMainnet ? true : true),
            donationEnabled: true,
            withdrawalEnabled: this.isDevnet ? true : (this.isMainnet ? true : true),
            transferEnabled: this.isDevnet ? true : (this.isMainnet ? true : false),
            WithdrawalFeedRateBPS: this.isDevnet ? 10 : 10,
            withdrawalBatchThresholdSeconds: new BN(this.isDevnet ? 60 : (this.isMainnet ? 86400 : 10)), // seconds

            solDepositable: false,
            solAccumulatedDepositCapacity: new BN(0),
            solAccumulatedDepositAmount: null,
            solWithdrawalable: false,
            solWithdrawalNormalReserveRateBPS: 0,
            solWithdrawalNormalReserveMaxAmount: new BN(0),

            supportedTokens: Object.entries(this.supportedTokenMetadata).map(([symbol, v]) => ({
                tokenMint: v.mint,
                tokenDepositable: true,
                tokenAccumulatedDepositCapacity: (() => {
                    switch (symbol) {
                        case "JTO":
                            return new BN(MAX_CAPACITY);
                        default:
                            throw `invalid accumulated deposit cap for ${symbol}`;
                    }
                })(),
                tokenAccumulatedDepositAmount: null,
                withdrawable: this.isDevnet ? true : (this.isMainnet ? true : true),
                withdrawalNormalReserveRateBPS: this.isDevnet ? 5 : 5,
                withdrawalNormalReserveMaxAmount: new BN(MAX_CAPACITY),
                tokenRebalancingAmount: null,
                solAllocationWeight: (() => {
                    switch (symbol) {
                        case "JTO":
                            return new BN(1);
                        default:
                            throw `invalid sol allocation weight for ${symbol}`;
                    }
                })(),
                solAllocationCapacityAmount: (() => {
                    switch (symbol) {
                        case "JTO":
                            return new BN(this.isMainnet ? MAX_CAPACITY : MAX_CAPACITY);
                        default:
                            throw `invalid sol allocation cap for ${symbol}`;
                    }
                })(),
            })),
            restakingVaults: Object.entries(this.restakingVaultMetadata).map(([symbol, v]) => ({
                vault: v.vault,
                solAllocationWeight: (() => {
                    switch (symbol) {
                        case "jitoJTOVault":
                            return new BN(this.isMainnet ? 1 : 1);
                        default:
                            throw `invalid sol allocation weight for ${symbol}`;
                    }
                })(),
                solAllocationCapacityAmount: (() => {
                    switch (symbol) {
                        case "jitoJTOVault":
                            return new BN(this.isMainnet ? MAX_CAPACITY : MAX_CAPACITY);
                        default:
                            throw `invalid sol allocation cap for ${symbol}`;
                    }
                })(),
                delegations: Object.entries(v.operators).map(([name, operator]) => ({
                    operator,
                    supportedTokenAllocationWeight: (() => {
                        switch (symbol) {
                            case "jitoJTOVault":
                                if (this.isDevnet) {
                                    switch (name) {
                                        default:
                                            throw `invalid restaking vault operator supported token allocation weight for ${symbol}(${name})`;
                                    }
                                } else {
                                    switch (name) {
                                        case "InfStones":
                                            return new BN(this.isMainnet ? 0 : 1);
                                        case "Hashkey":
                                            return new BN(this.isMainnet ? 0 : 1);
                                        case "PierTwo":
                                            return new BN(this.isMainnet ? 0 : 1);
                                        case "Luganodes":
                                            return new BN(this.isMainnet ? 0 : 1);
                                        case "Everstake":
                                            return new BN(this.isMainnet ? 0 : 1);
                                        case "Temporal":
                                            return new BN(this.isMainnet ? 0 : 1);
                                        case "ChorusOne":
                                            return new BN(this.isMainnet ? 0 : 1);
                                        case "KILN":
                                            return new BN(this.isMainnet ? 0 : 1);
                                        case "Helius":
                                            return new BN(this.isMainnet ? 0 : 1);
                                        default:
                                            throw `invalid restaking vault operator supported token allocation weight for ${symbol}(${name})`;
                                    }
                                }
                                default:
                                    throw `invalid restaking vault operator supported token allocation weight for ${symbol}`;
                        }
                    })(),
                    supportedTokenAllocationCapacityAmount: (() => {
                        switch (symbol) {
                            case "jitoJTOVault":
                                if (this.isDevnet) {
                                    switch (name) {
                                        default:
                                            throw `invalid restaking vault operator supported token allocation weight for ${symbol}(${name})`;
                                    }
                                } else {
                                    switch (name) {
                                        case "InfStones":
                                            return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                                        case "Hashkey":
                                            return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                                        case "PierTwo":
                                            return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                                        case "Luganodes":
                                            return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                                        case "Everstake":
                                            return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                                        case "Temporal":
                                            return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                                        case "ChorusOne":
                                            return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                                        case "KILN":
                                            return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                                        case "Helius":
                                            return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                                        default:
                                            throw `invalid restaking vault operator supported token allocation weight for ${symbol}(${name})`;
                                    }
                                }
                            default:
                                throw `invalid restaking vault operator supported token allocation capacity amount for ${symbol}`;
                        }
                    })(),
                })),
            })),
        };
    }

    public async runFundManagerAddSupportedTokens(symbol: keyof typeof this.supportedTokenMetadata) {
        const token = this.supportedTokenMetadata[symbol];
        const {event, error} = await this.run({
            instructions: [
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.fragJTOFundSupportedTokenReserveAccount(symbol as any),
                    this.knownAddress.fragJTOFundReserveAccount,
                    token.mint,
                    token.program,
                ),
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.fragJTOFundSupportedTokenTreasuryAccount(symbol as any),
                    this.knownAddress.fragJTOFundTreasuryAccount,
                    token.mint,
                    token.program,
                ),
                this.methods
                    .fundManagerAddSupportedToken(token.pricingSource)
                    .accountsPartial({
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        supportedTokenMint: token.mint,
                        supportedTokenProgram: token.program,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signerNames: ["FUND_MANAGER"],
            events: ["fundManagerUpdatedFund"],
        });

        logger.notice(`added fragJTO fund supported token`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOFund.toString());
        const fragJTOFund = await this.account.fundAccount.fetch(this.knownAddress.fragJTOFund, 'confirmed');
        return {event, error, fragJTOFund};
    }

    // update capacity and configurations
    public async runFundManagerUpdateFundConfigurations() {
        const config = this.targetFragJTOFundConfiguration;
        const {event, error} = await this.run({
            instructions: [
                this.program.methods.fundManagerUpdateFundStrategy(
                    config.depositEnabled,
                    config.donationEnabled,
                    config.withdrawalEnabled,
                    config.transferEnabled,
                    config.WithdrawalFeedRateBPS, // 1 fee rate = 1bps = 0.01%
                    config.withdrawalBatchThresholdSeconds,
                ).accountsPartial({
                    receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                }).instruction(),
                ...config.supportedTokens.flatMap((v) => {
                    return [
                        this.program.methods.fundManagerUpdateSupportedTokenStrategy(
                            v.tokenMint,
                            v.tokenDepositable,
                            v.tokenAccumulatedDepositCapacity,
                            v.tokenAccumulatedDepositAmount,
                            v.withdrawable,
                            v.withdrawalNormalReserveRateBPS,
                            v.withdrawalNormalReserveMaxAmount,
                            v.tokenRebalancingAmount,
                            v.solAllocationWeight,
                            v.solAllocationCapacityAmount,
                        ).accountsPartial({
                            receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        })
                        .remainingAccounts(this.pricingSourceAccounts)
                        .instruction(),
                    ];
                }),
                this.program.methods.fundManagerUpdateSolStrategy(
                    config.solDepositable,
                    config.solAccumulatedDepositCapacity,
                    config.solAccumulatedDepositAmount,
                    config.solWithdrawalable,
                    config.solWithdrawalNormalReserveRateBPS,
                    config.solWithdrawalNormalReserveMaxAmount,
                ).accountsPartial({
                    receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                }).instruction(),
                ...config.restakingVaults.flatMap((v) => {
                    return [
                        this.program.methods.fundManagerUpdateRestakingVaultStrategy(
                            v.vault,
                            v.solAllocationWeight,
                            v.solAllocationCapacityAmount,
                        ).accountsPartial({
                            receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        })
                        .remainingAccounts(this.pricingSourceAccounts)
                        .instruction(),
                    ];
                }),
            ],
            signerNames: ["FUND_MANAGER"],
            events: ["fundManagerUpdatedFund"],
        });

        const updateDelegationStrategyInstructions = config.restakingVaults.flatMap(vault => vault.delegations.map(delegation => {
            return this.methods.fundManagerUpdateRestakingVaultDelegationStrategy(
                vault.vault as web3.PublicKey,
                delegation.operator as web3.PublicKey,
                delegation.supportedTokenAllocationWeight,
                delegation.supportedTokenAllocationCapacityAmount,
                null,
            ).accountsPartial({
                receiptTokenMint: this.knownAddress.fragJTOTokenMint,
            })
            .remainingAccounts(this.pricingSourceAccounts)
            .instruction();
        }));
        const numInstructions = updateDelegationStrategyInstructions.length;
        const batchSize = 5;
        if (numInstructions > 0) {
            for (let i = 0; i < numInstructions / batchSize; i++) {
                const batchedInstructions = [];
                for (let j = i * batchSize; j < numInstructions && batchedInstructions.length < batchSize; j++) {
                    batchedInstructions.push(updateDelegationStrategyInstructions[j]);
                }
                logger.debug(`running batched "update_restaking_vault_delegation_strategy" instructions`.padEnd(LOG_PAD_LARGE), `${i * batchSize + batchedInstructions.length}/${numInstructions}`);
                await this.run({
                    instructions: batchedInstructions,
                    signerNames: ["FUND_MANAGER"],
                    events: ["fundManagerUpdatedFund"],
                });
            }
        }

        logger.notice(`updated fragJTO fund configuration`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOFund.toString());
        const fragJTOFund = await this.account.fundAccount.fetch(this.knownAddress.fragJTOFund);
        return {event, error, fragJTOFund};
    }

    public get distributingRewardsMetadata() {
        return this._getDistributingRewardsMetadata;
    }

    private readonly _getDistributingRewardsMetadata = this.getDistributingRewardsMetadata();

    private getDistributingRewardsMetadata() {
        return [
            {
                name: "fPoint",
                description: "Airdrop point for fToken",
                type: {point: {decimals: 4}},
                tokenMint: null,
                tokenProgram: null,
            },
        ];
    }

    public get rewardPoolsMetadata() {
        return this._getRewardPoolsMetadata;
    }

    private readonly _getRewardPoolsMetadata = this.getRewardPoolsMetadata();

    private getRewardPoolsMetadata() {
        return [
            {
                name: "base",
                holderId: null,
                customAccrualRateEnabled: false,
            },
            {
                name: "bonus",
                holderId: null,
                customAccrualRateEnabled: true,
            },
        ];
    }

    public async runFundManagerInitializeRewardPools() {
        const {event, error} = await this.run({
            instructions: [
                ...this.rewardPoolsMetadata.map((v) => {
                    return this.program.methods
                        .fundManagerAddRewardPool(v.name, v.holderId, v.customAccrualRateEnabled)
                        .accountsPartial({
                            receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        })
                        .instruction();
                }),
                ...this.distributingRewardsMetadata.map((v) => {
                    return this.program.methods
                        .fundManagerAddReward(v.name, v.description, v.type)
                        .accountsPartial({
                            receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                            rewardTokenMint: v.tokenMint ?? this.programId,
                            rewardTokenProgram: v.tokenProgram ?? this.programId,
                        })
                        .instruction();
                }),
            ],
            signerNames: ["FUND_MANAGER"],
            events: ["fundManagerUpdatedRewardPool"],
        });

        logger.notice(`configured fragJTO reward pools and reward`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOReward.toString());
        const fragJTOReward = await this.account.rewardAccount.fetch(this.knownAddress.fragJTOReward);
        return {event, error, fragJTOReward};
    }

    public async runFundManagerSettleReward(args: {
        poolName: (typeof this.rewardPoolsMetadata)[number]["name"];
        rewardName: (typeof this.rewardsMetadata)[number]["name"];
        amount: BN
    }) {
        let fragJTOReward = await this.account.rewardAccount.fetch(this.knownAddress.fragJTOReward);
        let rewardPool = fragJTOReward.rewardPools1.find((r) => this.binToString(r.name) == args.poolName);
        let reward = fragJTOReward.rewards1.find((r) => this.binToString(r.name) == args.rewardName);

        const rewardTokenMint = this.binIsEmpty(reward.tokenMint.toBuffer()) ? this.programId : reward.tokenMint;
        const rewardTokenProgram = this.binIsEmpty(reward.tokenProgram.toBuffer()) ? this.programId : reward.tokenProgram;
        const {event, error} = await this.run({
            instructions: [
                this.program.methods
                    .fundManagerSettleReward(rewardPool.id, reward.id, args.amount)
                    .accountsPartial({
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        rewardTokenMint,
                        rewardTokenProgram,
                    })
                    .instruction(),
            ],
            signerNames: ["FUND_MANAGER"],
            events: ["fundManagerUpdatedRewardPool"],
        });

        logger.notice(`settled fragJTO reward to pool=${rewardPool.id}/${args.poolName}, rewardId=${reward.id}/${args.rewardName}, amount=${args.amount.toString()} (decimals=${reward.decimals})`);
        fragJTOReward = await this.account.rewardAccount.fetch(this.knownAddress.fragJTOReward);
        rewardPool = fragJTOReward.rewardPools1.find((r) => this.binToString(r.name) == args.poolName);
        reward = fragJTOReward.rewards1.find((r) => this.binToString(r.name) == args.rewardName);

        return {event, error, fragJTOReward, rewardPool, reward};
    }

    private async getInstructionsToUpdateUserFragJTOFundAndRewardAccounts(user: web3.Keypair) {
        // Assumes that both user fund & reward account size is small enough

        // const fragJTOUserRewardAddress = this.knownAddress.fragJTOUserReward(user.publicKey);
        // const fragJTOUserFundAddress = this.knownAddress.fragJTOUserFund(user.publicKey);
        // const currentRewardVersion = await this.account.userRewardAccount
        //     .fetch(fragJTOUserRewardAddress)
        //     .then((a) => a.dataVersion)
        //     .catch((err) => 0);
        // const currentFundVersion = await this.account.userFundAccount
        //     .fetch(fragJTOUserFundAddress)
        //     .then((a) => a.dataVersion)
        //     .catch((err) => 0);

        // const targetRewardVersion = parseInt(this.getConstant("userRewardAccountCurrentVersion"));
        return [
            spl.createAssociatedTokenAccountIdempotentInstruction(
                user.publicKey,
                this.knownAddress.fragJTOUserTokenAccount(user.publicKey),
                user.publicKey,
                this.knownAddress.fragJTOTokenMint,
                spl.TOKEN_2022_PROGRAM_ID,
            ),
            this.program.methods.userCreateFundAccountIdempotent(null)
                .accountsPartial({
                    user: user.publicKey,
                    receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                })
                .instruction(),
            this.program.methods.userCreateRewardAccountIdempotent(null)
                .accountsPartial({
                    user: user.publicKey,
                    receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                })
                .instruction(),
            // ...(currentFundVersion == 0
            //     ? [
            //         this.program.methods.userInitializeFundAccount()
            //             .accounts({
            //                 user: user.publicKey,
            //                 receiptTokenMint: this.knownAddress.fragJTOTokenMint,
            //             })
            //             .instruction(),
            //     ]
            //     : [
            //         this.program.methods.userUpdateFundAccountIfNeeded()
            //             .accountsPartial({
            //                 user: user.publicKey,
            //                 receiptTokenMint: this.knownAddress.fragJTOTokenMint,
            //             })
            //             .instruction(),
            //     ]),
            // ...(currentRewardVersion == 0 ? [
            //     this.program.methods.userInitializeRewardAccount()
            //         .accountsPartial({
            //             user: user.publicKey,
            //             receiptTokenMint: this.knownAddress.fragJTOTokenMint,
            //         })
            //         .instruction(),
            //     ]
            //     : [
            //         ...new Array(targetRewardVersion - currentRewardVersion).fill(null).map((_, index, arr) =>
            //             this.program.methods
            //                 .userUpdateRewardAccountIfNeeded(null)
            //                 .accountsPartial({
            //                     user: user.publicKey,
            //                     receiptTokenMint: this.knownAddress.fragJTOTokenMint,
            //                 })
            //                 .instruction(),
            //         ),
            //     ]),
        ];
    }

    public async runUserCreateOrUpdateFragJTOFundAndRewardAccount(user: web3.Keypair) {
        await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragJTOFundAndRewardAccounts(user)),
            ],
            signers: [user],
        });
    }

    public lamportsToFragJTO(lamports: BN): string {
        return super.lamportsToX(lamports, this.fragJTODecimals, "fragJTO");
    }

    public async runUserDepositSupportedToken(
        user: web3.Keypair,
        tokenSymbol: keyof typeof this.supportedTokenMetadata,
        amount: BN,
        depositMetadata: IdlTypes<Restaking>["depositMetadata"] | null = null,
        depositMetadataSigningKeypair: web3.Keypair | null = null
    ) {
        let depositMetadataInstruction: web3.TransactionInstruction[] = [];
        if (depositMetadata) {
            depositMetadataSigningKeypair = depositMetadataSigningKeypair ?? this.keychain.getKeypair("ADMIN");
            const message = this.program.coder.types.encode("depositMetadata", depositMetadata);
            const signature = ed25519.Sign(message, Buffer.from(depositMetadataSigningKeypair.secretKey));
            depositMetadataInstruction.push(
                web3.Ed25519Program.createInstructionWithPublicKey({
                    publicKey: depositMetadataSigningKeypair.publicKey.toBytes(),
                    message,
                    signature,
                })
            );
        }

        const supportedToken = this.supportedTokenMetadata[tokenSymbol];
        const userSupportedTokenAddress = this.knownAddress.userSupportedTokenAccount(user.publicKey, tokenSymbol);

        const {event, error} = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragJTOFundAndRewardAccounts(user)),
                ...depositMetadataInstruction,
                this.program.methods
                    .userDepositSupportedToken(amount, depositMetadata)
                    .accountsPartial({
                        user: user.publicKey,
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        supportedTokenMint: supportedToken.mint,
                        supportedTokenProgram: supportedToken.program,
                        userSupportedTokenAccount: userSupportedTokenAddress,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signers: [user],
            events: ["userDepositedToFund"],
        });

        const [fragJTOFund, fragJTOFundReserveAccountBalance, fragJTOReward, fragJTOUserFund, fragJTOUserReward, fragJTOUserTokenAccount, userSupportedTokenAccount] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragJTOFund),
            this.getFragJTOFundReserveAccountBalance(),
            this.account.rewardAccount.fetch(this.knownAddress.fragJTOReward),
            this.account.userFundAccount.fetch(this.knownAddress.fragJTOUserFund(user.publicKey)),
            this.account.userRewardAccount.fetch(this.knownAddress.fragJTOUserReward(user.publicKey)),
            this.getUserFragJTOAccount(user.publicKey),
            this.getUserSupportedTokenAccount(user.publicKey, tokenSymbol),
        ]);
        logger.notice(`deposited: ${this.lamportsToX(amount, supportedToken.decimals, tokenSymbol)} (${this.lamportsToX(fragJTOFund.oneReceiptTokenAsSol, fragJTOFund.receiptTokenDecimals, 'SOL/fragJTO')})`.padEnd(LOG_PAD_LARGE), userSupportedTokenAddress.toString());
        logger.info(`user fragJTO balance: ${this.lamportsToFragJTO(new BN(fragJTOUserTokenAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());

        return {
            event,
            error,
            fragJTOFund,
            fragJTOFundReserveAccountBalance,
            fragJTOReward,
            fragJTOUserFund,
            fragJTOUserReward,
            fragJTOUserTokenAccount,
            userSupportedTokenAccount
        };
    }

    public async runOperatorDonateSupportedTokenToFund(
        operator: web3.Keypair,
        tokenSymbol: keyof typeof this.supportedTokenMetadata,
        amount: BN,
        offsetReceivable: boolean = false,
    ) {
        const supportedToken = this.supportedTokenMetadata[tokenSymbol];
        const operatorSupportedTokenAddress = this.knownAddress.userSupportedTokenAccount(operator.publicKey, tokenSymbol);

        const {event, error} = await this.run({
            instructions: [
                web3.ComputeBudgetProgram.setComputeUnitLimit({
                    units: 300_000,
                }),
                this.program.methods
                    .operatorDonateSupportedTokenToFund(amount, offsetReceivable)
                    .accountsPartial({
                        operator: operator.publicKey,
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        supportedTokenMint: supportedToken.mint,
                        supportedTokenProgram: supportedToken.program,
                        operatorSupportedTokenAccount: operatorSupportedTokenAddress,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signers: [operator],
            events: ["operatorDonatedToFund"],
        });

        const [fragJTOFund, fragJTOFundSupportedTokenAccount] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragJTOFund),
            this.getFragJTOFundReserveAccountBalance(),
            this.getFragJTOFundSupportedTokenReserveAccount(tokenSymbol),
        ]);
        logger.notice(`operator donated: ${this.lamportsToX(amount, supportedToken.decimals, tokenSymbol)} (${this.lamportsToX(fragJTOFund.oneReceiptTokenAsSol, fragJTOFund.receiptTokenDecimals, 'SOL/fragJTO')})`.padEnd(LOG_PAD_LARGE), operatorSupportedTokenAddress.toString());

        return {
            event,
            error,
            fragJTOFund,
            fragJTOFundSupportedTokenAccount,
        };
    }

    public async runUserRequestWithdrawal(user: web3.Keypair, amount: BN, supportedTokenMint: web3.PublicKey = this.supportedTokenMetadata['JTO'].mint) {
        const {event, error} = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragJTOFundAndRewardAccounts(user)),
                this.program.methods
                    .userRequestWithdrawal(amount, supportedTokenMint)
                    .accountsPartial({
                        user: user.publicKey,
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signers: [user],
            events: ["userRequestedWithdrawalFromFund"],
        });

        logger.notice(
            `requested withdrawal: ${this.lamportsToFragJTO(event.userRequestedWithdrawalFromFund.requestedReceiptTokenAmount)} -> ${supportedTokenMint ?? 'SOL'} #${event.userRequestedWithdrawalFromFund.requestId.toString()}/${event.userRequestedWithdrawalFromFund.batchId.toString()}`.padEnd(LOG_PAD_LARGE),
            user.publicKey.toString()
        );
        const [fragJTOFund, fragJTOFundReserveAccountBalance, fragJTOReward, fragJTOUserFund, fragJTOUserReward, fragJTOUserTokenAccount, fragJTOLockAccount] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragJTOFund),
            this.getFragJTOFundReserveAccountBalance(),
            this.account.rewardAccount.fetch(this.knownAddress.fragJTOReward),
            this.account.userFundAccount.fetch(this.knownAddress.fragJTOUserFund(user.publicKey)),
            this.account.userRewardAccount.fetch(this.knownAddress.fragJTOUserReward(user.publicKey)),
            this.getUserFragJTOAccount(user.publicKey),
            this.getFragJTOFundReceiptTokenLockAccount(),
        ]);

        logger.info(`user fragJTO balance: ${this.lamportsToFragJTO(new BN(fragJTOUserTokenAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());

        return {
            event,
            error,
            fragJTOFund,
            fragJTOFundReserveAccountBalance,
            fragJTOReward,
            fragJTOUserFund,
            fragJTOUserReward,
            fragJTOUserTokenAccount,
            fragJTOLockAccount
        };
    }

    public async runUserCancelWithdrawalRequest(
        user: web3.Keypair,
        requestId: BN,
        supportedTokenMint: web3.PublicKey = this.supportedTokenMetadata['JTO'].mint,
    ) {
        const {event, error} = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragJTOFundAndRewardAccounts(user)),
                this.program.methods
                    .userCancelWithdrawalRequest(requestId, supportedTokenMint)
                    .accountsPartial({
                        user: user.publicKey,
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signers: [user],
            events: ["userCanceledWithdrawalRequestFromFund"],
        });

        logger.notice(`canceled withdrawal request: #${requestId.toString()}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());
        const [fragJTOFund, fragJTOFundReserveAccountBalance, fragJTOReward, fragJTOUserFund, fragJTOUserReward, fragJTOUserTokenAccount, fragJTOLockAccount] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragJTOFund),
            this.getFragJTOFundReserveAccountBalance(),
            this.account.rewardAccount.fetch(this.knownAddress.fragJTOReward),
            this.account.userFundAccount.fetch(this.knownAddress.fragJTOUserFund(user.publicKey)),
            this.account.userRewardAccount.fetch(this.knownAddress.fragJTOUserReward(user.publicKey)),
            this.getUserFragJTOAccount(user.publicKey),
            this.getFragJTOFundReceiptTokenLockAccount(),
        ]);

        logger.info(`user fragJTO balance: ${this.lamportsToFragJTO(new BN(fragJTOUserTokenAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());

        return {
            event,
            error,
            fragJTOFund,
            fragJTOFundReserveAccountBalance,
            fragJTOReward,
            fragJTOUserFund,
            fragJTOUserReward,
            fragJTOUserTokenAccount,
            fragJTOLockAccount
        };
    }

    public async runUserWrapReceiptToken(
        user: web3.Keypair,
        amount: BN,
    ) {
        const {event, error} = await this.run({
            instructions: [
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.wfragJTOUserTokenAccount(user.publicKey),
                    user.publicKey,
                    this.knownAddress.wfragJTOTokenMint,
                ),
                this.methods.userWrapReceiptToken(amount)
                    .accountsPartial({
                        user: user.publicKey,
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        wrappedTokenMint: this.knownAddress.wfragJTOTokenMint,
                    })
                    .instruction(),
            ],
            signers: [user],
            events: ['userWrappedReceiptToken'],
        });

        const [fragJTOFund, fragJTOReward, fragJTOUserFund, fragJTOUserReward, fragJTOFundWrapAccountReward, fragJTOUserTokenAccount, fragJTOWrapAccount, wfragJTOUserTokenAccount] = await Promise.all([
            this.getFragJTOFundAccount(),
            this.getFragJTORewardAccount(),
            this.getUserFragJTOFundAccount(user.publicKey).catch(err => null),
            this.getUserFragJTORewardAccount(user.publicKey).catch(err => null),
            this.getFragJTOFundWrapAccountRewardAccount(),
            this.getUserFragJTOAccount(user.publicKey),
            this.getFragJTOFundReceiptTokenWrapAccount(),
            this.getUserWfragJTOAccount(user.publicKey),
        ]);
        
        logger.info(`user fragJTO balance: ${this.lamportsToFragJTO(new BN(fragJTOUserTokenAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());
        logger.info(`user wfragJTO balance: ${this.lamportsToWfragJTO(new BN(wfragJTOUserTokenAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());
        logger.debug(`total wrapped fragJTO amount: ${this.lamportsToFragJTO(new BN(fragJTOWrapAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOFundReceiptTokenWrapAccount.toString());

        return {
            event,
            error,
            fragJTOFund,
            fragJTOReward,
            fragJTOUserFund,
            fragJTOUserReward,
            fragJTOFundWrapAccountReward,
            fragJTOUserTokenAccount,
            fragJTOWrapAccount,
            wfragJTOUserTokenAccount,
        };
    }

    public async runUserWrapReceiptTokenIfNeeded(
        user: web3.Keypair,
        targetBalance: BN,
    ) {
        const {event, error} = await this.run({
            instructions: [
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.wfragJTOUserTokenAccount(user.publicKey),
                    user.publicKey,
                    this.knownAddress.wfragJTOTokenMint,
                ),
                this.methods.userWrapReceiptTokenIfNeeded(targetBalance)
                    .accountsPartial({
                        user: user.publicKey,
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        wrappedTokenMint: this.knownAddress.wfragJTOTokenMint,
                    })
                    .instruction(),
            ],
            signers: [user],
            events: ['userWrappedReceiptToken'],
        });

        const [fragJTOFund, fragJTOReward, fragJTOUserFund, fragJTOUserReward, fragJTOFundWrapAccountReward, fragJTOUserTokenAccount, fragJTOWrapAccount, wfragJTOUserTokenAccount] = await Promise.all([
            this.getFragJTOFundAccount(),
            this.getFragJTORewardAccount(),
            this.getUserFragJTOFundAccount(user.publicKey).catch(err => null),
            this.getUserFragJTORewardAccount(user.publicKey).catch(err => null),
            this.getFragJTOFundWrapAccountRewardAccount(),
            this.getUserFragJTOAccount(user.publicKey),
            this.getFragJTOFundReceiptTokenWrapAccount(),
            this.getUserWfragJTOAccount(user.publicKey),
        ]);
        
        logger.info(`user fragJTO balance: ${this.lamportsToFragJTO(new BN(fragJTOUserTokenAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());
        logger.info(`user wfragJTO balance: ${this.lamportsToWfragJTO(new BN(wfragJTOUserTokenAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());
        logger.debug(`total wrapped fragJTO amount: ${this.lamportsToFragJTO(new BN(fragJTOWrapAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOFundReceiptTokenWrapAccount.toString());

        return {
            event,
            error,
            fragJTOFund,
            fragJTOReward,
            fragJTOUserFund,
            fragJTOUserReward,
            fragJTOFundWrapAccountReward,
            fragJTOUserTokenAccount,
            fragJTOWrapAccount,
            wfragJTOUserTokenAccount,
        };
    }

    public async runUserUnwrapReceiptToken(
        user: web3.Keypair,
        amount: BN,
    ) {
        const {event, error} = await this.run({
            instructions: [
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.fragJTOUserTokenAccount(user.publicKey),
                    user.publicKey,
                    this.knownAddress.fragJTOTokenMint,
                    spl.TOKEN_2022_PROGRAM_ID,
                ),
                this.methods.userUnwrapReceiptToken(amount)
                    .accountsPartial({
                        user: user.publicKey,
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        wrappedTokenMint: this.knownAddress.wfragJTOTokenMint,
                    })
                    .instruction(),
            ],
            signers: [user],
            events: ['userUnwrappedReceiptToken'],
        });

        const [fragJTOFund, fragJTOReward, fragJTOUserFund, fragJTOUserReward, fragJTOFundWrapAccountReward, fragJTOUserTokenAccount, fragJTOWrapAccount, wfragJTOUserTokenAccount] = await Promise.all([
            this.getFragJTOFundAccount(),
            this.getFragJTORewardAccount(),
            this.getUserFragJTOFundAccount(user.publicKey).catch(err => null),
            this.getUserFragJTORewardAccount(user.publicKey).catch(err => null),
            this.getFragJTOFundWrapAccountRewardAccount(),
            this.getUserFragJTOAccount(user.publicKey),
            this.getFragJTOFundReceiptTokenWrapAccount(),
            this.getUserWfragJTOAccount(user.publicKey),
        ]);
        
        logger.info(`user fragJTO balance: ${this.lamportsToFragJTO(new BN(fragJTOUserTokenAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());
        logger.info(`user wfragJTO balance: ${this.lamportsToWfragJTO(new BN(wfragJTOUserTokenAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());
        logger.debug(`total wrapped fragJTO amount: ${this.lamportsToFragJTO(new BN(fragJTOWrapAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOFundReceiptTokenWrapAccount.toString());

        return {
            event,
            error,
            fragJTOFund,
            fragJTOReward,
            fragJTOUserFund,
            fragJTOUserReward,
            fragJTOFundWrapAccountReward,
            fragJTOUserTokenAccount,
            fragJTOWrapAccount,
            wfragJTOUserTokenAccount,
        };
    }

    public lamportsToWfragJTO(lamports: BN): string {
        return super.lamportsToX(lamports, this.fragJTODecimals, "wfragJTO");
    }

    public async runOperatorUpdateFundPrices(operator: web3.Keypair = this.wallet) {
        const {event, error} = await this.run({
            instructions: [
                web3.ComputeBudgetProgram.setComputeUnitLimit({
                    units: 300_000,
                }),
                this.program.methods
                    .operatorUpdateFundPrices()
                    .accountsPartial({
                        operator: operator.publicKey,
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signers: [operator],
            events: ["operatorUpdatedFundPrices"],
        });

        const [fragJTOFund] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragJTOFund),
        ]);
        logger.notice(`operator updated fund prices: ${this.lamportsToSOL(fragJTOFund.oneReceiptTokenAsSol)}/fragJTO`.padEnd(LOG_PAD_LARGE), operator.publicKey.toString());

        return {
            event,
            error,
            fragJTOFund,
        };
    }

    public async runOperatorEnqueueWithdrawalBatches(operator: web3.Keypair = this.keychain.getKeypair('ADMIN'), forced: boolean = false) {
        const {event, error} = await this.runOperatorFundCommands({
            command: {
                enqueueWithdrawalBatch: {
                    0: {
                        forced: forced,
                    }
                }
            },
            requiredAccounts: [],
        }, operator);

        const [fragJTOFund, fragJTOFundReserveAccountBalance, fragJTOReward, fragJTOLockAccount] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragJTOFund),
            this.getFragJTOFundReserveAccountBalance(),
            this.account.rewardAccount.fetch(this.knownAddress.fragJTOReward),
            this.getFragJTOFundReceiptTokenLockAccount(),
        ]);
        logger.info(`operator enqueued withdrawal batches up to #${fragJTOFund.sol.withdrawalLastProcessedBatchId.toString()}`.padEnd(LOG_PAD_LARGE), operator.publicKey.toString());

        return {event, error, fragJTOFund, fragJTOFundReserveAccountBalance, fragJTOReward, fragJTOLockAccount};
    }

    public async runOperatorInitialize(operator: web3.Keypair = this.keychain.getKeypair('ADMIN')) {
        await this.runOperatorFundCommands({
            command: {
                initialize: {
                    0: {
                        state: {
                            new: {},
                        }
                    }
                }
            },
            requiredAccounts: [],
        }, operator);
    }

    public async runOperatorProcessWithdrawalBatches(operator: web3.Keypair = this.keychain.getKeypair('ADMIN'), forced: boolean = false) {
        const {event: _event, error: _error} = await this.runOperatorFundCommands({
            command: {
                enqueueWithdrawalBatch: {
                    0: {
                        forced: forced,
                    }
                }
            },
            requiredAccounts: [],
        }, operator);

        const {event, error} = await this.runOperatorFundCommands({
            command: {
                processWithdrawalBatch: {
                    0: {
                        state: {
                            new: {},
                        },
                        forced: forced,
                    }
                }
            },
            requiredAccounts: [],
        }, operator);

        const [fragJTOFund, fragJTOFundReserveAccountBalance, fragJTOReward, fragJTOLockAccount] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragJTOFund),
            this.getFragJTOFundReserveAccountBalance(),
            this.account.rewardAccount.fetch(this.knownAddress.fragJTOReward),
            this.getFragJTOFundReceiptTokenLockAccount(),
        ]);
        logger.info(`operator processed withdrawal batches up to #${fragJTOFund.sol.withdrawalLastProcessedBatchId.toString()}`.padEnd(LOG_PAD_LARGE), operator.publicKey.toString());

        return {event, error, fragJTOFund, fragJTOFundReserveAccountBalance, fragJTOReward, fragJTOLockAccount};
    }

    public async runUserWithdraw(user: web3.Keypair, supportedTokenMint: web3.PublicKey, requestId: BN) {
        const request = await this.getUserFragJTOFundAccount(user.publicKey)
            .then(userFundAccount => userFundAccount.withdrawalRequests.find(req => req.requestId.eq(requestId) && (supportedTokenMint ? supportedTokenMint.equals(req.supportedTokenMint) : !req.supportedTokenMint)));

        if (!request) {
            throw "request not found";
        }
        const userSupportedTokenAccount = request.supportedTokenMint ? spl.getAssociatedTokenAddressSync(request.supportedTokenMint, user.publicKey, true, request.supportedTokenProgram) : null;

        const {event, error} = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragJTOFundAndRewardAccounts(user)),
                ...(
                    request.supportedTokenMint ? [
                        spl.createAssociatedTokenAccountIdempotentInstruction(
                            this.wallet.publicKey,
                            userSupportedTokenAccount,
                            user.publicKey,
                            request.supportedTokenMint,
                            request.supportedTokenProgram,
                        ),
                        this.program.methods
                            .userWithdrawSupportedToken(request.batchId, request.requestId)
                            .accountsPartial({
                                receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                                user: user.publicKey,
                                userSupportedTokenAccount,
                                supportedTokenMint: request.supportedTokenMint,
                                supportedTokenProgram: request.supportedTokenProgram,
                            })
                            .instruction(),
                    ] : [
                        this.program.methods
                            .userWithdrawSol(request.batchId, request.requestId)
                            .accountsPartial({
                                user: user.publicKey,
                                receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                            })
                            .instruction(),
                    ]
                )
            ],
            signers: [user],
            events: ["userWithdrewFromFund"],
        });

        const [fragJTOFund, fragJTOFundReserveAccountBalance, fragJTOReward, fragJTOUserFund, fragJTOUserReward, fragJTOUserTokenAccount, fragJTOLockAccount] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragJTOFund),
            this.getFragJTOFundReserveAccountBalance(),
            this.account.rewardAccount.fetch(this.knownAddress.fragJTOReward),
            this.account.userFundAccount.fetch(this.knownAddress.fragJTOUserFund(user.publicKey)),
            this.account.userRewardAccount.fetch(this.knownAddress.fragJTOUserReward(user.publicKey)),
            this.getUserFragJTOAccount(user.publicKey),
            this.getFragJTOFundReceiptTokenLockAccount(),
        ]);
        logger.notice(`user withdrew: ${this.lamportsToX(event.userWithdrewFromFund.withdrawnAmount, 9, event.userWithdrewFromFund.supportedTokenMint?.toString().substring(0, 4) ?? 'SOL' /** TODO: later.. **/)} #${requestId.toString()}, (${this.lamportsToX(fragJTOFund.oneReceiptTokenAsSol, fragJTOFund.receiptTokenDecimals, 'SOL/fragJTO')})`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());

        return {
            event,
            error,
            fragJTOFund,
            fragJTOFundReserveAccountBalance,
            fragJTOReward,
            fragJTOUserFund,
            fragJTOUserReward,
            fragJTOUserTokenAccount,
            fragJTOLockAccount
        };
    }

    public async runTransfer(source: web3.Keypair, destination: web3.PublicKey, amount: BN) {
        const {event, error} = await this.run({
            instructions: [
                spl.createTransferCheckedWithTransferHookInstruction(
                    this.connection,
                    this.knownAddress.fragJTOUserTokenAccount(source.publicKey),
                    this.knownAddress.fragJTOTokenMint,
                    this.knownAddress.fragJTOUserTokenAccount(destination),
                    source.publicKey,
                    BigInt(amount.toString()),
                    this.fragJTODecimals,
                    [],
                    "confirmed",
                    spl.TOKEN_2022_PROGRAM_ID
                ),
            ],
            signers: [source],
            events: ["userTransferredReceiptToken"],
        });

        return {event, error};
    }

    public async runUserUpdateRewardPools(user: web3.Keypair) {
        const {event, error} = await this.run({
            instructions: [
                this.program.methods
                    .userUpdateRewardPools()
                    .accountsPartial({
                        user: user.publicKey,
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                    })
                    .instruction(),
            ],
            signers: [user],
            events: ['userUpdatedRewardPool'],
        });

        logger.notice(`user manually updated user reward pool:`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());
        const [fragJTOUserReward] = await Promise.all([this.account.userRewardAccount.fetch(this.knownAddress.fragJTOUserReward(user.publicKey))]);

        return {event, error, fragJTOUserReward};
    }

    public async runOperatorUpdateRewardPools(operator: web3.Keypair = this.wallet) {
        const {event, error} = await this.run({
            instructions: [
                this.program.methods
                    .operatorUpdateRewardPools()
                    .accountsPartial({
                        operator: operator.publicKey,
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                    })
                    .instruction(),
            ],
            signers: [operator],
            events: ["operatorUpdatedRewardPools"], // won't emit it for such void update requests
        });

        logger.notice(`operator manually updated global reward pool:`.padEnd(LOG_PAD_LARGE), operator.publicKey.toString());
        const [fragJTOReward] = await Promise.all([this.getFragJTORewardAccount()]);

        return {event, error, fragJTOReward};
    }

    public async runOperatorRunScheduled(i = 0) {
        logger.notice(`operation ${i}`, new Date().toString());
        try {
            await this.runOperatorFundCommands(null, this.keychain.getKeypair('ADMIN'), 100, undefined, 100_000);
            await new Promise(resolve => setTimeout(resolve, 1000 * 60 * 30));
        } catch (err) {
            console.error(err);
            await new Promise(resolve => setTimeout(resolve, 1000 * 60));
        }
        this.runOperatorRunScheduled(i+1);
    }

    public async runOperatorFundCommands(resetCommand: Parameters<typeof this.program.methods.operatorRunFundCommand>[0] = null, operator: web3.Keypair = this.keychain.getKeypair('ADMIN'), maxTxCount = 100, setComputeUnitLimitUnits?: number, setComputeUnitPriceMicroLamports?: number) {
        let txCount = 0;
        while (txCount < maxTxCount) {
            const {event, error} = await this.runOperatorSingleFundCommand(operator, txCount == 0 ? resetCommand : null, setComputeUnitLimitUnits, setComputeUnitPriceMicroLamports);
            txCount++;
            if (txCount == maxTxCount || event.operatorRanFundCommand.nextSequence == 0) {
                return {event, error}
            }
        }
    }

    private async runOperatorSingleFundCommand(operator: web3.Keypair, resetCommand?: Parameters<typeof this.program.methods.operatorRunFundCommand>[0], setComputeUnitLimitUnits: number = 1_600_000, setComputeUnitPriceMicroLamports: number = 1_000_000) {
        // prepare accounts according to the current state of operation.
        // - can contain 57 accounts out of 64 with reserved 6 accounts and payer.
        // - order doesn't matter, no need to put duplicate.
        const requiredAccounts: Map<web3.PublicKey, web3.AccountMeta> = new Map();
        this.pricingSourceAccounts.forEach(accoutMeta => {
            requiredAccounts.set(accoutMeta.pubkey, accoutMeta);
        });
        requiredAccounts.set(this.knownAddress.programEventAuthority, {
            pubkey: this.knownAddress.programEventAuthority,
            isWritable: false,
            isSigner: false,
        });
        requiredAccounts.set(this.programId, {
            pubkey: this.programId,
            isWritable: false,
            isSigner: false,
        });
        requiredAccounts.set(operator.publicKey, {
            pubkey: operator.publicKey,
            isWritable: true,
            isSigner: true,
        });
        requiredAccounts.set(web3.SystemProgram.programId, {
            pubkey: web3.SystemProgram.programId,
            isWritable: false,
            isSigner: false,
        });
        requiredAccounts.set(this.knownAddress.fragJTOTokenMint, {
            pubkey: this.knownAddress.fragJTOTokenMint,
            isWritable: true,
            isSigner: false,
        });
        requiredAccounts.set(this.knownAddress.fragJTOFund, {
            pubkey: this.knownAddress.fragJTOFund,
            isWritable: true,
            isSigner: false,
        });

        let fragJTOFund = await this.getFragJTOFundAccount();
        let nextOperationCommand = resetCommand ?? fragJTOFund.operation.nextCommand;
        let nextOperationSequence = resetCommand ? 0 : fragJTOFund.operation.nextSequence;
        if (nextOperationCommand) {
            for (const accountMeta of nextOperationCommand.requiredAccounts) {
                if (requiredAccounts.has(accountMeta.pubkey)) {
                    if (accountMeta.isWritable != 0) {
                        requiredAccounts.get(accountMeta.pubkey).isWritable = true;
                    }
                } else {
                    requiredAccounts.set(accountMeta.pubkey, {
                        pubkey: accountMeta.pubkey,
                        isWritable: (accountMeta.isWritable != 0),
                        isSigner: false,
                    });
                }
            }
        }

        const tx = await this.run({
            instructions: [
                this.program.methods
                    .operatorRunFundCommand(resetCommand)
                    .accountsPartial({
                        operator: operator.publicKey,
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                    })
                    .remainingAccounts(Array.from(requiredAccounts.values()))
                    .instruction(),
            ],
            signers: [operator],
            events: ["operatorRanFundCommand"],
            skipPreflight: true,
            // TODO: why is requestHeapFrameBytes not working?
            // requestHeapFrameBytes, : 64 * 1024,
            setComputeUnitLimitUnits,
            setComputeUnitPriceMicroLamports,
        });

        const executedCommand = tx.event.operatorRanFundCommand.command;
        const commandResult = tx.event.operatorRanFundCommand.result;
        const commandName = Object.keys(executedCommand)[0];
        logger.notice(`operator ran command#${nextOperationSequence}: ${commandName}`.padEnd(LOG_PAD_LARGE));
        console.log('executed command:', executedCommand[commandName][0]);
        console.log('executed result:', commandResult && commandResult[commandName][0]);

        return {
            event: tx.event,
            error: tx.error,
        };
    }
}
