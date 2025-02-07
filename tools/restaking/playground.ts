import * as anchor from "@coral-xyz/anchor";
import {BN, web3} from "@coral-xyz/anchor";
// @ts-ignore
import * as splTokenMetadata from "@solana/spl-token-metadata";
import * as splStakePool from "@solana/spl-stake-pool";
import * as marinade from "@marinade.finance/marinade-ts-sdk";
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
            () => this.runAdminInitializeFragSOLTokenMint(), // 0
            () => this.runAdminInitializeOrUpdateFundAccount(), // 1
            () => this.runAdminInitializeOrUpdateRewardAccount(), // 2
            () => this.runAdminInitializeFragSOLExtraAccountMetaList(), // 3
            () => this.runAdminInitializeNSOLTokenMint(), // 4
            () => this.runAdminInitializeNormalizedTokenPoolAccounts(), // 5
            () => this.runFundManagerInitializeNormalizeTokenPoolSupportedTokens(), // 6
            () => this.runFundManagerInitializeRewardPools(), // 7
            () => this.runFundManagerSettleReward({ poolName: "bonus", rewardName: "fPoint", amount: new BN(0) }), // 8
            () => this.runFundManagerInitializeFundSupportedTokens(), // 9
            () => this.runFundManagerInitializeFundNormalizedToken(), // 10
            () => this.runFundManagerInitializeFundJitoRestakingVaults(), // 11
            () => this.runFundManagerUpdateFundConfigurations(), // 12
            () => this.runAdminInitializeWFragSOLTokenMint(), // 13
            () => this.runAdminInitializeOrUpdateFundWrapAccountRewardAccount(), // 14
            () => this.runFundManagerInitializeFundWrappedToken(), // 15
        ];
    }

    public async getOrCreateKnownAddressLookupTable() {
        if (this._knownAddressLookupTableAddress) {
            return this._knownAddressLookupTableAddress;
        }

        const existingLookupTableAddress = this.getConstantAsPublicKey('fragsolAddressLookupTableAddress');
        const existingLookupTable = await this.connection.getAddressLookupTable(existingLookupTableAddress)
            .then(res => res.value, () => null);
        if (existingLookupTable) {
            this._knownAddressLookupTableAddress = existingLookupTableAddress;
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

    public get knownAddressLookupTableAddress() {
        return this._knownAddressLookupTableAddress
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

        // fragSOL
        const fragSOLTokenMint = this.getConstantAsPublicKey("fragsolMintAddress");
        const fragSOLTokenMintBuf = fragSOLTokenMint.toBuffer();
        const fragSOLExtraAccountMetasAccount = spl.getExtraAccountMetaAddress(fragSOLTokenMint, this.programId);

        // wFragSOL
        const wFragSOLTokenMint = this.getConstantAsPublicKey("fragsolWrappedTokenMintAddress");

        // nSOL
        const nSOLTokenMint = this.getConstantAsPublicKey("fragsolNormalizedTokenMintAddress");
        const nSOLTokenMintBuf = nSOLTokenMint.toBuffer();

        // jitoSOL
        const jitoSOLTokenMint = this.supportedTokenMetadata['jitoSOL'].mint;

        // fragSOL jito nSOL VRT
        const fragSOLJitoNSOLVRTMint = this.getConstantAsPublicKey('fragsolJitoNsolVaultReceiptTokenMintAddress');

        // fragSOL jito jitoSOL VRT
        const fragSOLJitoJitoSOLVRTMint = this.getConstantAsPublicKey('fragsolJitoJitosolVaultReceiptTokenMintAddress');

        // fragSOL fund & ATAs
        const [fragSOLFund] = web3.PublicKey.findProgramAddressSync([Buffer.from("fund"), fragSOLTokenMintBuf], this.programId);
        const [fragSOLFundReserveAccount] = web3.PublicKey.findProgramAddressSync([Buffer.from("fund_reserve"), fragSOLTokenMintBuf], this.programId);
        const fragSOLFundSupportedTokenReserveAccount = (symbol: keyof typeof this.supportedTokenMetadata) =>
            spl.getAssociatedTokenAddressSync(this.supportedTokenMetadata[symbol].mint, fragSOLFundReserveAccount, true, this.supportedTokenMetadata[symbol].program);
        const fragSOLFundSupportedTokenReserveAccounts = Object.keys(this.supportedTokenMetadata).reduce((obj, symbol) => ({
            [`fragSOLFundSupportedTokenReserveAccount_${symbol}`]: fragSOLFundSupportedTokenReserveAccount(symbol as any),
            ...obj,
        }), {} as { string: web3.PublicKey });
        const [fragSOLFundTreasuryAccount] = web3.PublicKey.findProgramAddressSync([Buffer.from("fund_treasury"), fragSOLTokenMintBuf], this.programId);
        const fragSOLFundSupportedTokenTreasuryAccount = (symbol: keyof typeof this.supportedTokenMetadata) =>
            spl.getAssociatedTokenAddressSync(this.supportedTokenMetadata[symbol].mint, fragSOLFundTreasuryAccount, true, this.supportedTokenMetadata[symbol].program);
        const fragSOLFundSupportedTokenTreasuryAccounts = Object.keys(this.supportedTokenMetadata).reduce((obj, symbol) => ({
            [`fragSOLFundSupportedTokenTreasuryAccount_${symbol}`]: fragSOLFundSupportedTokenTreasuryAccount(symbol as any),
            ...obj,
        }), {} as { string: web3.PublicKey });
        const [fragSOLFundWrapAccount] = web3.PublicKey.findProgramAddressSync([Buffer.from("fund_wrap"), fragSOLTokenMintBuf], this.programId);
        const fragSOLFundReceiptTokenWrapAccount = spl.getAssociatedTokenAddressSync(
            fragSOLTokenMint,
            fragSOLFundWrapAccount,
            true,
            spl.TOKEN_2022_PROGRAM_ID,
        );

        const fragSOLFundReceiptTokenLockAccount = spl.getAssociatedTokenAddressSync(
            fragSOLTokenMint,
            fragSOLFund,
            true,
            spl.TOKEN_2022_PROGRAM_ID,
        );

        const fragSOLFundNSOLAccount = spl.getAssociatedTokenAddressSync(
            nSOLTokenMint,
            fragSOLFundReserveAccount,
            true,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        const fragSOLFundJitoNSOLVRTAccount = spl.getAssociatedTokenAddressSync(
            fragSOLJitoNSOLVRTMint,
            fragSOLFundReserveAccount,
            true,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        const fragSOLFundJitoJitoSOLVRTAccount = spl.getAssociatedTokenAddressSync(
            fragSOLJitoJitoSOLVRTMint,
            fragSOLFundReserveAccount,
            true,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );

        const fragSOLUserFund = (user: web3.PublicKey) => web3.PublicKey.findProgramAddressSync([Buffer.from("user_fund"), fragSOLTokenMintBuf, user.toBuffer()], this.programId)[0];
        const fragSOLUserTokenAccount = (user: web3.PublicKey) => spl.getAssociatedTokenAddressSync(fragSOLTokenMint, user, false, spl.TOKEN_2022_PROGRAM_ID);
        const wFragSOLUserTokenAccount = (user: web3.PublicKey) => spl.getAssociatedTokenAddressSync(wFragSOLTokenMint, user, false);
        const userSupportedTokenAccount = (user: web3.PublicKey, symbol: keyof typeof this.supportedTokenMetadata) =>
            spl.getAssociatedTokenAddressSync(this.supportedTokenMetadata[symbol].mint, user, false, this.supportedTokenMetadata[symbol].program);

        // reward
        const [fragSOLReward] = web3.PublicKey.findProgramAddressSync([Buffer.from("reward"), fragSOLTokenMintBuf], this.programId);
        const fragSOLUserReward = (user: web3.PublicKey) => web3.PublicKey.findProgramAddressSync([Buffer.from("user_reward"), fragSOLTokenMintBuf, user.toBuffer()], this.programId)[0];
        const fragSOLFundWrapAccountReward = fragSOLUserReward(fragSOLFundWrapAccount);

        // NTP
        const [nSOLTokenPool] = web3.PublicKey.findProgramAddressSync([Buffer.from("nt_pool"), nSOLTokenMintBuf], this.programId);
        const nSOLSupportedTokenReserveAccount = (symbol: keyof typeof this.supportedTokenMetadata) =>
            spl.getAssociatedTokenAddressSync(this.supportedTokenMetadata[symbol].mint, nSOLTokenPool, true, this.supportedTokenMetadata[symbol].program);
        const nSOLSupportedTokenReserveAccounts = Object.keys(this.supportedTokenMetadata).reduce((obj, symbol) => ({
            [`nSOLSupportedTokenReserveAccount_${symbol}`]: nSOLSupportedTokenReserveAccount(symbol as any),
            ...obj,
        }), {} as { string: web3.PublicKey });

        // jito
        const jitoVaultProgram = this.getConstantAsPublicKey('jitoVaultProgramId');
        const jitoVaultProgramFeeWallet = this.getConstantAsPublicKey('jitoVaultProgramFeeWallet');
        const jitoVaultFeeWallet = this.keychain.getPublicKey('ADMIN');
        const jitoVaultConfig = this.getConstantAsPublicKey('jitoVaultConfigAddress');
        const jitoRestakingProgram = this.getConstantAsPublicKey('jitoRestakingProgramId');
        const jitoRestakingConfig = this.getConstantAsPublicKey('jitoRestakingConfigAddress');

        // fragSOL jito vault
        const fragSOLJitoVaultUpdateStateTracker = (vaultAccount: web3.PublicKey) => {
            return (slot: BN, epochLength: BN) => {
                let ncnEpoch = slot.div(epochLength).toBuffer('le', 8);
                return web3.PublicKey.findProgramAddressSync([Buffer.from('vault_update_state_tracker'), vaultAccount.toBuffer(), ncnEpoch], jitoVaultProgram)[0];
            };
        }

        // fragSOL jito nSOL vault
        const fragSOLJitoNSOLVaultAccount = this.getConstantAsPublicKey('fragsolJitoNsolVaultAccountAddress');
        const fragSOLJitoNSOLVaultUpdateStateTracker = fragSOLJitoVaultUpdateStateTracker(fragSOLJitoNSOLVaultAccount);
        const fragSOLJitoNSOLVaultTokenAccount = spl.getAssociatedTokenAddressSync(
            nSOLTokenMint,
            fragSOLJitoNSOLVaultAccount,
            true,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        const fragSOLJitoNSOLVaultProgramFeeWalletTokenAccount = spl.getAssociatedTokenAddressSync(
            fragSOLJitoNSOLVRTMint,
            jitoVaultProgramFeeWallet,
            true,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        const fragSOLJitoNSOLVaultFeeWalletTokenAccount = spl.getAssociatedTokenAddressSync(
            fragSOLJitoNSOLVRTMint,
            jitoVaultFeeWallet,
            false,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );

        // fragSOL jito jitoSOL vault
        const fragSOLJitoJitoSOLVaultAccount = this.getConstantAsPublicKey('fragsolJitoJitosolVaultAccountAddress');
        const fragSOLJitoJitoSOLVaultUpdateStateTracker = fragSOLJitoVaultUpdateStateTracker(fragSOLJitoJitoSOLVaultAccount);
        const fragSOLJitoJitoSOLVaultTokenAccount = spl.getAssociatedTokenAddressSync(
            jitoSOLTokenMint,
            fragSOLJitoJitoSOLVaultAccount,
            true,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        const fragSOLJitoJitoSOLVaultProgramFeeWalletTokenAccount = spl.getAssociatedTokenAddressSync(
            fragSOLJitoJitoSOLVRTMint,
            jitoVaultProgramFeeWallet,
            true,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        const fragSOLJitoJitoSOLVaultFeeWalletTokenAccount = spl.getAssociatedTokenAddressSync(
            fragSOLJitoJitoSOLVRTMint,
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
            // fragSOL
            fragSOLTokenMint,
            fragSOLExtraAccountMetasAccount,
            // wFragSOL
            wFragSOLTokenMint,
            // nSOL
            nSOLTokenMint,
            // jitoSOL
            jitoSOLTokenMint,
            // fragSOL jito nSOL VRT
            fragSOLJitoNSOLVRTMint,
            // fragSOL jito jitoSOL VRT
            fragSOLJitoJitoSOLVRTMint,
            // fragSOL fund & ATAs
            fragSOLFund,
            fragSOLFundReserveAccount,
            fragSOLFundSupportedTokenReserveAccount,
            ...fragSOLFundSupportedTokenReserveAccounts,
            fragSOLFundTreasuryAccount,
            fragSOLFundSupportedTokenTreasuryAccount,
            ...fragSOLFundSupportedTokenTreasuryAccounts,
            fragSOLFundWrapAccount,
            fragSOLFundReceiptTokenWrapAccount,
            fragSOLFundReceiptTokenLockAccount,
            fragSOLFundNSOLAccount,
            fragSOLFundJitoNSOLVRTAccount,
            fragSOLFundJitoJitoSOLVRTAccount,
            fragSOLUserFund,
            fragSOLUserTokenAccount,
            wFragSOLUserTokenAccount,
            userSupportedTokenAccount,
            // reward
            fragSOLReward,
            fragSOLUserReward,
            fragSOLFundWrapAccountReward,
            // NTP
            nSOLTokenPool,
            nSOLSupportedTokenReserveAccount,
            ...nSOLSupportedTokenReserveAccounts,
            // jito
            jitoVaultProgram,
            jitoVaultProgramFeeWallet,
            jitoVaultFeeWallet,
            jitoVaultConfig,
            jitoRestakingProgram,
            jitoRestakingConfig,
            // fragSOL jito nSOL vault
            fragSOLJitoNSOLVaultAccount,
            fragSOLJitoNSOLVaultUpdateStateTracker,
            fragSOLJitoNSOLVaultTokenAccount,
            fragSOLJitoNSOLVaultProgramFeeWalletTokenAccount,
            fragSOLJitoNSOLVaultFeeWalletTokenAccount,
            // fragSOL jito jitoSOL vault
            fragSOLJitoJitoSOLVaultAccount,
            fragSOLJitoJitoSOLVaultUpdateStateTracker,
            fragSOLJitoJitoSOLVaultTokenAccount,
            fragSOLJitoJitoSOLVaultProgramFeeWalletTokenAccount,
            fragSOLJitoJitoSOLVaultFeeWalletTokenAccount,
            // program revenue
            programRevenueAccount,
            programSupportedTokenRevenueAccount,
            ...programSupportedTokenRevenueAccounts,

            tokenProgram: spl.TOKEN_PROGRAM_ID,
            token2022Program: spl.TOKEN_2022_PROGRAM_ID,
            sysvarClock: web3.SYSVAR_CLOCK_PUBKEY,
            sysvarStakeHistory: web3.SYSVAR_STAKE_HISTORY_PUBKEY,
            stakeProgram: web3.StakeProgram.programId,
            systemProgram: web3.SystemProgram.programId,
            splStakePoolProgram: splStakePool.STAKE_POOL_PROGRAM_ID,
        };
    }

    public readonly fragSOLDecimals = 9;
    public readonly nSOLDecimals = 9;
    public readonly wFragSOLDecimals = 9;
    public readonly vrtDecimals = 9;

    public get supportedTokenMetadata() {
        if (this._supportedTokenMetadata) return this._supportedTokenMetadata;
        return (this._supportedTokenMetadata = this._getSupportedTokenMetadata());
    }

    private _supportedTokenMetadata: ReturnType<typeof this._getSupportedTokenMetadata>;

    private _getSupportedTokenMetadata() {
        if (this.isDevnet) {
            return {
                bSOL: {
                    mint: this.getConstantAsPublicKey("devnetBsolMintAddress"),
                    program: spl.TOKEN_PROGRAM_ID,
                    pricingSourceAddress: this.getConstantAsPublicKey("devnetBsolStakePoolAddress"),
                    pricingSource: {
                        splStakePool: {
                            address: this.getConstantAsPublicKey("devnetBsolStakePoolAddress"),
                        },
                    },
                    decimals: 9,
                },
                mSOL: {
                    mint: this.getConstantAsPublicKey("devnetMsolMintAddress"),
                    program: spl.TOKEN_PROGRAM_ID,
                    pricingSourceAddress: this.getConstantAsPublicKey("devnetMsolStakePoolAddress"),
                    pricingSource: {
                        marinadeStakePool: {
                            address: this.getConstantAsPublicKey("devnetMsolStakePoolAddress"),
                        },
                    },
                    decimals: 9,
                },
                jitoSOL: {
                    mint: this.getConstantAsPublicKey("devnetJitosolMintAddress"),
                    program: spl.TOKEN_PROGRAM_ID,
                    pricingSourceAddress: this.getConstantAsPublicKey("devnetJitosolStakePoolAddress"),
                    pricingSource: {
                        splStakePool: {
                            address: this.getConstantAsPublicKey("devnetJitosolStakePoolAddress"),
                        },
                    },
                    decimals: 9,
                },
            };
        } else {
            // for 'localnet', it would be cloned from mainnet
            let metadata = {
                bSOL: {
                    mint: this.getConstantAsPublicKey("mainnetBsolMintAddress"),
                    program: spl.TOKEN_PROGRAM_ID,
                    pricingSourceAddress: this.getConstantAsPublicKey("mainnetBsolStakePoolAddress"),
                    pricingSource: {
                        splStakePool: {
                            address: this.getConstantAsPublicKey("mainnetBsolStakePoolAddress"),
                        },
                    },
                    decimals: 9,
                },
                jitoSOL: {
                    mint: this.getConstantAsPublicKey("mainnetJitosolMintAddress"),
                    program: spl.TOKEN_PROGRAM_ID,
                    pricingSourceAddress: this.getConstantAsPublicKey("mainnetJitosolStakePoolAddress"),
                    pricingSource: {
                        splStakePool: {
                            address: this.getConstantAsPublicKey("mainnetJitosolStakePoolAddress"),
                        },
                    },
                    decimals: 9,
                },
                mSOL: {
                    mint: this.getConstantAsPublicKey("mainnetMsolMintAddress"),
                    program: spl.TOKEN_PROGRAM_ID,
                    pricingSourceAddress: this.getConstantAsPublicKey("mainnetMsolStakePoolAddress"),
                    pricingSource: {
                        marinadeStakePool: {
                            address: this.getConstantAsPublicKey("mainnetMsolStakePoolAddress"),
                        },
                    },
                    decimals: 9,
                },
                BNSOL: {
                    mint: this.getConstantAsPublicKey("mainnetBnsolMintAddress"),
                    program: spl.TOKEN_PROGRAM_ID,
                    pricingSourceAddress: this.getConstantAsPublicKey("mainnetBnsolStakePoolAddress"),
                    pricingSource: {
                        splStakePool: {
                            address: this.getConstantAsPublicKey("mainnetBnsolStakePoolAddress"),
                        },
                    },
                    decimals: 9,
                },
                bbSOL: {
                    mint: this.getConstantAsPublicKey("mainnetBbsolMintAddress"),
                    program: spl.TOKEN_PROGRAM_ID,
                    pricingSourceAddress: this.getConstantAsPublicKey("mainnetBbsolStakePoolAddress"),
                    pricingSource: {
                        sanctumSingleValidatorSplStakePool: {
                            address: this.getConstantAsPublicKey("mainnetBbsolStakePoolAddress"),
                        },
                    },
                    decimals: 9,
                },
            };

            // Later we will remove bSOL.

            // // remove bSOL
            // delete metadata.bSOL;

            return metadata
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
                jitoNSOLVault: {
                    VSTMint: this.knownAddress.nSOLTokenMint,
                    VRTMint: this.knownAddress.fragSOLJitoNSOLVRTMint,
                    vault: this.getConstantAsPublicKey("fragsolJitoNsolVaultAccountAddress"),
                    operators: [],
                    program: this.getConstantAsPublicKey("jitoVaultProgramId"),
                    programFeeWalletTokenAccount: this.knownAddress.fragSOLJitoNSOLVaultProgramFeeWalletTokenAccount,
                    feeWalletTokenAccount: this.knownAddress.fragSOLJitoNSOLVaultFeeWalletTokenAccount,
                    vaultTokenAccount: this.knownAddress.fragSOLJitoNSOLVaultTokenAccount,
                    fundVRTAccount: this.knownAddress.fragSOLFundJitoNSOLVRTAccount,
                },
                jitoJitoSOLVault: {
                    VSTMint: this.knownAddress.jitoSOLTokenMint,
                    VRTMint: this.knownAddress.fragSOLJitoJitoSOLVRTMint,
                    vault: this.getConstantAsPublicKey("fragsolJitoJitosolVaultAccountAddress"),
                    operators: [],
                    program: this.getConstantAsPublicKey("jitoVaultProgramId"),
                    programFeeWalletTokenAccount: this.knownAddress.fragSOLJitoJitoSOLVaultProgramFeeWalletTokenAccount,
                    feeWalletTokenAccount: this.knownAddress.fragSOLJitoJitoSOLVaultFeeWalletTokenAccount,
                    vaultTokenAccount: this.knownAddress.fragSOLJitoJitoSOLVaultTokenAccount,
                    fundVRTAccount: this.knownAddress.fragSOLFundJitoJitoSOLVRTAccount,
                }
            };
        } else {
            // for 'localnet', it would be cloned from mainnet
            return {
                jitoNSOLVault: {
                    VSTMint: this.knownAddress.nSOLTokenMint,
                    VRTMint: this.knownAddress.fragSOLJitoNSOLVRTMint,
                    vault: this.getConstantAsPublicKey("fragsolJitoNsolVaultAccountAddress"),
                    operators: [
                        // TODO v0.4.2: operator mock file is missing?!
                        // new web3.PublicKey("2p4kQZTYL3jKHpkjTaFULvqcKNsF8LoeFGEHWYt2sJAV"),
                    ],
                    program: this.getConstantAsPublicKey("jitoVaultProgramId"),
                    programFeeWalletTokenAccount: this.knownAddress.fragSOLJitoNSOLVaultProgramFeeWalletTokenAccount,
                    feeWalletTokenAccount: this.knownAddress.fragSOLJitoNSOLVaultFeeWalletTokenAccount,
                    vaultTokenAccount: this.knownAddress.fragSOLJitoNSOLVaultTokenAccount,
                    fundVRTAccount: this.knownAddress.fragSOLFundJitoNSOLVRTAccount,
                },
                jitoJitoSOLVault: {
                    VSTMint: this.knownAddress.jitoSOLTokenMint,
                    VRTMint: this.knownAddress.fragSOLJitoJitoSOLVRTMint,
                    vault: this.getConstantAsPublicKey("fragsolJitoJitosolVaultAccountAddress"),
                    operators: [],
                    program: this.getConstantAsPublicKey("jitoVaultProgramId"),
                    programFeeWalletTokenAccount: this.knownAddress.fragSOLJitoJitoSOLVaultProgramFeeWalletTokenAccount,
                    feeWalletTokenAccount: this.knownAddress.fragSOLJitoJitoSOLVaultFeeWalletTokenAccount,
                    vaultTokenAccount: this.knownAddress.fragSOLJitoJitoSOLVaultTokenAccount,
                    fundVRTAccount: this.knownAddress.fragSOLFundJitoJitoSOLVRTAccount,
                }
            };
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
            {
                pubkey: this.knownAddress.nSOLTokenPool,
                isSigner: false,
                isWritable: false,
            },
            ...Object.values(this.restakingVaultMetadata).map((v) => {
                return {
                    pubkey: v.vault,
                    isSigner: false,
                    isWritable: false,
                }
            }),
        ];
    }

    public async tryAirdropSupportedTokens(account: web3.PublicKey, lamports: BN = new BN(100 * web3.LAMPORTS_PER_SOL)) {
        await this.tryAirdrop(account, lamports.muln(Object.keys(this.supportedTokenMetadata).length));
        const txData = await Promise.all(
            Object.entries(this.supportedTokenMetadata).map(async ([_, token]) => {
                const ata = await spl.getOrCreateAssociatedTokenAccount(
                    this.connection,
                    this.wallet,
                    token.mint,
                    account,
                    false,
                    "confirmed",
                    {
                        skipPreflight: false,
                        commitment: "confirmed",
                    },
                    token.program
                );
                const splStakePoolAddress: web3.PublicKey | null = token.pricingSource["splStakePool"]?.address ?? null;
                if (splStakePoolAddress) {
                    const res = await splStakePool.depositSol(this.connection, splStakePoolAddress, this.wallet.publicKey, lamports.toNumber(), ata.address);
                    return {
                        instructions: res.instructions,
                        signers: res.signers,
                    };
                }

                const marinadeStakePoolAddress: web3.PublicKey | null = token.pricingSource["marinadeStakePool"]?.address ?? null;
                if (marinadeStakePoolAddress) {
                    const marinadeStakePool = new marinade.Marinade(
                        new marinade.MarinadeConfig({
                            connection: this.connection as unknown as web3.Connection,
                            publicKey: this.wallet.publicKey,
                        })
                    );
                    const res = await marinadeStakePool.deposit(lamports, {
                        mintToOwnerAddress: account,
                    });
                    return {
                        instructions: res.transaction.instructions,
                        signers: [],
                    };
                }

                return {instructions: [], signers: []};
            })
        );

        for (const {instructions, signers} of txData) {
            await this.run({instructions, signers});
        }

        for (const [symbol, token] of Object.entries(this.supportedTokenMetadata)) {
            const ata = await this.getUserSupportedTokenAccount(account, symbol as any);
            const balance = new BN(ata.amount.toString());
            logger.debug(`${symbol} airdropped (+${this.lamportsToSOL(lamports)}): ${this.lamportsToX(balance, token.decimals, symbol)}`.padEnd(LOG_PAD_LARGE), ata.address.toString());
        }
    }

    public getUserSupportedTokenAccount(user: web3.PublicKey, symbol: keyof typeof this.supportedTokenMetadata) {
        return spl.getAccount(
            this.connection,
            this.knownAddress.userSupportedTokenAccount(user, symbol),
            "confirmed",
            this.supportedTokenMetadata[symbol].program
        );
    }

    public getUserFragSOLAccount(user: web3.PublicKey) {
        return spl.getAccount(
            this.connection,
            this.knownAddress.fragSOLUserTokenAccount(user),
            "confirmed",
            spl.TOKEN_2022_PROGRAM_ID
        );
    }

    public async getOrCreateUserFragSOLAccount(user: web3.PublicKey) {
        return await spl.getOrCreateAssociatedTokenAccount(
            this.connection,
            this.wallet,
            this.knownAddress.fragSOLTokenMint,
            user,
            false,
            'confirmed',
            {
                commitment: 'confirmed',
            },
            spl.TOKEN_2022_PROGRAM_ID,
        )
    }

    public getUserWFragSOLAccount(user: web3.PublicKey) {
        return spl.getAccount(
            this.connection,
            this.knownAddress.wFragSOLUserTokenAccount(user),
            "confirmed",
        )
    }

    public getFragSOLFundSupportedTokenTreasuryAccountBalance(symbol: keyof typeof this.supportedTokenMetadata) {
        return spl.getAccount(
            this.connection,
            this.knownAddress.fragSOLFundSupportedTokenTreasuryAccount(symbol),
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

    public getFragSOLFundSupportedTokenReserveAccount(symbol: keyof typeof this.supportedTokenMetadata) {
        return spl.getAccount(
            this.connection,
            this.knownAddress.fragSOLFundSupportedTokenReserveAccount(symbol),
            "confirmed",
            this.supportedTokenMetadata[symbol].program
        );
    }

    public getFragSOLFundVRTAccount(symbol: keyof typeof this.restakingVaultMetadata) {
        let vault = this.restakingVaultMetadata[symbol];
        let account = spl.getAssociatedTokenAddressSync(
            vault.VRTMint,
            this.knownAddress.fragSOLFundReserveAccount,
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

    public getFragSOLSupportedTokenReserveAccountByMintAddress(mint: web3.PublicKey) {
        for (const [symbol, token] of Object.entries(this.supportedTokenMetadata)) {
            if (mint.toString() != token.mint.toString()) continue;
            return spl.getAccount(
                this.connection,
                this.knownAddress.fragSOLFundSupportedTokenReserveAccount(symbol as any),
                "confirmed",
                token.program,
            );
        }
        throw new Error("fund supported token account not found")
    }

    public getFragSOLFundReceiptTokenWrapAccount() {
        return spl.getAccount(
            this.connection,
            this.knownAddress.fragSOLFundReceiptTokenWrapAccount,
            "confirmed",
            spl.TOKEN_2022_PROGRAM_ID,
        )
    }

    public getFragSOLFundReceiptTokenLockAccount() {
        return spl.getAccount(
            this.connection,
            this.knownAddress.fragSOLFundReceiptTokenLockAccount,
            "confirmed",
            spl.TOKEN_2022_PROGRAM_ID
        );
    }

    public getUserFragSOLFundAccount(user: web3.PublicKey) {
        return this.account.userFundAccount.fetch(this.knownAddress.fragSOLUserFund(user));
    }

    public getUserFragSOLRewardAccount(user: web3.PublicKey) {
        return this.account.userRewardAccount.fetch(this.knownAddress.fragSOLUserReward(user));
    }

    public getFragSOLFundWrapAccountRewardAccount() {
        return this.getUserFragSOLRewardAccount(this.knownAddress.fragSOLFundWrapAccount);
    }

    public getFragSOLRewardAccount() {
        return this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward);
    }

    public getFragSOLFundAccount() {
        return this.account.fundAccount.fetch(this.knownAddress.fragSOLFund, "confirmed");
    }

    public getFragSOLFundReserveAccountBalance() {
        return this.connection.getBalance(this.knownAddress.fragSOLFundReserveAccount, "confirmed")
            .then(v => new BN(v));
    }

    public getFragSOLFundTreasuryAccountBalance() {
        return this.connection.getBalance(this.knownAddress.fragSOLFundTreasuryAccount, "confirmed")
            .then(v => new BN(v));
    }

    public getNSOLTokenPoolAccount() {
        return this.account.normalizedTokenPoolAccount.fetch(this.knownAddress.nSOLTokenPool, "confirmed");
    }

    public getNSOLSupportedTokenReserveAccountBalance(symbol: keyof typeof this.supportedTokenMetadata) {
        return this.connection.getTokenAccountBalance(this.knownAddress.nSOLSupportedTokenReserveAccount(symbol), "confirmed")
            .then(v => new BN(v.value.amount));
    }

    public getFragSOLFundNSOLAccountBalance() {
        return this.connection.getTokenAccountBalance(this.knownAddress.fragSOLFundNSOLAccount)
            .then(v => new BN(v.value.amount));
    }

    public getFragSOLJitoNSOLVaultTokenAccountBalance() {
        return this.connection.getTokenAccountBalance(this.knownAddress.fragSOLJitoNSOLVaultTokenAccount, "confirmed")
            .then(v => new BN(v.value.amount));
    }

    public getNSOLTokenMint() {
        return spl.getMint(
            this.connection,
            this.knownAddress.nSOLTokenMint,
            "confirmed",
            spl.TOKEN_PROGRAM_ID
        );
    }

    public getFragSOLTokenMint() {
        return spl.getMint(
            this.connection,
            this.knownAddress.fragSOLTokenMint,
            "confirmed",
            spl.TOKEN_2022_PROGRAM_ID
        );
    }

    public getSplStakePoolInfo(stakePoolAddress: web3.PublicKey) {
        return splStakePool.stakePoolInfo(
            this.connection,
            stakePoolAddress,
        );
    }

    private fragSOLImageURI = "https://fragmetric-assets.s3.ap-northeast-2.amazonaws.com/fragsol.png";
    private fragSOLMetadataURI = "https://quicknode.quicknode-ipfs.com/ipfs/QmcueajXkNzoYRhcCv323PMC8VVGiDvXaaVXkMyYcyUSRw";
    private fragSOLMetadata: splTokenMetadata.TokenMetadata = {
        mint: this.keychain.getPublicKey("FRAGSOL_MINT"),
        name: "Fragmetric Restaked SOL",
        symbol: "fragSOL",
        uri: this.fragSOLMetadataURI,
        additionalMetadata: [["description", `fragSOL is Solana's first native LRT that provides optimized restaking rewards.`]],
        updateAuthority: this.keychain.getPublicKey("ADMIN"),
    };

    public async runAdminInitializeFragSOLTokenMint() {
        const fileForMetadataURI = JSON.stringify(
            {
                name: this.fragSOLMetadata.name,
                symbol: this.fragSOLMetadata.symbol,
                description: this.fragSOLMetadata.additionalMetadata[0][1],
                image: this.fragSOLImageURI,
                // attributes: [],
            },
            null,
            0
        );
        logger.debug(`fragSOL metadata file:\n> ${this.fragSOLMetadata.uri}\n> ${fileForMetadataURI}`);

        const mintInitialSize = spl.getMintLen([spl.ExtensionType.TransferHook, spl.ExtensionType.MetadataPointer]);
        const mintMetadataExtensionSize = spl.TYPE_SIZE + spl.LENGTH_SIZE + splTokenMetadata.pack(this.fragSOLMetadata).length;
        const mintTotalSize = mintInitialSize + mintMetadataExtensionSize;
        const lamports = await this.connection.getMinimumBalanceForRentExemption(mintTotalSize);

        await this.run({
            instructions: [
                web3.SystemProgram.createAccount({
                    fromPubkey: this.wallet.publicKey,
                    newAccountPubkey: this.knownAddress.fragSOLTokenMint,
                    lamports: lamports,
                    space: mintInitialSize,
                    programId: spl.TOKEN_2022_PROGRAM_ID,
                }),
                spl.createInitializeTransferHookInstruction(this.knownAddress.fragSOLTokenMint, this.keychain.getPublicKey("ADMIN"), this.programId, spl.TOKEN_2022_PROGRAM_ID),
                spl.createInitializeMetadataPointerInstruction(this.knownAddress.fragSOLTokenMint, this.keychain.getPublicKey("ADMIN"), this.knownAddress.fragSOLTokenMint, spl.TOKEN_2022_PROGRAM_ID),
                spl.createInitializeMintInstruction(
                    this.knownAddress.fragSOLTokenMint,
                    this.fragSOLDecimals,
                    this.keychain.getPublicKey("ADMIN"),
                    null, // freeze authority to be null
                    spl.TOKEN_2022_PROGRAM_ID
                ),
                splTokenMetadata.createInitializeInstruction({
                    programId: spl.TOKEN_2022_PROGRAM_ID,
                    mint: this.knownAddress.fragSOLTokenMint,
                    metadata: this.knownAddress.fragSOLTokenMint,
                    name: this.fragSOLMetadata.name,
                    symbol: this.fragSOLMetadata.symbol,
                    uri: this.fragSOLMetadata.uri,
                    mintAuthority: this.keychain.getPublicKey("ADMIN"),
                    updateAuthority: this.fragSOLMetadata.updateAuthority,
                }),
                ...this.fragSOLMetadata.additionalMetadata.map(([field, value]) =>
                    splTokenMetadata.createUpdateFieldInstruction({
                        programId: spl.TOKEN_2022_PROGRAM_ID,
                        metadata: this.knownAddress.fragSOLTokenMint,
                        updateAuthority: this.fragSOLMetadata.updateAuthority,
                        field,
                        value,
                    })
                ),
            ],
            signerNames: ["ADMIN", "FRAGSOL_MINT"],
        });
        const fragSOLMint = await spl.getMint(
            this.connection,
            this.knownAddress.fragSOLTokenMint,
            "confirmed",
            spl.TOKEN_2022_PROGRAM_ID
        );
        logger.notice("fragSOL token mint created with extensions".padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLTokenMint.toString());
        return {fragSOLMint};
    }

    public async runAdminInitializeNSOLTokenMint(createMetadata = false) {
        const mintSize = spl.getMintLen([]);
        const lamports = await this.connection.getMinimumBalanceForRentExemption(mintSize);

        await this.run({
            instructions: [
                web3.SystemProgram.createAccount({
                    fromPubkey: this.wallet.publicKey,
                    newAccountPubkey: this.knownAddress.nSOLTokenMint,
                    lamports: lamports,
                    space: mintSize,
                    programId: spl.TOKEN_PROGRAM_ID,
                }),
                spl.createInitializeMintInstruction(
                    this.knownAddress.nSOLTokenMint,
                    this.nSOLDecimals,
                    this.keychain.getPublicKey("ADMIN"),
                    null, // freeze authority to be null
                    spl.TOKEN_PROGRAM_ID
                ),
            ],
            signerNames: ["FRAGSOL_NORMALIZED_TOKEN_MINT"],
        });

        if (this.isLocalnet) {
            const txSig = await this.connection.requestAirdrop(this.keychain.getKeypair("ADMIN").publicKey, 1_000_000_000)
            await this.connection.confirmTransaction(txSig, 'confirmed');
        }

        if (createMetadata) {
            const umiInstance = umi2.createUmi(this.connection.rpcEndpoint).use(mpl.mplTokenMetadata());
            const keypair = this.keychain.getKeypair('FRAGSOL_NORMALIZED_TOKEN_MINT');
            const umiKeypair = umiInstance.eddsa.createKeypairFromSecretKey(keypair.secretKey);
            const mint = umi.createSignerFromKeypair(umiInstance, umiKeypair);

            const authKeypair = umiInstance.eddsa.createKeypairFromSecretKey(this.keychain.getKeypair("ADMIN").secretKey);
            const authority = umi.createSignerFromKeypair(umiInstance, authKeypair);
            umiInstance.use(umi.signerIdentity(authority));

            await mpl.createV1(umiInstance, {
                mint,
                authority,
                name: "normalized Liquid Staked Solana",
                symbol: "nSOL",
                decimals: 9,
                uri: "https://quicknode.quicknode-ipfs.com/ipfs/QmR5pP6Zo65XWCEXgixY8UtZjWbYPKmYHcyxzUq4p1KZt5",
                sellerFeeBasisPoints: umi.percentAmount(0),
                tokenStandard: mpl.TokenStandard.Fungible,
            }).sendAndConfirm(umiInstance);

            const assets = await mpl.fetchAllDigitalAssetByUpdateAuthority(umiInstance, authority.publicKey);
            logger.notice("nSOL token mint metadata created".padEnd(LOG_PAD_LARGE), assets);
        }

        const nSOLMint = await spl.getMint(this.connection, this.knownAddress.nSOLTokenMint, "confirmed", spl.TOKEN_PROGRAM_ID);
        logger.notice("nSOL token mint created".padEnd(LOG_PAD_LARGE), this.knownAddress.nSOLTokenMint.toString());
        return {nSOLMint};
    }

    public async runAdminInitializeWFragSOLTokenMint(createMetadata = false) {
        const mintSize = spl.getMintLen([]);
        const lamports = await this.connection.getMinimumBalanceForRentExemption(mintSize);

        await this.run({
            instructions: [
                web3.SystemProgram.createAccount({
                    fromPubkey: this.wallet.publicKey,
                    newAccountPubkey: this.knownAddress.wFragSOLTokenMint,
                    lamports: lamports,
                    space: mintSize,
                    programId: spl.TOKEN_PROGRAM_ID,
                }),
                spl.createInitializeMintInstruction(
                    this.knownAddress.wFragSOLTokenMint,
                    this.wFragSOLDecimals,
                    this.keychain.getPublicKey("ADMIN"),
                    null, // freeze authority to be null
                    spl.TOKEN_PROGRAM_ID
                ),
            ],
            signerNames: ["FRAGSOL_WRAPPED_TOKEN_MINT"],
        });

        if (this.isLocalnet) {
            const txSig = await this.connection.requestAirdrop(this.keychain.getKeypair("ADMIN").publicKey, 1_000_000_000)
            await this.connection.confirmTransaction(txSig, 'confirmed');
        }

        if (createMetadata) {
            const umiInstance = umi2.createUmi(this.connection.rpcEndpoint).use(mpl.mplTokenMetadata());
            const keypair = this.keychain.getKeypair('FRAGSOL_WRAPPED_TOKEN_MINT');
            const umiKeypair = umiInstance.eddsa.createKeypairFromSecretKey(keypair.secretKey);
            const mint = umi.createSignerFromKeypair(umiInstance, umiKeypair);

            const authKeypair = umiInstance.eddsa.createKeypairFromSecretKey(this.keychain.getKeypair("ADMIN").secretKey);
            const authority = umi.createSignerFromKeypair(umiInstance, authKeypair);
            umiInstance.use(umi.signerIdentity(authority));

            // TODO
            await mpl.createV1(umiInstance, {
                mint,
                authority,
                name: "TODO",
                symbol: "TODO",
                decimals: this.wFragSOLDecimals,
                uri: "TODO",
                sellerFeeBasisPoints: umi.percentAmount(0),
                tokenStandard: mpl.TokenStandard.Fungible,
            }).sendAndConfirm(umiInstance);

            const assets = await mpl.fetchAllDigitalAssetByUpdateAuthority(umiInstance, authority.publicKey);
            logger.notice("wFragSOL token mint metadata created".padEnd(LOG_PAD_LARGE), assets);
        }

        const wFragSOLMint = await spl.getMint(this.connection, this.knownAddress.wFragSOLTokenMint, "confirmed", spl.TOKEN_PROGRAM_ID);
        logger.notice("wFragSOL token mint created".padEnd(LOG_PAD_LARGE), this.knownAddress.wFragSOLTokenMint.toString());
        return {wFragSOLMint};
    }

    public async runAdminUpdateTokenMetadata() {
        const fragSOLTokenMetadataAddress = this.knownAddress.fragSOLTokenMint;

        let tokenMetadata = await spl.getTokenMetadata(this.connection, fragSOLTokenMetadataAddress, undefined, spl.TOKEN_2022_PROGRAM_ID);
        logger.debug(`current token metadata:\n> ${JSON.stringify(tokenMetadata, null, 0)}`);

        const updatedFileForMetadataURI = JSON.stringify(
            {
                name: tokenMetadata.name,
                symbol: tokenMetadata.symbol,
                description: tokenMetadata.additionalMetadata[0][1],
                image: this.fragSOLImageURI,
                // attributes: [],
            },
            null,
            0
        );
        logger.debug(`fragSOL metadata file:\n> ${updatedFileForMetadataURI}`);

        const updatedMetadata = spl.updateTokenMetadata(tokenMetadata, splTokenMetadata.Field.Uri, this.fragSOLMetadataURI);
        logger.debug(`will update token metadata:\n> ${JSON.stringify(updatedMetadata, null, 0)}`);

        await this.run({
            instructions: [
                splTokenMetadata.createUpdateFieldInstruction({
                    programId: spl.TOKEN_2022_PROGRAM_ID,
                    metadata: this.knownAddress.fragSOLTokenMint,
                    updateAuthority: tokenMetadata.updateAuthority,
                    field: splTokenMetadata.Field.Uri,
                    value: this.fragSOLMetadataURI,
                }),
            ],
            signerNames: ["ADMIN"],
        });

        tokenMetadata = await spl.getTokenMetadata(this.connection, fragSOLTokenMetadataAddress, "confirmed", spl.TOKEN_2022_PROGRAM_ID);
        logger.notice(`updated token metadata:\n> ${JSON.stringify(tokenMetadata, null, 2)}`);
    }

    public async runAdminInitializeOrUpdateRewardAccount(batchSize = 35) {
        const currentVersion = await this.connection
            .getAccountInfo(this.knownAddress.fragSOLReward)
            .then((a) => a.data.readInt16LE(8))
            .catch((err) => 0);

        const targetVersion = parseInt(this.getConstant("rewardAccountCurrentVersion"));
        const instructions = [
            ...(currentVersion == 0 ? [this.program.methods.adminInitializeRewardAccount().accounts({
                payer: this.wallet.publicKey,
                receiptTokenMint: this.knownAddress.fragSOLTokenMint,
            }).instruction()] : []),
            ...new Array(targetVersion - currentVersion).fill(null).map((_, index, arr) => this.program.methods.adminUpdateRewardAccountIfNeeded(null).accounts({
                payer: this.wallet.publicKey,
                receiptTokenMint: this.knownAddress.fragSOLTokenMint,
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

        const fragSOLRewardAccount = await this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward, "confirmed");
        logger.notice(`updated reward account version from=${currentVersion}, to=${fragSOLRewardAccount.dataVersion}, target=${targetVersion}`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLReward.toString());

        return {fragSOLRewardAccount};
    }

    public async runAdminInitializeOrUpdateFundAccount(batchSize = 35) {
        const knownAddressLookupTableAddress = await this.getOrCreateKnownAddressLookupTable();

        const currentVersion = await this.connection
            .getAccountInfo(this.knownAddress.fragSOLFund)
            .then((a) => a.data.readInt16LE(8))
            .catch((err) => 0);

        const targetVersion = parseInt(this.getConstant("fundAccountCurrentVersion"));
        const instructions = [
            spl.createAssociatedTokenAccountIdempotentInstruction(
                this.wallet.publicKey,
                this.knownAddress.fragSOLFundReceiptTokenLockAccount,
                this.knownAddress.fragSOLFund,
                this.knownAddress.fragSOLTokenMint,
                spl.TOKEN_2022_PROGRAM_ID,
            ),
            ...(currentVersion == 0 ? [this.program.methods.adminInitializeFundAccount().accounts({
                payer: this.wallet.publicKey,
                receiptTokenMint: this.knownAddress.fragSOLTokenMint,
            }).instruction()] : []),
            ...new Array(targetVersion - currentVersion).fill(null).map((_, index, arr) => this.program.methods.adminUpdateFundAccountIfNeeded(null).accounts({
                payer: this.wallet.publicKey,
                receiptTokenMint: this.knownAddress.fragSOLTokenMint,
            }).instruction()),
            this.methods.adminSetAddressLookupTableAccount(knownAddressLookupTableAddress).accounts({
                payer: this.wallet.publicKey,
                receiptTokenMint: this.knownAddress.fragSOLTokenMint,
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

        const [fragSOLMint, fragSOLFundAccount] = await Promise.all([
            spl.getMint(this.connection, this.knownAddress.fragSOLTokenMint, "confirmed", spl.TOKEN_2022_PROGRAM_ID),
            this.account.fundAccount.fetch(this.knownAddress.fragSOLFund, "confirmed"),
        ]);
        logger.notice(`updated fund account version from=${currentVersion}, to=${fragSOLFundAccount.dataVersion}, target=${targetVersion}`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLFund.toString());

        return {fragSOLMint, fragSOLFundAccount};
    }

    public async runAdminInitializeOrUpdateFundWrapAccountRewardAccount() {
        const fragSOLFundWrapAccountAddress = this.knownAddress.fragSOLFundWrapAccount;
        const currentRewardVersion = await this.getFragSOLFundWrapAccountRewardAccount()
            .then(a => a.dataVersion)
            .catch(err => 0);
        
        const targetRewardVersion = parseInt(this.getConstant("userRewardAccountCurrentVersion"));

        await this.run({
            instructions: [
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.fragSOLFundReceiptTokenWrapAccount,
                    fragSOLFundWrapAccountAddress,
                    this.knownAddress.fragSOLTokenMint,
                    spl.TOKEN_2022_PROGRAM_ID,
                ),
                ...(currentRewardVersion == 0 ? [
                    this.program.methods.adminInitializeFundWrapAccountRewardAccount()
                        .accountsPartial({
                            payer: this.wallet.publicKey,
                            receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                        })
                        .instruction(),
                    ]
                    : [
                        ...new Array(targetRewardVersion - currentRewardVersion).fill(null).map((_, index, arr) =>
                            this.program.methods
                                .adminUpdateFundWrapAccountRewardAccountIfNeeded(null)
                                .accountsPartial({
                                    payer: this.wallet.publicKey,
                                    receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                                })
                                .instruction(),
                        )
                    ]),
            ],
            signerNames: ['ADMIN'],
        });

        const fragSOLFundWrapAccountRewardAccount = await this.getFragSOLFundWrapAccountRewardAccount();
        logger.notice(`created fund wrap account reward account`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLFundWrapAccountReward.toString());

        return {fragSOLFundWrapAccountRewardAccount};
    }

    public async runAdminInitializeNormalizedTokenPoolAccounts() {
        await this.run({
            instructions: [
                this.program.methods.adminInitializeNormalizedTokenPoolAccount()
                    .accounts({
                        payer: this.wallet.publicKey,
                        normalizedTokenMint: this.knownAddress.nSOLTokenMint,
                    })
                    .instruction(),
            ],
            signerNames: ["ADMIN"],
        });
        const nSOLTokenPoolAccount = await this.getNSOLTokenPoolAccount();
        logger.notice("nSOL token pool account created".padEnd(LOG_PAD_LARGE), this.knownAddress.nSOLTokenPool.toString());

        return {nSOLTokenPoolAccount};
    }

    public async runAdminUpdateNormalizedTokenPoolAccounts() {
        await this.run({
            instructions: [
                this.program.methods.adminUpdateNormalizedTokenPoolAccountIfNeeded()
                    .accountsPartial({
                        payer: this.wallet.publicKey,
                        normalizedTokenMint: this.knownAddress.nSOLTokenMint,
                    })
                    .instruction(),
            ],
            signerNames: ["ADMIN"],
        });
        const nSOLTokenPoolAccount = await this.getNSOLTokenPoolAccount();
        logger.notice("nSOL token pool account updated".padEnd(LOG_PAD_LARGE), this.knownAddress.nSOLTokenPool.toString());

        return {nSOLTokenPoolAccount};
    }

    public async runFundManagerInitializeFundNormalizedToken() {
        await this.run({
            instructions: [
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.fragSOLFundNSOLAccount,
                    this.knownAddress.fragSOLFundReserveAccount,
                    this.knownAddress.nSOLTokenMint,
                ),
                this.program.methods.fundManagerInitializeFundNormalizedToken()
                    .accountsPartial({
                        receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                        normalizedTokenMint: this.knownAddress.nSOLTokenMint,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signerNames: ["FUND_MANAGER"],
            events: ['fundManagerUpdatedFund'],
        });

        const fragSOLFundAccount = await this.getFragSOLFundAccount();
        logger.notice("set fragSOL fund normalized token".padEnd(LOG_PAD_LARGE), this.knownAddress.nSOLTokenMint.toString());

        return {fragSOLFundAccount};
    }

    public async runFundManagerInitializeFundWrappedToken() {
        await this.run({
            instructions: [
                this.program.methods.fundManagerInitializeFundWrappedToken()
                    .accountsPartial({
                        receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                        wrappedTokenMint: this.knownAddress.wFragSOLTokenMint,
                    })
                    .instruction(),
            ],
            signerNames: ['ADMIN', 'FUND_MANAGER'],
            events: ['fundManagerUpdatedFund']
        });

        const [wFragSOLMint, fragSOLFundAccount] = await Promise.all([
            spl.getMint(this.connection, this.knownAddress.wFragSOLTokenMint, "confirmed"),
            this.getFragSOLFundAccount(),
        ]);
        logger.notice('set fragSOL fund wrapped token'.padEnd(LOG_PAD_LARGE), this.knownAddress.wFragSOLTokenMint.toString());

        return {wFragSOLMint, fragSOLFundAccount};
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
                        this.knownAddress.fragSOLFundReserveAccount,
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
                            receiptTokenMint: this.knownAddress.fragSOLTokenMint,
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

        logger.notice(`initialized fragSOL fund jito restaking vaults`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLFund.toString());
        const fragSOLFund = await this.account.fundAccount.fetch(this.knownAddress.fragSOLFund, 'confirmed');
        return {event, error, fragSOLFund};
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
                    this.knownAddress.fragSOLFundReserveAccount,
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
                        receiptTokenMint: this.knownAddress.fragSOLTokenMint,
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
            fragSOLFundJitoFeeWalletTokenAccount,
            fragSOLJitoVaultTokenAccount,
            fragSOLFundJitoVRTAccount,
            fragSOLJitoVaultProgramFeeWalletTokenAccount,
            fragSOLFundAccount,
        ] = await Promise.all([
            spl.getAccount(this.connection, vault.feeWalletTokenAccount, 'confirmed'),
            spl.getAccount(this.connection, vault.vaultTokenAccount, 'confirmed'),
            spl.getAccount(this.connection, vault.fundVRTAccount, 'confirmed'),
            spl.getAccount(this.connection, vault.programFeeWalletTokenAccount, 'confirmed'),
            this.getFragSOLFundAccount(),
        ]);
        logger.notice("jito VRT fee account created".padEnd(LOG_PAD_LARGE), vault.feeWalletTokenAccount.toString());
        logger.notice("jito vault VST account created".padEnd(LOG_PAD_LARGE), vault.vaultTokenAccount.toString());
        logger.notice("jito fund VRT account created".padEnd(LOG_PAD_LARGE), vault.fundVRTAccount.toString());
        logger.notice("jito VRT account (of program fee wallet) created".padEnd(LOG_PAD_LARGE), vault.programFeeWalletTokenAccount.toString());

        return {fragSOLFundJitoVRTAccount, fragSOLJitoVaultTokenAccount, fragSOLFundJitoFeeWalletTokenAccount, fragSOLJitoVaultProgramFeeWalletTokenAccount, fragSOLFundAccount};
    }

    // for test - create jito vault
    public async runAdminCreateJitoVault(vstMint: web3.PublicKey, depositFeeBps = 0, withdrawalFeeBps = 0, rewardFeeBps = 0, vstDecimals = 9, authority = this.keychain.getKeypair("ADMIN")) {
        const vrtMint = web3.Keypair.generate();
        const InitializeVaultInstructionDataSize = {
            discriminator: 1, // u8
            depositFeeBps: 2, // u16
            withdrawalFeeBps: 2, // u16
            rewardFeeBps: 2, /// u16
            decimals: 1, // u8
        };

        const base = web3.Keypair.generate();
        const vaultPublicKey = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("vault"), base.publicKey.toBuffer()],
            this.knownAddress.jitoVaultProgram,
        );

        const discriminator = 1;
        const data = Buffer.alloc(
            InitializeVaultInstructionDataSize.discriminator +
            InitializeVaultInstructionDataSize.depositFeeBps +
            InitializeVaultInstructionDataSize.withdrawalFeeBps +
            InitializeVaultInstructionDataSize.rewardFeeBps +
            InitializeVaultInstructionDataSize.decimals
        );
        logger.notice("vst mint".padEnd(LOG_PAD_LARGE), vstMint.toString());
        logger.notice("vault account".padEnd(LOG_PAD_LARGE), vaultPublicKey[0].toString());
        logger.notice("vrt mint".padEnd(LOG_PAD_LARGE), vrtMint.publicKey.toString());

        let offset = 0;
        data.writeUInt8(discriminator, offset);
        data.writeUInt16LE(depositFeeBps, offset += InitializeVaultInstructionDataSize.discriminator);
        data.writeUInt16LE(withdrawalFeeBps, offset += InitializeVaultInstructionDataSize.depositFeeBps);
        data.writeUint16LE(rewardFeeBps, offset += InitializeVaultInstructionDataSize.withdrawalFeeBps);
        data.writeUInt8(vstDecimals, offset += InitializeVaultInstructionDataSize.rewardFeeBps);

        const ix = new web3.TransactionInstruction(
            {
                programId: this.knownAddress.jitoVaultProgram,
                keys: [
                    {
                        pubkey: this.knownAddress.jitoVaultConfig,
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: vaultPublicKey[0],
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: vrtMint.publicKey,
                        isSigner: true,
                        isWritable: true,
                    },
                    {
                        pubkey: vstMint,
                        isSigner: false,
                        isWritable: false,
                    },
                    {
                        pubkey: authority.publicKey,
                        isSigner: true,
                        isWritable: true,
                    },
                    {
                        pubkey: base.publicKey,
                        isSigner: true,
                        isWritable: false,
                    },
                    {
                        pubkey: web3.SystemProgram.programId,
                        isSigner: false,
                        isWritable: false,
                    },
                    {
                        pubkey: spl.TOKEN_PROGRAM_ID,
                        isSigner: false,
                        isWritable: false,
                    },
                ],
                data,
            }
        );

        await this.run({
            instructions: [ix],
            signers: [authority, vrtMint, base],
        });
    }

    public async runFundManagerAddFundJitoRestakingVaultDelegation(vault: web3.PublicKey, operator: web3.PublicKey) {
        await this.run({
            instructions: [
                this.methods.fundManagerInitializeFundJitoRestakingVaultDelegation()
                    .accountsPartial({
                        receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                        vaultAccount: vault,
                        vaultOperator: operator,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signerNames: ["FUND_MANAGER"],
            events: ['fundManagerUpdatedFund'],
        });

        logger.notice("jito vault operator initialized".padEnd(LOG_PAD_LARGE), operator.toString());
    }

    // for test - create and initialize operator
    public async runAdminInitializeJitoRestakingOperator(operatorFeeBps = 0, authority = this.keychain.getKeypair("ADMIN")) {
        const InitializeOperatorInstructionDataSize = {
            discriminator: 1, // u8
            operatorFeeBps: 2, // u16
        };

        const base = web3.Keypair.generate();
        const operatorPublicKey = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("operator"), base.publicKey.toBuffer()],
            this.knownAddress.jitoRestakingProgram,
        );
        logger.notice(`operator key`.padEnd(LOG_PAD_LARGE), operatorPublicKey.toString());

        const discriminator = 2;
        const data = Buffer.alloc(
            InitializeOperatorInstructionDataSize.discriminator +
            InitializeOperatorInstructionDataSize.operatorFeeBps
        );

        let offset = 0;
        data.writeUInt8(discriminator, offset);
        data.writeUInt16LE(operatorFeeBps, offset += InitializeOperatorInstructionDataSize.discriminator);

        const ix = new web3.TransactionInstruction(
            {
                programId: this.knownAddress.jitoRestakingProgram,
                keys: [
                    {
                        pubkey: this.knownAddress.jitoRestakingConfig,
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: operatorPublicKey[0],
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: authority.publicKey,
                        isSigner: true,
                        isWritable: true,
                    },
                    {
                        pubkey: base.publicKey,
                        isSigner: true,
                        isWritable: false,
                    },
                    {
                        pubkey: web3.SystemProgram.programId,
                        isSigner: false,
                        isWritable: false,
                    },
                ],
                data,
            }
        );

        await this.run({
            instructions: [ix],
            signers: [authority, base],
        });

        return { operator: operatorPublicKey };
    }

    // for test - create and initialize ncn
    public async runAdminJitoInitializeNcn(authority = this.keychain.getKeypair("ADMIN")) {
        const InitializeNcnInstructionDataSize = {
            discriminator: 1, // u8
        };

        const base = web3.Keypair.generate();
        const ncnPublicKey = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("ncn"), base.publicKey.toBuffer()],
            this.knownAddress.jitoRestakingProgram,
        );
        logger.notice("ncn key".padEnd(LOG_PAD_LARGE), ncnPublicKey[0].toString());

        const discriminator = 1;
        const data = Buffer.alloc(
            InitializeNcnInstructionDataSize.discriminator
        );

        let offset = 0;
        data.writeUInt8(discriminator, offset);

        const ix = new web3.TransactionInstruction(
            {
                programId: this.knownAddress.jitoRestakingProgram,
                keys: [
                    {
                        pubkey: this.knownAddress.jitoRestakingConfig,
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: ncnPublicKey[0],
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: authority.publicKey,
                        isSigner: true,
                        isWritable: true,
                    },
                    {
                        pubkey: base.publicKey,
                        isSigner: true,
                        isWritable: false,
                    },
                    {
                        pubkey: web3.SystemProgram.programId,
                        isSigner: false,
                        isWritable: false,
                    },
                ],
                data,
            }
        );

        await this.run({
            instructions: [ix],
            signers: [authority, base],
        });
    }

    // for test - delegate jito vault token account
    public async runAdminJitoVaultDelegateJitoSOLTokenAccount() {
        const DelegateTokenAccountInstructionDataSize = {
            discriminator: 1, // u8
        };

        const discriminator = 20;
        const data = Buffer.alloc(
            DelegateTokenAccountInstructionDataSize.discriminator
        );

        let offset = 0;
        data.writeUInt8(discriminator, offset);

        ///   0. `[]` config
        ///   1. `[]` vault
        ///   2. `[signer]` delegate_asset_admin
        ///   3. `[]` token_mint
        ///   4. `[writable]` token_account
        ///   5. `[]` delegate
        const admin = this.keychain.getKeypair("ADMIN");
        const ix = new web3.TransactionInstruction(
            {
                programId: this.knownAddress.jitoVaultProgram,
                keys: [
                    {
                        pubkey: this.knownAddress.jitoVaultConfig,
                        isSigner: false,
                        isWritable: false,
                    },
                    {
                        pubkey: this.restakingVaultMetadata.jitoNSOLVault.vault,
                        isSigner: false,
                        isWritable: false,
                    },
                    {
                        pubkey: admin.publicKey,
                        isSigner: true,
                        isWritable: false,
                    },
                    {
                        pubkey: this.supportedTokenMetadata.jitoSOL.mint,
                        isSigner: false,
                        isWritable: false,
                    },
                    {
                        pubkey: spl.getAssociatedTokenAddressSync(
                            this.supportedTokenMetadata.jitoSOL.mint,
                            this.restakingVaultMetadata.jitoNSOLVault.vault,
                            true,
                            this.supportedTokenMetadata.jitoSOL.program,
                        ),
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: admin.publicKey,
                        isSigner: false,
                        isWritable: false,
                    },
                    {
                        pubkey: this.supportedTokenMetadata.jitoSOL.program,
                        isSigner: false,
                        isWritable: false,
                    },
                ],
                data,
            }
        );

        await this.run({
            instructions: [ix],
            signers: [admin, this.wallet],
        });
    }

    // for test - initialize operator_vault_ticket
    public async runAdminInitializeOperatorVaultTicket(vault: web3.PublicKey, operator: web3.PublicKey, authority = this.keychain.getKeypair("ADMIN")) {
        const InitializeOperatorVaultTicketInstructionDataSize = {
            discriminator: 1, // u8
        };

        const operatorVaultTicketPublicKey = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("operator_vault_ticket"), operator.toBuffer(), vault.toBuffer()],
            this.knownAddress.jitoRestakingProgram,
        );
        logger.notice(`operator_vault_ticket key`.padEnd(LOG_PAD_LARGE), operatorVaultTicketPublicKey.toString());

        const discriminator = 5;
        const data = Buffer.alloc(
            InitializeOperatorVaultTicketInstructionDataSize.discriminator
        );

        let offset = 0;
        data.writeUInt8(discriminator, offset);

        const ix = new web3.TransactionInstruction(
            {
                programId: this.knownAddress.jitoRestakingProgram,
                keys: [
                    {
                        pubkey: this.knownAddress.jitoRestakingConfig,
                        isSigner: false,
                        isWritable: false,
                    },
                    {
                        pubkey: operator,
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: vault,
                        isSigner: false,
                        isWritable: false,
                    },
                    {
                        pubkey: operatorVaultTicketPublicKey[0],
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: authority.publicKey,
                        isSigner: true,
                        isWritable: false,
                    },
                    {
                        pubkey: this.wallet.publicKey,
                        isSigner: true,
                        isWritable: true,
                    },
                    {
                        pubkey: web3.SystemProgram.programId,
                        isSigner: false,
                        isWritable: false,
                    },
                ],
                data,
            }
        );

        await this.run({
            instructions: [ix],
            signers: [authority, this.wallet],
        });

        return { operatorVaultTicket: operatorVaultTicketPublicKey };
    }

    // need for operation - initialize vault_operator_delegation
    public async runAdminInitializeVaultOperatorDelegation(vault: web3.PublicKey, operator: web3.PublicKey, operatorVaultTicket: web3.PublicKey, authority = this.keychain.getKeypair("ADMIN")) {
        const InitializeVaultOperatorDelegationInstructionDataSize = {
            discriminator: 1, // u8
        };

        const vaultOperatorDelegationPublicKey = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("vault_operator_delegation"), vault.toBuffer(), operator.toBuffer()],
            this.knownAddress.jitoVaultProgram,
        );
        logger.notice(`vault_operator_delegation key`.padEnd(LOG_PAD_LARGE), vaultOperatorDelegationPublicKey.toString());

        const discriminator = 3;
        const data = Buffer.alloc(
            InitializeVaultOperatorDelegationInstructionDataSize.discriminator
        );

        let offset = 0;
        data.writeUInt8(discriminator, offset);

        const ix = new web3.TransactionInstruction(
            {
                programId: this.knownAddress.jitoVaultProgram,
                keys: [
                    {
                        pubkey: this.knownAddress.jitoVaultConfig,
                        isSigner: false,
                        isWritable: false,
                    },
                    {
                        pubkey: vault,
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: operator,
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: operatorVaultTicket,
                        isSigner: false,
                        isWritable: false,
                    },
                    {
                        pubkey: vaultOperatorDelegationPublicKey[0],
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: authority.publicKey,
                        isSigner: true,
                        isWritable: false,
                    },
                    {
                        pubkey: this.wallet.publicKey,
                        isSigner: true,
                        isWritable: true,
                    },
                    {
                        pubkey: web3.SystemProgram.programId,
                        isSigner: false,
                        isWritable: false,
                    },
                ],
                data,
            }
        );

        await this.run({
            instructions: [ix],
            signers: [authority, this.wallet],
        });

        return { vaultOperatorDelegation: vaultOperatorDelegationPublicKey };
    }

    // need for operation - set vault_delegation_admin to fund_account
    public async runAdminSetSecondaryAdminForJitoVault(vault: web3.PublicKey, oldAuthority = this.keychain.getKeypair("ADMIN")) {
        const newAuthority = this.knownAddress.fragSOLFund;
        logger.notice("old authority".padEnd(LOG_PAD_LARGE), oldAuthority.publicKey.toString());
        logger.notice("new authority".padEnd(LOG_PAD_LARGE), newAuthority.toString());

        const SetSecondaryAdminInstructionDataSize = {
            discriminator: 1, // u8
            vaultAdminRole: 1, // enum VaultAdminRole
        };

        const discriminator = 22;
        const vaultDelegationAdminRole = 0; // enum 0
        const data = Buffer.alloc(
            SetSecondaryAdminInstructionDataSize.discriminator +
            SetSecondaryAdminInstructionDataSize.vaultAdminRole
        );

        let offset = 0;
        data.writeUInt8(discriminator, offset);
        data.writeUInt8(vaultDelegationAdminRole, offset += SetSecondaryAdminInstructionDataSize.discriminator);

        const ix = new web3.TransactionInstruction(
            {
                programId: this.knownAddress.jitoVaultProgram,
                keys: [
                    {
                        pubkey: this.knownAddress.jitoVaultConfig,
                        isSigner: false,
                        isWritable: false,
                    },
                    {
                        pubkey: vault,
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: oldAuthority.publicKey,
                        isSigner: true,
                        isWritable: false,
                    },
                    {
                        pubkey: newAuthority,
                        isSigner: false,
                        isWritable: false,
                    },
                ],
                data,
            }
        );

        await this.run({
            instructions: [ix],
            signers: [oldAuthority],
        });
    }

    // for test - initialize ncn operator state
    public async runAdminJitoInitializeNcnOperatorState(ncn: web3.PublicKey, operator: web3.PublicKey, authority = this.keychain.getKeypair("ADMIN")) {
        const InitializeNcnOperatorStateInstructionDataSize = {
            discriminator: 1, // u8
        };

        const ncnOperatorStatePublicKey = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("ncn_operator_state"), ncn.toBuffer(), operator.toBuffer()],
            this.knownAddress.jitoRestakingProgram,
        );
        logger.notice("ncn_operator_state key".padEnd(LOG_PAD_LARGE), ncnOperatorStatePublicKey[0].toString());

        const discriminator = 6;
        const data = Buffer.alloc(
            InitializeNcnOperatorStateInstructionDataSize.discriminator
        );

        const offset = 0;
        data.writeUInt8(discriminator, offset);

        const ix = new web3.TransactionInstruction(
            {
                programId: this.knownAddress.jitoRestakingProgram,
                keys: [
                    {
                        pubkey: this.knownAddress.jitoRestakingConfig,
                        isSigner: false,
                        isWritable: false,
                    },
                    {
                        pubkey: ncn,
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: operator,
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: ncnOperatorStatePublicKey[0],
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: authority.publicKey,
                        isSigner: true,
                        isWritable: false,
                    },
                    {
                        pubkey: this.wallet.publicKey,
                        isSigner: true,
                        isWritable: true,
                    },
                    {
                        pubkey: web3.SystemProgram.programId,
                        isSigner: false,
                        isWritable: false,
                    },
                ],
                data,
            }
        );

        await this.run({
            instructions: [ix],
            signers: [authority, this.wallet],
        });

        return { ncnOperatorState: ncnOperatorStatePublicKey[0] };
    }

    // for test - initialize ncn_vault_ticket
    public async runAdminJitoInitializeNcnVaultTicket(ncn: web3.PublicKey, vault: web3.PublicKey, authority: web3.Keypair = this.keychain.getKeypair("ADMIN")) {
        const InitializeNcnVaultTicketInstructionDataSize = {
            discriminator: 1, // u8
        };

        const ncnVaultTicketPublicKey = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("ncn_vault_ticket"), ncn.toBuffer(), vault.toBuffer()],
            this.knownAddress.jitoRestakingProgram,
        );
        logger.notice("ncn_vault_ticket key".padEnd(LOG_PAD_LARGE), ncnVaultTicketPublicKey[0].toString());

        const discriminator = 4;
        const data = Buffer.alloc(
            InitializeNcnVaultTicketInstructionDataSize.discriminator
        );

        let offset = 0;
        data.writeUInt8(discriminator, offset);

        const ix = new web3.TransactionInstruction(
            {
                programId: this.knownAddress.jitoRestakingProgram,
                keys: [
                    {
                        pubkey: this.knownAddress.jitoRestakingConfig,
                        isSigner: false,
                        isWritable: false,
                    },
                    {
                        pubkey: ncn,
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: vault,
                        isSigner: false,
                        isWritable: false,
                    },
                    {
                        pubkey: ncnVaultTicketPublicKey[0],
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: authority.publicKey,
                        isSigner: true,
                        isWritable: false,
                    },
                    {
                        pubkey: this.wallet.publicKey,
                        isSigner: true,
                        isWritable: true,
                    },
                    {
                        pubkey: web3.SystemProgram.programId,
                        isSigner: false,
                        isWritable: false,
                    },
                ],
                data,
            }
        );

        await this.run({
            instructions: [ix],
            signers: [authority, this.wallet],
        });

        return { ncnVaultTicket: ncnVaultTicketPublicKey[0] };
    }

    // need for operation - initialize vault_ncn_ticket
    public async runAdminJitoInitializeVaultNcnTicket(vault: web3.PublicKey, ncn: web3.PublicKey, ncnVaultTicket: web3.PublicKey, authority: web3.Keypair = this.keychain.getKeypair("ADMIN")) {
        const InitializeVaultNcnTicketInstructionDataSize = {
            discriminator: 1, // u8
        };

        const vaultNcnTicketPublicKey = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("vault_ncn_ticket"), vault.toBuffer(), ncn.toBuffer()],
            this.knownAddress.jitoVaultProgram,
        );
        logger.notice("vault_ncn_ticket key".padEnd(LOG_PAD_LARGE), vaultNcnTicketPublicKey[0].toString());

        const discriminator = 4;
        const data = Buffer.alloc(
            InitializeVaultNcnTicketInstructionDataSize.discriminator
        );

        let offset = 0;
        data.writeUInt8(discriminator, offset);

        const ix = new web3.TransactionInstruction(
            {
                programId: this.knownAddress.jitoVaultProgram,
                keys: [
                    {
                        pubkey: this.knownAddress.jitoVaultConfig,
                        isSigner: false,
                        isWritable: false,
                    },
                    {
                        pubkey: vault,
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: ncn,
                        isSigner: false,
                        isWritable: false,
                    },
                    {
                        pubkey: ncnVaultTicket,
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: vaultNcnTicketPublicKey[0],
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: authority.publicKey,
                        isSigner: true,
                        isWritable: false,
                    },
                    {
                        pubkey: this.wallet.publicKey,
                        isSigner: true,
                        isWritable: true,
                    },
                    {
                        pubkey: web3.SystemProgram.programId,
                        isSigner: false,
                        isWritable: false,
                    },
                ],
                data,
            }
        );

        await this.run({
            instructions: [ix],
            signers: [authority, this.wallet],
        });

        return { vaultNcnTicket: vaultNcnTicketPublicKey[0] };
    }

    public async runAdminInitializeFragSOLExtraAccountMetaList() {
        await this.run({
            instructions: [
                this.program.methods.adminInitializeExtraAccountMetaList().accounts({
                    payer: this.wallet.publicKey,
                    receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                }).instruction(),
            ],
            signerNames: ["ADMIN"],
        });
        const fragSOLExtraAccountMetasAccount = await this.connection.getAccountInfo(spl.getExtraAccountMetaAddress(this.knownAddress.fragSOLTokenMint, this.programId)).then((acc) => spl.getExtraAccountMetas(acc));
        logger.notice(`initialized fragSOL extra account meta list`.padEnd(LOG_PAD_LARGE));

        return {fragSOLExtraAccountMetasAccount};
    }

    public async runAdminUpdateFragSOLExtraAccountMetaList() {
        await this.run({
            instructions: [
                this.program.methods.adminUpdateExtraAccountMetaListIfNeeded().accounts({
                    payer: this.wallet.publicKey,
                    receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                }).instruction(),
            ],
            signerNames: ["ADMIN"],
        });
        const fragSOLExtraAccountMetasAccount = await this.connection.getAccountInfo(spl.getExtraAccountMetaAddress(this.knownAddress.fragSOLTokenMint, this.programId)).then((acc) => spl.getExtraAccountMetas(acc));
        logger.notice(`updated fragSOL extra account meta list`.padEnd(LOG_PAD_LARGE));

        return {fragSOLExtraAccountMetasAccount};
    }

    public async runFundManagerInitializeFundSupportedTokens() {
        const {event, error} = await this.run({
            instructions: Object.entries(this.supportedTokenMetadata).flatMap(([symbol, v]) => {
                return [
                    spl.createAssociatedTokenAccountIdempotentInstruction(
                        this.wallet.publicKey,
                        this.knownAddress.fragSOLFundSupportedTokenReserveAccount(symbol as any),
                        this.knownAddress.fragSOLFundReserveAccount,
                        v.mint,
                        v.program,
                    ),
                    spl.createAssociatedTokenAccountIdempotentInstruction(
                        this.wallet.publicKey,
                        this.knownAddress.fragSOLFundSupportedTokenTreasuryAccount(symbol as any),
                        this.knownAddress.fragSOLFundTreasuryAccount,
                        v.mint,
                        v.program,
                    ),
                    this.program.methods
                        .fundManagerAddSupportedToken(v.pricingSource)
                        .accountsPartial({
                            receiptTokenMint: this.knownAddress.fragSOLTokenMint,
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

        logger.notice(`initialized fragSOL fund configuration`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLFund.toString());
        const fragSOLFund = await this.account.fundAccount.fetch(this.knownAddress.fragSOLFund, 'confirmed');
        return {event, error, fragSOLFund};
    }

    public get targetFragSOLFundConfiguration() {

        return {
            depositEnabled: this.isDevnet ? true : (this.isMainnet ? true : true),
            donationEnabled: this.isDevnet ? true : (this.isMainnet ? false : true),
            withdrawalEnabled: this.isDevnet ? true : (this.isMainnet ? true : true),
            transferEnabled: this.isDevnet ? true : (this.isMainnet ? false : false),
            WithdrawalFeedRateBPS: this.isDevnet ? 20 : 20,
            withdrawalBatchThresholdSeconds: new BN(this.isDevnet ? 60 : (this.isMainnet ? 86400 : 10)), // seconds

            solDepositable: true,
            solAccumulatedDepositCapacity: this.isDevnet
                ? new BN(1_000_000_000).mul(new BN(web3.LAMPORTS_PER_SOL)) : (
                    this.isMainnet ? new BN(185_844_305_400_574) : new BN(1_000_000_000).mul(new BN(web3.LAMPORTS_PER_SOL))
                ),
            solAccumulatedDepositAmount: null,
            solWithdrawalable: true,
            solWithdrawalNormalReserveRateBPS: 0,
            solWithdrawalNormalReserveMaxAmount: new BN(MAX_CAPACITY),

            supportedTokens: Object.entries(this.supportedTokenMetadata).map(([symbol, v]) => ({
                tokenMint: v.mint,
                tokenDepositable: (() => {
                    switch (symbol) {
                        case "bSOL":
                            return this.isDevnet ? true : (this.isMainnet ? false : true);
                        case "jitoSOL":
                            return true;
                        case "mSOL":
                            return true;
                        case "BNSOL":
                            return true;
                        case "bbSOL":
                            return this.isDevnet ? true : (this.isMainnet ? false : true);
                        default:
                            throw `invalid accumulated deposit cap for ${symbol}`;
                    }
                })(),
                tokenAccumulatedDepositCapacity: (() => {
                    switch (symbol) {
                        case "bSOL":
                            return new BN(this.isDevnet ? 90_000_000_000 : (this.isMainnet ? 0 : 90_000_000_000));
                        case "jitoSOL":
                            return new BN(this.isDevnet ? 80_000_000_000 : (this.isMainnet ? 90_941_492_854_023 : 80_000_000_000));
                        case "mSOL":
                            return new BN(this.isDevnet ? 70_000_000_000 : (this.isMainnet ? 4_500_002_000_000 : 70_000_000_000));
                        case "BNSOL":
                            return new BN(this.isDevnet ? 60_000_000_000 : (this.isMainnet ? 11_311_923_730_911 : 60_000_000_000));
                        case "bbSOL":
                            return new BN(this.isDevnet ? 60_000_000_000 : (this.isMainnet ? 0 : 60_000_000_000));
                        default:
                            throw `invalid accumulated deposit cap for ${symbol}`;
                    }
                })(),
                tokenAccumulatedDepositAmount : null,
                withdrawable: false,
                withdrawalNormalReserveRateBPS: 0,
                withdrawalNormalReserveMaxAmount: new BN(MAX_CAPACITY),
                tokenRebalancingAmount: null,
                solAllocationWeight: (() => {
                    switch (symbol) {
                        case "bSOL":
                            return new BN(0);
                        case "jitoSOL":
                            return new BN(1);
                        case "mSOL":
                            return new BN(0);
                        case "BNSOL":
                            return new BN(0);
                        case "bbSOL":
                            return new BN(0);
                        default:
                            throw `invalid sol allocation weight for ${symbol}`;
                    }
                })(),
                solAllocationCapacityAmount: (() => {
                    switch (symbol) {
                        case "bSOL":
                            return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                        case "jitoSOL":
                            return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                        case "mSOL":
                            return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                        case "BNSOL":
                            return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                        case "bbSOL":
                            return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                        default:
                            throw `invalid sol allocation cap for ${symbol}`;
                    }
                })(),
            })),
            restakingVaults: Object.entries(this.restakingVaultMetadata).map(([symbol, v]) => ({
                vault: v.vault,
                solAllocationWeight: (() => {
                    switch (symbol) {
                        case "jitoNSOLVault":
                            return new BN(this.isDevnet ? 0 : 1);
                        case "jitoJitoSOLVault":
                            return new BN(this.isDevnet ? 1 : 2);
                        default:
                            throw `invalid sol allocation weight for ${symbol}`;
                    }
                })(),
                solAllocationCapacityAmount: (() => {
                    switch (symbol) {
                        case "jitoNSOLVault":
                            return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                        case "jitoJitoSOLVault":
                            return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                        default:
                            throw `invalid sol allocation cap for ${symbol}`;
                    }
                })(),
                delegations: v.operators.map((operator, index) => ({
                    operator,
                    supportedTokenAllocationWeight: (() => {
                        switch (symbol) {
                            case "jitoNSOLVault":
                                if (index == 0) {
                                    return new BN(this.isDevnet ? 0 : 1);
                                } else if (index == 1) {
                                    return new BN(this.isDevnet ? 0 : 2);
                                }
                            case "jitoJitoSOLVault":
                                if (index == 0) {
                                    return new BN(this.isDevnet ? 0 : 1);
                                } else if (index == 1) {
                                    return new BN(this.isDevnet ? 0 : 2);
                                }
                            default:
                                throw `invalid supported token allocation weight for ${symbol}`;
                        }
                    })(),
                    supportedTokenAllocationCapacityAmount: (() => {
                        switch (symbol) {
                            case "jitoNSOLVault":
                                if (index == 0) {
                                    return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                                } else if (index == 1) {
                                    return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                                }
                            case "jitoJitoSOLVault":
                                if (index == 0) {
                                    return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                                } else if (index == 1) {
                                    return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                                }
                            default:
                                throw `invalid supported token allocation capacity amount for ${symbol}`;
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
                    this.knownAddress.fragSOLFundSupportedTokenReserveAccount(symbol as any),
                    this.knownAddress.fragSOLFundReserveAccount,
                    token.mint,
                    token.program,
                ),
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.fragSOLFundSupportedTokenTreasuryAccount(symbol as any),
                    this.knownAddress.fragSOLFundTreasuryAccount,
                    token.mint,
                    token.program,
                ),
                this.methods
                    .fundManagerAddSupportedToken(token.pricingSource)
                    .accountsPartial({
                        receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                        supportedTokenMint: token.mint,
                        supportedTokenProgram: token.program,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signerNames: ["FUND_MANAGER"],
            events: ["fundManagerUpdatedFund"],
        });

        logger.notice(`added fragSOL fund supported token`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLFund.toString());
        const fragSOLFund = await this.account.fundAccount.fetch(this.knownAddress.fragSOLFund, 'confirmed');
        return {event, error, fragSOLFund};
    }

    // update capacity and configurations
    public async runFundManagerUpdateFundConfigurations() {
        const config = this.targetFragSOLFundConfiguration;
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
                    receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                }).instruction(),
                this.program.methods.fundManagerUpdateSolStrategy(
                    config.solDepositable,
                    config.solAccumulatedDepositCapacity,
                    config.solAccumulatedDepositAmount,
                    config.solWithdrawalable,
                    config.solWithdrawalNormalReserveRateBPS,
                    config.solWithdrawalNormalReserveMaxAmount,
                ).accountsPartial({
                    receiptTokenMint: this.knownAddress.fragSOLTokenMint,
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
                            receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                        })
                        .remainingAccounts(this.pricingSourceAccounts)
                        .instruction(),
                    ];
                }),
                ...config.restakingVaults.flatMap((v) => {
                    return [
                        this.program.methods.fundManagerUpdateRestakingVaultStrategy(
                            v.vault,
                            v.solAllocationWeight,
                            v.solAllocationCapacityAmount,
                        ).accountsPartial({
                            receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                        })
                        .remainingAccounts(this.pricingSourceAccounts)
                        .instruction(),
                        // vault's delegations
                        ...v.delegations.flatMap(delegation => {
                            return [
                                this.methods.fundManagerUpdateRestakingVaultDelegationStrategy(
                                    v.vault,
                                    delegation.operator,
                                    delegation.supportedTokenAllocationWeight,
                                    delegation.supportedTokenAllocationCapacityAmount,
                                    null,
                                ).accountsPartial({
                                    receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                                })
                                .remainingAccounts(this.pricingSourceAccounts)
                                .instruction(),
                            ];
                        }),
                    ];
                }),
            ],
            signerNames: ["FUND_MANAGER"],
            events: ["fundManagerUpdatedFund"],
        });

        logger.notice(`updated fragSOL fund configuration`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLFund.toString());
        const fragSOLFund = await this.account.fundAccount.fetch(this.knownAddress.fragSOLFund);
        return {event, error, fragSOLFund};
    }

    public async runFundManagerInitializeNormalizeTokenPoolSupportedTokens() {
        await this.run({
            instructions: Object.entries(this.supportedTokenMetadata).flatMap(([symbol, v]) => {
                return [
                    spl.createAssociatedTokenAccountIdempotentInstruction(
                        this.wallet.publicKey,
                        this.knownAddress.nSOLSupportedTokenReserveAccount(symbol as any),
                        this.knownAddress.nSOLTokenPool,
                        v.mint,
                        v.program,
                    ),
                    this.program.methods
                        .fundManagerAddNormalizedTokenPoolSupportedToken(
                            v.pricingSource,
                        )
                        .accountsPartial({
                            normalizedTokenMint: this.knownAddress.nSOLTokenMint,
                            supportedTokenMint: v.mint,
                            supportedTokenProgram: v.program,
                        })
                        .remainingAccounts(this.pricingSourceAccounts)
                        .instruction(),
                ];
            }),
            signerNames: ["FUND_MANAGER"],
        });

        logger.notice(`configured nSOL supported tokens`.padEnd(LOG_PAD_LARGE), this.knownAddress.nSOLTokenPool.toString());
        const nSOLTokenPool = await this.account.normalizedTokenPoolAccount.fetch(this.knownAddress.nSOLTokenPool);
        return {nSOLTokenPool};
    }

    public async runFundManagerAddNormalizeTokenPoolSupportedToken(symbol: keyof typeof this.supportedTokenMetadata) {
        const token = this.supportedTokenMetadata[symbol];
        await this.run({
            instructions: [
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.nSOLSupportedTokenReserveAccount(symbol as any),
                    this.knownAddress.nSOLTokenPool,
                    token.mint,
                    token.program,
                ),
                this.program.methods
                    .fundManagerAddNormalizedTokenPoolSupportedToken(
                        token.pricingSource,
                    )
                    .accountsPartial({
                        normalizedTokenMint: this.knownAddress.nSOLTokenMint,
                        supportedTokenMint: token.mint,
                        supportedTokenProgram: token.program,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signerNames: ["FUND_MANAGER"],
        });

        logger.notice(`added nSOL supported tokens`.padEnd(LOG_PAD_LARGE), this.knownAddress.nSOLTokenPool.toString());
        const nSOLTokenPool = await this.account.normalizedTokenPoolAccount.fetch(this.knownAddress.nSOLTokenPool);
        return {nSOLTokenPool};
    }

    public get rewardsMetadata() {
        return this._getRewardsMetadata;
    }

    private readonly _getRewardsMetadata = this.getRewardsMetadata();

    private getRewardsMetadata() {
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
                            receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                        })
                        .instruction();
                }),
                ...this.rewardsMetadata.map((v) => {
                    return this.program.methods
                        .fundManagerAddReward(v.name, v.description, v.type)
                        .accountsPartial({
                            receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                            rewardTokenMint: v.tokenMint ?? this.programId,
                            rewardTokenProgram: v.tokenProgram ?? this.programId,
                        })
                        .instruction();
                }),
            ],
            signerNames: ["FUND_MANAGER"],
            events: ["fundManagerUpdatedRewardPool"],
        });

        logger.notice(`configured fragSOL reward pools and reward`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLReward.toString());
        const fragSOLReward = await this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward);
        return {event, error, fragSOLReward};
    }

    public async runFundManagerSettleReward(args: {
        poolName: (typeof this.rewardPoolsMetadata)[number]["name"];
        rewardName: (typeof this.rewardsMetadata)[number]["name"];
        amount: BN
    }) {
        let fragSOLReward = await this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward);
        let rewardPool = fragSOLReward.rewardPools1.find((r) => this.binToString(r.name) == args.poolName);
        let reward = fragSOLReward.rewards1.find((r) => this.binToString(r.name) == args.rewardName);

        const rewardTokenMint = this.binIsEmpty(reward.tokenMint.toBuffer()) ? this.programId : reward.tokenMint;
        const rewardTokenProgram = this.binIsEmpty(reward.tokenProgram.toBuffer()) ? this.programId : reward.tokenProgram;
        const {event, error} = await this.run({
            instructions: [
                this.program.methods
                    .fundManagerSettleReward(rewardPool.id, reward.id, args.amount)
                    .accountsPartial({
                        receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                        rewardTokenMint,
                        rewardTokenProgram,
                    })
                    .instruction(),
            ],
            signerNames: ["FUND_MANAGER"],
            events: ["fundManagerUpdatedRewardPool"],
        });

        logger.notice(`settled fragSOL reward to pool=${rewardPool.id}/${args.poolName}, rewardId=${reward.id}/${args.rewardName}, amount=${args.amount.toString()} (decimals=${reward.decimals})`);
        fragSOLReward = await this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward);
        rewardPool = fragSOLReward.rewardPools1.find((r) => this.binToString(r.name) == args.poolName);
        reward = fragSOLReward.rewards1.find((r) => this.binToString(r.name) == args.rewardName);

        return {event, error, fragSOLReward, rewardPool, reward};
    }

    private async getInstructionsToUpdateUserFragSOLFundAndRewardAccounts(user: web3.Keypair) {
        // Assumes that both user fund & reward account size is small enough

        // const fragSOLUserRewardAddress = this.knownAddress.fragSOLUserReward(user.publicKey);
        // const fragSOLUserFundAddress = this.knownAddress.fragSOLUserFund(user.publicKey);
        // const currentRewardVersion = await this.account.userRewardAccount
        //     .fetch(fragSOLUserRewardAddress)
        //     .then((a) => a.dataVersion)
        //     .catch((err) => 0);
        // const currentFundVersion = await this.account.userFundAccount
        //     .fetch(fragSOLUserFundAddress)
        //     .then((a) => a.dataVersion)
        //     .catch((err) => 0);
        //
        // const targetRewardVersion = parseInt(this.getConstant("userRewardAccountCurrentVersion"));
        return [
            spl.createAssociatedTokenAccountIdempotentInstruction(
                user.publicKey,
                this.knownAddress.fragSOLUserTokenAccount(user.publicKey),
                user.publicKey,
                this.knownAddress.fragSOLTokenMint,
                spl.TOKEN_2022_PROGRAM_ID,
            ),
            this.program.methods.userCreateFundAccountIdempotent(null)
                .accountsPartial({
                    user: user.publicKey,
                    receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                })
                .instruction(),
            this.program.methods.userCreateRewardAccountIdempotent(null)
                .accountsPartial({
                    user: user.publicKey,
                    receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                })
                .instruction(),
            // ...(currentFundVersion == 0
            //     ? [
            //         this.program.methods.userInitializeFundAccount()
            //             .accounts({
            //                 user: user.publicKey,
            //                 receiptTokenMint: this.knownAddress.fragSOLTokenMint,
            //             })
            //             .instruction(),
            //     ]
            //     : [
            //         this.program.methods.userUpdateFundAccountIfNeeded()
            //             .accountsPartial({
            //                 user: user.publicKey,
            //                 receiptTokenMint: this.knownAddress.fragSOLTokenMint,
            //             })
            //             .instruction(),
            //     ]),
            // ...(currentRewardVersion == 0 ? [
            //     this.program.methods.userInitializeRewardAccount()
            //         .accountsPartial({
            //             user: user.publicKey,
            //             receiptTokenMint: this.knownAddress.fragSOLTokenMint,
            //         })
            //         .instruction(),
            //     ]
            //     : [
            //         ...new Array(targetRewardVersion - currentRewardVersion).fill(null).map((_, index, arr) =>
            //             this.program.methods
            //                 .userUpdateRewardAccountIfNeeded(null)
            //                 .accountsPartial({
            //                     user: user.publicKey,
            //                     receiptTokenMint: this.knownAddress.fragSOLTokenMint,
            //                 })
            //                 .instruction(),
            //         ),
            //     ]),
        ];
    }

    public async runUserCreateOrUpdateFragSOLFundAndRewardAccount(user: web3.Keypair) {
        await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragSOLFundAndRewardAccounts(user)),
            ],
            signers: [user],
        });
    }

    public async runUserDepositSOL(user: web3.Keypair, amount: BN, depositMetadata: IdlTypes<Restaking>["depositMetadata"]|null = null, depositMetadataSigningKeypair: web3.Keypair|null = null) {
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

        const {event, error} = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragSOLFundAndRewardAccounts(user)),
                ...depositMetadataInstruction,
                this.program.methods
                    .userDepositSol(amount, depositMetadata)
                    .accountsPartial({
                        user: user.publicKey,
                        receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signers: [user],
            events: ["userDepositedToFund"],
        });

        const [fragSOLFund, fragSOLFundReserveAccountBalance, fragSOLReward, fragSOLUserFund, fragSOLUserReward, fragSOLUserTokenAccount] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragSOLFund),
            this.getFragSOLFundReserveAccountBalance(),
            this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward),
            this.account.userFundAccount.fetch(this.knownAddress.fragSOLUserFund(user.publicKey)),
            this.account.userRewardAccount.fetch(this.knownAddress.fragSOLUserReward(user.publicKey)),
            this.getUserFragSOLAccount(user.publicKey),
        ]);
        logger.notice(`deposited: ${this.lamportsToSOL(amount)} (${this.lamportsToX(fragSOLFund.oneReceiptTokenAsSol, fragSOLFund.receiptTokenDecimals, 'SOL/fragSOL')})`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());
        logger.info(`user fragSOL balance: ${this.lamportsToFragSOL(new BN(fragSOLUserTokenAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());

        return {
            event,
            error,
            fragSOLFund,
            fragSOLFundReserveAccountBalance,
            fragSOLReward,
            fragSOLUserFund,
            fragSOLUserReward,
            fragSOLUserTokenAccount
        };
    }

    public lamportsToFragSOL(lamports: BN): string {
        return super.lamportsToX(lamports, this.fragSOLDecimals, "fragSOL");
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
                ...(await this.getInstructionsToUpdateUserFragSOLFundAndRewardAccounts(user)),
                ...depositMetadataInstruction,
                this.program.methods
                    .userDepositSupportedToken(amount, depositMetadata)
                    .accountsPartial({
                        user: user.publicKey,
                        receiptTokenMint: this.knownAddress.fragSOLTokenMint,
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

        const [fragSOLFund, fragSOLFundReserveAccountBalance, fragSOLReward, fragSOLUserFund, fragSOLUserReward, fragSOLUserTokenAccount, userSupportedTokenAccount] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragSOLFund),
            this.getFragSOLFundReserveAccountBalance(),
            this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward),
            this.account.userFundAccount.fetch(this.knownAddress.fragSOLUserFund(user.publicKey)),
            this.account.userRewardAccount.fetch(this.knownAddress.fragSOLUserReward(user.publicKey)),
            this.getUserFragSOLAccount(user.publicKey),
            this.getUserSupportedTokenAccount(user.publicKey, tokenSymbol),
        ]);
        logger.notice(`deposited: ${this.lamportsToX(amount, supportedToken.decimals, tokenSymbol)} (${this.lamportsToX(fragSOLFund.oneReceiptTokenAsSol, fragSOLFund.receiptTokenDecimals, 'SOL/fragSOL')})`.padEnd(LOG_PAD_LARGE), userSupportedTokenAddress.toString());
        logger.info(`user fragSOL balance: ${this.lamportsToFragSOL(new BN(fragSOLUserTokenAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());

        return {
            event,
            error,
            fragSOLFund,
            fragSOLFundReserveAccountBalance,
            fragSOLReward,
            fragSOLUserFund,
            fragSOLUserReward,
            fragSOLUserTokenAccount,
            userSupportedTokenAccount
        };
    }

    public async runOperatorDonateSOLToFund(
        operator: web3.Keypair,
        amount: BN,
        offsetReceivable: boolean = false,
    ) {
        const {event, error} = await this.run({
            instructions: [
                web3.ComputeBudgetProgram.setComputeUnitLimit({
                    units: 300_000,
                }),
                this.program.methods
                    .operatorDonateSolToFund(amount, offsetReceivable)
                    .accountsPartial({
                        operator: operator.publicKey,
                        receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signers: [operator],
            events: ["operatorDonatedToFund"],
        });

        const [fragSOLFund, fragSOLFundReserveAccountBalance] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragSOLFund),
            this.getFragSOLFundReserveAccountBalance(),
        ]);
        logger.notice(`operator donated: ${this.lamportsToSOL(amount)} (${this.lamportsToX(fragSOLFund.oneReceiptTokenAsSol, fragSOLFund.receiptTokenDecimals, 'SOL/fragSOL')})`.padEnd(LOG_PAD_LARGE), operator.publicKey.toString());

        return {
            event,
            error,
            fragSOLFund,
            fragSOLFundReserveAccountBalance,
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
                        receiptTokenMint: this.knownAddress.fragSOLTokenMint,
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

        const [fragSOLFund, fragSOLFundSupportedTokenAccount] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragSOLFund),
            this.getFragSOLFundReserveAccountBalance(),
            this.getFragSOLFundSupportedTokenReserveAccount(tokenSymbol),
        ]);
        logger.notice(`operator donated: ${this.lamportsToX(amount, supportedToken.decimals, tokenSymbol)} (${this.lamportsToX(fragSOLFund.oneReceiptTokenAsSol, fragSOLFund.receiptTokenDecimals, 'SOL/fragSOL')})`.padEnd(LOG_PAD_LARGE), operatorSupportedTokenAddress.toString());

        return {
            event,
            error,
            fragSOLFund,
            fragSOLFundSupportedTokenAccount,
        };
    }

    public async runUserRequestWithdrawal(user: web3.Keypair, amount: BN, supportedTokenMint: web3.PublicKey|null = null) {
        const {event, error} = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragSOLFundAndRewardAccounts(user)),
                this.program.methods
                    .userRequestWithdrawal(amount, supportedTokenMint)
                    .accountsPartial({
                        user: user.publicKey,
                        receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signers: [user],
            events: ["userRequestedWithdrawalFromFund"],
        });

        logger.notice(
            `requested withdrawal: ${this.lamportsToFragSOL(event.userRequestedWithdrawalFromFund.requestedReceiptTokenAmount)} -> ${supportedTokenMint ?? 'SOL'} #${event.userRequestedWithdrawalFromFund.requestId.toString()}/${event.userRequestedWithdrawalFromFund.batchId.toString()}`.padEnd(LOG_PAD_LARGE),
            user.publicKey.toString()
        );
        const [fragSOLFund, fragSOLFundReserveAccountBalance, fragSOLReward, fragSOLUserFund, fragSOLUserReward, fragSOLUserTokenAccount, fragSOLLockAccount] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragSOLFund),
            this.getFragSOLFundReserveAccountBalance(),
            this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward),
            this.account.userFundAccount.fetch(this.knownAddress.fragSOLUserFund(user.publicKey)),
            this.account.userRewardAccount.fetch(this.knownAddress.fragSOLUserReward(user.publicKey)),
            this.getUserFragSOLAccount(user.publicKey),
            this.getFragSOLFundReceiptTokenLockAccount(),
        ]);

        logger.info(`user fragSOL balance: ${this.lamportsToFragSOL(new BN(fragSOLUserTokenAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());

        return {
            event,
            error,
            fragSOLFund,
            fragSOLFundReserveAccountBalance,
            fragSOLReward,
            fragSOLUserFund,
            fragSOLUserReward,
            fragSOLUserTokenAccount,
            fragSOLLockAccount
        };
    }

    public async runUserCancelWithdrawalRequest(
        user: web3.Keypair,
        requestId: BN,
        supportedTokenMint: web3.PublicKey | null = null,
    ) {
        const {event, error} = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragSOLFundAndRewardAccounts(user)),
                this.program.methods
                    .userCancelWithdrawalRequest(requestId, supportedTokenMint)
                    .accountsPartial({
                        user: user.publicKey,
                        receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signers: [user],
            events: ["userCanceledWithdrawalRequestFromFund"],
        });

        logger.notice(`canceled withdrawal request: #${requestId.toString()}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());
        const [fragSOLFund, fragSOLFundReserveAccountBalance, fragSOLReward, fragSOLUserFund, fragSOLUserReward, fragSOLUserTokenAccount, fragSOLLockAccount] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragSOLFund),
            this.getFragSOLFundReserveAccountBalance(),
            this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward),
            this.account.userFundAccount.fetch(this.knownAddress.fragSOLUserFund(user.publicKey)),
            this.account.userRewardAccount.fetch(this.knownAddress.fragSOLUserReward(user.publicKey)),
            this.getUserFragSOLAccount(user.publicKey),
            this.getFragSOLFundReceiptTokenLockAccount(),
        ]);

        logger.info(`user fragSOL balance: ${this.lamportsToFragSOL(new BN(fragSOLUserTokenAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());

        return {
            event,
            error,
            fragSOLFund,
            fragSOLFundReserveAccountBalance,
            fragSOLReward,
            fragSOLUserFund,
            fragSOLUserReward,
            fragSOLUserTokenAccount,
            fragSOLLockAccount
        };
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
                        receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signers: [operator],
            events: ["operatorUpdatedFundPrices"],
        });

        const [fragSOLFund] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragSOLFund),
        ]);
        logger.notice(`operator updated fund prices: ${this.lamportsToSOL(fragSOLFund.oneReceiptTokenAsSol)}/fragSOL`.padEnd(LOG_PAD_LARGE), operator.publicKey.toString());

        return {
            event,
            error,
            fragSOLFund,
        };
    }

    public async runOperatorUpdateNormalizedTokenPoolPrices(operator: web3.Keypair = this.wallet) {
        const {event, error} = await this.run({
            instructions: [
                this.program.methods
                    .operatorUpdateNormalizedTokenPoolPrices()
                    .accountsPartial({
                        operator: operator.publicKey,
                        normalizedTokenMint: this.knownAddress.nSOLTokenMint,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signers: [operator],
            events: ["operatorUpdatedNormalizedTokenPoolPrices"],
        });

        const [nSOLTokenPoolAccount] = await Promise.all([
            this.getNSOLTokenPoolAccount(),
        ]);
        logger.notice(`operator updated normalized token pool prices: ${this.lamportsToSOL(nSOLTokenPoolAccount.oneNormalizedTokenAsSol)}/nSOL`.padEnd(LOG_PAD_LARGE), operator.publicKey.toString());

        return {
            event,
            error,
            nSOLTokenPoolAccount,
        };
    }

    public async runOperatorEnqueueWithdrawalBatches(operator: web3.Keypair = this.keychain.getKeypair('FUND_MANAGER'), forced: boolean = false) {
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

        const [fragSOLFund, fragSOLFundReserveAccountBalance, fragSOLReward, fragSOLLockAccount] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragSOLFund),
            this.getFragSOLFundReserveAccountBalance(),
            this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward),
            this.getFragSOLFundReceiptTokenLockAccount(),
        ]);
        logger.info(`operator enqueued withdrawal batches up to #${fragSOLFund.sol.withdrawalLastProcessedBatchId.toString()}`.padEnd(LOG_PAD_LARGE), operator.publicKey.toString());

        return {event, error, fragSOLFund, fragSOLFundReserveAccountBalance, fragSOLReward, fragSOLLockAccount};
    }

    public async runOperatorInitialize(operator: web3.Keypair = this.keychain.getKeypair("FUND_MANAGER")) {
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

    public async runOperatorProcessWithdrawalBatches(operator: web3.Keypair = this.keychain.getKeypair('FUND_MANAGER'), forced: boolean = false) {
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

        const [fragSOLFund, fragSOLFundReserveAccountBalance, fragSOLReward, fragSOLLockAccount] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragSOLFund),
            this.getFragSOLFundReserveAccountBalance(),
            this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward),
            this.getFragSOLFundReceiptTokenLockAccount(),
        ]);
        logger.info(`operator processed withdrawal batches up to #${fragSOLFund.sol.withdrawalLastProcessedBatchId.toString()}`.padEnd(LOG_PAD_LARGE), operator.publicKey.toString());

        return {event: event ?? _event, error: error ?? _error, fragSOLFund, fragSOLFundReserveAccountBalance, fragSOLReward, fragSOLLockAccount};
    }

    public async runUserWithdraw(user: web3.Keypair, supportedTokenMint: web3.PublicKey|null, requestId: BN) {
        const request = await this.getUserFragSOLFundAccount(user.publicKey)
            .then(userFundAccount => userFundAccount.withdrawalRequests.find(req => req.requestId.eq(requestId) && (supportedTokenMint ? supportedTokenMint.equals(req.supportedTokenMint) : !req.supportedTokenMint)));

        if (!request) {
            throw "request not found";
        }
        const userSupportedTokenAccount = request.supportedTokenMint ? spl.getAssociatedTokenAddressSync(request.supportedTokenMint, user.publicKey, true, request.supportedTokenProgram) : null;

        const {event, error} = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragSOLFundAndRewardAccounts(user)),
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
                                receiptTokenMint: this.knownAddress.fragSOLTokenMint,
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
                                receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                            })
                            .instruction(),
                    ]
                )
            ],
            signers: [user],
            events: ["userWithdrewFromFund"],
        });

        const [fragSOLFund, fragSOLFundReserveAccountBalance, fragSOLReward, fragSOLUserFund, fragSOLUserReward, fragSOLUserTokenAccount, fragSOLLockAccount] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragSOLFund),
            this.getFragSOLFundReserveAccountBalance(),
            this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward),
            this.account.userFundAccount.fetch(this.knownAddress.fragSOLUserFund(user.publicKey)),
            this.account.userRewardAccount.fetch(this.knownAddress.fragSOLUserReward(user.publicKey)),
            this.getUserFragSOLAccount(user.publicKey),
            this.getFragSOLFundReceiptTokenLockAccount(),
        ]);
        logger.notice(`user withdrew: ${this.lamportsToX(event.userWithdrewFromFund.withdrawnAmount, 9, event.userWithdrewFromFund.supportedTokenMint?.toString().substring(0, 4) ?? 'SOL' /** TODO: later.. **/)} #${requestId.toString()}, (${this.lamportsToX(fragSOLFund.oneReceiptTokenAsSol, fragSOLFund.receiptTokenDecimals, 'SOL/fragSOL')})`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());

        return {
            event,
            error,
            fragSOLFund,
            fragSOLFundReserveAccountBalance,
            fragSOLReward,
            fragSOLUserFund,
            fragSOLUserReward,
            fragSOLUserTokenAccount,
            fragSOLLockAccount
        };
    }

    public async runTransfer(source: web3.Keypair, destination: web3.PublicKey, amount: BN) {
        const {event, error} = await this.run({
            instructions: [
                spl.createTransferCheckedWithTransferHookInstruction(
                    this.connection,
                    this.knownAddress.fragSOLUserTokenAccount(source.publicKey),
                    this.knownAddress.fragSOLTokenMint,
                    this.knownAddress.fragSOLUserTokenAccount(destination),
                    source.publicKey,
                    BigInt(amount.toString()),
                    this.fragSOLDecimals,
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
                        receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                    })
                    .instruction(),
            ],
            signers: [user],
            events: ['userUpdatedRewardPool'],
        });

        logger.notice(`user manually updated user reward pool:`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());
        const [fragSOLUserReward] = await Promise.all([this.account.userRewardAccount.fetch(this.knownAddress.fragSOLUserReward(user.publicKey))]);

        return {event, error, fragSOLUserReward};
    }

    public async runOperatorUpdateRewardPools(operator: web3.Keypair = this.wallet) {
        const {event, error} = await this.run({
            instructions: [
                this.program.methods
                    .operatorUpdateRewardPools()
                    .accountsPartial({
                        operator: operator.publicKey,
                        receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                    })
                    .instruction(),
            ],
            signers: [operator],
            events: ["operatorUpdatedRewardPools"], // won't emit it for such void update requests
        });

        logger.notice(`operator manually updated global reward pool:`.padEnd(LOG_PAD_LARGE), operator.publicKey.toString());
        const [fragSOLReward] = await Promise.all([this.getFragSOLRewardAccount()]);

        return {event, error, fragSOLReward};
    }

    public async runOperatorFundCommands(resetCommand: Parameters<typeof this.program.methods.operatorRunFundCommand>[0] = null, operator: web3.Keypair = this.keychain.getKeypair('FUND_MANAGER'), maxTxCount = 100, setComputeUnitLimitUnits?: number, setComputeUnitPriceMicroLamports?: number) {
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
        const requiredAccounts: Map<string, web3.AccountMeta> = new Map();
        this.pricingSourceAccounts.forEach(accoutMeta => {
            requiredAccounts.set(accoutMeta.pubkey.toBase58(), accoutMeta);
        });
        requiredAccounts.set(this.knownAddress.programEventAuthority.toBase58(), {
            pubkey: this.knownAddress.programEventAuthority,
            isWritable: false,
            isSigner: false,
        });
        requiredAccounts.set(this.programId.toBase58(), {
            pubkey: this.programId,
            isWritable: false,
            isSigner: false,
        });
        requiredAccounts.set(operator.publicKey.toBase58(), {
            pubkey: operator.publicKey,
            isWritable: true,
            isSigner: true,
        });
        requiredAccounts.set(web3.SystemProgram.programId.toBase58(), {
            pubkey: web3.SystemProgram.programId,
            isWritable: false,
            isSigner: false,
        });
        requiredAccounts.set(this.knownAddress.fragSOLTokenMint.toBase58(), {
            pubkey: this.knownAddress.fragSOLTokenMint,
            isWritable: true,
            isSigner: false,
        });
        requiredAccounts.set(this.knownAddress.fragSOLFund.toBase58(), {
            pubkey: this.knownAddress.fragSOLFund,
            isWritable: true,
            isSigner: false,
        });

        let fragSOLFund = await this.getFragSOLFundAccount();
        let nextOperationCommand = resetCommand ?? fragSOLFund.operation.nextCommand;
        let nextOperationSequence = resetCommand ? 0 : fragSOLFund.operation.nextSequence;
        if (nextOperationCommand) {
            for (let i = 0; i < nextOperationCommand.numRequiredAccounts; i++) {
                const accountMeta = nextOperationCommand.requiredAccounts[i];
                const pubkey = accountMeta.pubkey.toBase58();
                if (requiredAccounts.has(pubkey)) {
                    if (accountMeta.isWritable != 0) {
                        requiredAccounts.get(pubkey).isWritable = true;
                    }
                } else {
                    requiredAccounts.set(pubkey, {
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
                        receiptTokenMint: this.knownAddress.fragSOLTokenMint,
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

        // ... track fund asset state
        if (commandResult) {
            await Promise.all([
                this.getFragSOLFundAccount(),
                this.getFragSOLFundReceiptTokenLockAccount().then(a => new BN(a.amount.toString())),
                this.getFragSOLFundReserveAccountBalance(),
                this.getFragSOLFundNSOLAccountBalance(),
                this.getNSOLTokenPoolAccount(),
                ...Object.keys(this.supportedTokenMetadata).map(
                    symbol => this.getFragSOLFundSupportedTokenReserveAccount(symbol as keyof typeof this.supportedTokenMetadata)
                        .then(a => [a.mint, new BN(a.amount.toString())])
                ),
                ...Object.keys(this.restakingVaultMetadata).map(
                    symbol => this.getFragSOLFundVRTAccount(symbol as keyof typeof this.restakingVaultMetadata)
                        .then(a => [a.mint, new BN(a.amount.toString())])
                ),
            ]).then(([fund, fragSOLLocked, sol, nSOL, ntp, ...tokens]) => {
                console.log('fund asset state:', {
                    receiptToken: {
                        oneTokenAsSOL: fund.oneReceiptTokenAsSol,
                        supply: fund.receiptTokenSupplyAmount,
                        locked: fragSOLLocked,
                    },
                    accounts: {
                        sol,
                        tokens: {
                            [this.knownAddress.nSOLTokenMint.toString()]: nSOL,
                            ...Object.fromEntries(tokens),
                        },
                    },
                    data: {
                        sol: {
                            reserved: fund.sol.operationReservedAmount,
                            receivable: fund.sol.operationReceivableAmount,
                            withdrawable: fund.sol.withdrawalUserReservedAmount,
                            total: fund.sol.operationReservedAmount.add(fund.sol.operationReceivableAmount).add(fund.sol.withdrawalUserReservedAmount),
                        },
                        tokens: {
                            [this.knownAddress.nSOLTokenMint.toString()]: {
                                reserved: fund.normalizedToken.operationReservedAmount,
                                supply: ntp.normalizedTokenSupplyAmount,
                                oneTokenAsSOL: ntp.oneNormalizedTokenAsSol,
                                total: fund.normalizedToken.operationReservedAmount,
                            },
                            ...Object.fromEntries(fund.supportedTokens.slice(0, fund.numSupportedTokens).map(supported => {
                                return [supported.mint, {
                                    reserved: supported.token.operationReservedAmount,
                                    receivable: supported.token.operationReceivableAmount,
                                    withdrawable: supported.token.withdrawalUserReservedAmount,
                                    normalized: ntp.supportedTokens.find(st => st.mint.equals(supported.mint))?.lockedAmount ?? 0,
                                    total: supported.token.operationReservedAmount.add(supported.token.operationReceivableAmount).add(supported.token.withdrawalUserReservedAmount),
                                }];
                            })),
                            ...Object.fromEntries(fund.restakingVaults.slice(0, fund.numRestakingVaults).map(vault => {
                                return [vault.receiptTokenMint, {
                                    reserved: vault.receiptTokenOperationReservedAmount,
                                    receivable: vault.receiptTokenOperationReceivableAmount,
                                    total: vault.receiptTokenOperationReservedAmount.add(vault.receiptTokenOperationReceivableAmount),
                                }];
                            }))
                        },
                    },
                });
            });
        }

        return {
            event: tx.event,
            error: tx.error,
        };
    }
}
