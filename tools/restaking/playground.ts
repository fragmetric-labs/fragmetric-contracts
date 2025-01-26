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
            () => this.runOperatorUpdateFundPrices(), // 13
            () => this.runOperatorUpdateNormalizedTokenPoolPrices(), // 14
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
                ...(Object.values(this.knownAddress)
                    .filter(address => typeof address != 'function').flat() as web3.PublicKey[]),
                ...this.pricingSourceAccounts.map(meta => meta.pubkey),
            ]
            .filter(address => !existingAddresses.has(address.toString()));
            return [...new Set(addresses)];
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

        // nSOL
        const nSOLTokenMint = this.getConstantAsPublicKey("fragsolNormalizedTokenMintAddress");
        const nSOLTokenMintBuf = nSOLTokenMint.toBuffer();

        // jitoSOL
        const jitoSOLTokenMint = this.supportedTokenMetadata['jitoSOL'].mint;
        const jitoSOLTokenMintBuf = jitoSOLTokenMint.toBuffer();

        // fragSOL jito nSOL VRT
        const fragSOLJitoNSOLVRTMint = this.getConstantAsPublicKey('fragsolJitoNsolVaultReceiptTokenMintAddress');

        // fragSOL jito jitoSOL VRT
        const fragSOLJitoJitoSOLVRTMint = this.getConstantAsPublicKey('fragsolJitoJitosolVaultReceiptTokenMintAddress');

        // fragSOL fund & ATAs
        const [fragSOLFund] = web3.PublicKey.findProgramAddressSync([Buffer.from("fund"), fragSOLTokenMintBuf], this.programId);
        const [fragSOLFundReserveAccount] = web3.PublicKey.findProgramAddressSync([Buffer.from("fund_reserve"), fragSOLTokenMintBuf], this.programId);
        const fragSOLFundReserveSupportedTokenAccount = (symbol: keyof typeof this.supportedTokenMetadata) =>
            spl.getAssociatedTokenAddressSync(this.supportedTokenMetadata[symbol].mint, fragSOLFundReserveAccount, true, this.supportedTokenMetadata[symbol].program);
        const fragSOLFundReserveSupportedTokenAccounts = Object.keys(this.supportedTokenMetadata).reduce((obj, symbol) => ({
            [`fragSOLFundReserveSupportedTokenAccount_${symbol}`]: fragSOLFundReserveSupportedTokenAccount(symbol as any),
            ...obj,
        }), {} as { string: web3.PublicKey });
        const [fragSOLFundTreasuryAccount] = web3.PublicKey.findProgramAddressSync([Buffer.from("fund_treasury"), fragSOLTokenMintBuf], this.programId);
        const fragSOLFundTreasurySupportedTokenAccount = (symbol: keyof typeof this.supportedTokenMetadata) =>
            spl.getAssociatedTokenAddressSync(this.supportedTokenMetadata[symbol].mint, fragSOLFundTreasuryAccount, true, this.supportedTokenMetadata[symbol].program);
        const fragSOLFundTreasurySupportedTokenAccounts = Object.keys(this.supportedTokenMetadata).reduce((obj, symbol) => ({
            [`fragSOLFundTreasurySupportedTokenAccount_${symbol}`]: fragSOLFundTreasurySupportedTokenAccount(symbol as any),
            ...obj,
        }), {} as { string: web3.PublicKey });

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
        const userSupportedTokenAccount = (user: web3.PublicKey, symbol: keyof typeof this.supportedTokenMetadata) =>
            spl.getAssociatedTokenAddressSync(this.supportedTokenMetadata[symbol].mint, user, false, this.supportedTokenMetadata[symbol].program);

        const fragSOLFundWithdrawalBatch = (supportedTokenMint: web3.PublicKey|null, batchId: BN) => web3.PublicKey.findProgramAddressSync([Buffer.from("withdrawal_batch"), fragSOLTokenMintBuf, (supportedTokenMint || web3.PublicKey.default).toBuffer(), batchId.toBuffer('le', 8)], this.programId)[0];

        // reward
        const [fragSOLReward] = web3.PublicKey.findProgramAddressSync([Buffer.from("reward"), fragSOLTokenMintBuf], this.programId);
        const fragSOLUserReward = (user: web3.PublicKey) => web3.PublicKey.findProgramAddressSync([Buffer.from("user_reward"), fragSOLTokenMintBuf, user.toBuffer()], this.programId)[0];

        // NTP
        const [nSOLTokenPool] = web3.PublicKey.findProgramAddressSync([Buffer.from("nt_pool"), nSOLTokenMintBuf], this.programId);
        const nSOLSupportedTokenReserveAccount = (symbol: keyof typeof this.supportedTokenMetadata) =>
            spl.getAssociatedTokenAddressSync(this.supportedTokenMetadata[symbol].mint, nSOLTokenPool, true, this.supportedTokenMetadata[symbol].program);
        const nSOLSupportedTokenReserveAccounts = Object.keys(this.supportedTokenMetadata).reduce((obj, symbol) => ({
            [`nSOLSupportedTokenReserveAccount_${symbol}`]: nSOLSupportedTokenReserveAccount(symbol as any),
            ...obj,
        }), {} as { string: web3.PublicKey });

        // // TODO: deprecate (client must not know this)
        // const vaultBaseAccount1 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_base_account1"), fragSOLTokenMintBuf], this.programId)[0];
        // const vaultBaseAccount2 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_base_account2"), fragSOLTokenMintBuf], this.programId)[0];
        // const vaultBaseAccount3 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_base_account3"), fragSOLTokenMintBuf], this.programId)[0];
        // const vaultBaseAccount4 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_base_account4"), fragSOLTokenMintBuf], this.programId)[0];
        // const vaultBaseAccount5 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_base_account5"), fragSOLTokenMintBuf], this.programId)[0];

        // jito
        const jitoVaultProgram = this.getConstantAsPublicKey('jitoVaultProgramId');
        const jitoVaultProgramFeeWallet = this.getConstantAsPublicKey('jitoVaultProgramFeeWallet');
        const jitoVaultConfig = this.getConstantAsPublicKey('jitoVaultConfigAddress');
        const jitoRestakingProgram = this.getConstantAsPublicKey('jitoRestakingProgramId');
        const jitoRestakingConfig = this.getConstantAsPublicKey('jitoRestakingConfigAddress');

        // fragSOL jito vault
        const fragSOLJitoVaultUpdateStateTracker = (vaultAccount: web3.PublicKey) => {
            return (slot: anchor.BN, epoch_length: anchor.BN) => {
                let ncn_epoch = slot.div(epoch_length).toBuffer('le', 8);
                return web3.PublicKey.findProgramAddressSync([Buffer.from('vault_update_state_tracker'), vaultAccount.toBuffer(), ncn_epoch], jitoVaultProgram)[0];
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
            this.keychain.getPublicKey('ADMIN'),
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
            this.keychain.getPublicKey('ADMIN'),
            false,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );

        // // TODO: deprecate (client must not know this)
        // const fragSOLJitoVaultWithdrawalTicketAccount1 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_staker_withdrawal_ticket"), fragSOLJitoNSOLVaultAccount.toBuffer(), vaultBaseAccount1.toBuffer()], jitoVaultProgram)[0];
        // const fragSOLJitoVaultWithdrawalTicketTokenAccount1 = spl.getAssociatedTokenAddressSync(
        //     fragSOLJitoNSOLVRTMint,
        //     fragSOLJitoVaultWithdrawalTicketAccount1,
        //     true,
        //     spl.TOKEN_PROGRAM_ID,
        //     spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        // )
        // const fragSOLJitoVaultWithdrawalTicketAccount2 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_staker_withdrawal_ticket"), fragSOLJitoNSOLVaultAccount.toBuffer(), vaultBaseAccount2.toBuffer()], jitoVaultProgram)[0];
        // const fragSOLJitoVaultWithdrawalTicketTokenAccount2 = spl.getAssociatedTokenAddressSync(
        //     fragSOLJitoNSOLVRTMint,
        //     fragSOLJitoVaultWithdrawalTicketAccount2,
        //     true,
        //     spl.TOKEN_PROGRAM_ID,
        //     spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        // );

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
            fragSOLFundReserveSupportedTokenAccount,
            ...fragSOLFundReserveSupportedTokenAccounts,
            fragSOLFundTreasuryAccount,
            fragSOLFundTreasurySupportedTokenAccount,
            ...fragSOLFundTreasurySupportedTokenAccounts,
            fragSOLFundReceiptTokenLockAccount,
            fragSOLFundNSOLAccount,
            fragSOLFundJitoNSOLVRTAccount,
            fragSOLFundJitoJitoSOLVRTAccount,
            fragSOLUserFund,
            fragSOLUserTokenAccount,
            userSupportedTokenAccount,
            fragSOLFundWithdrawalBatch,
            // reward
            fragSOLReward,
            fragSOLUserReward,
            // NTP
            nSOLTokenPool,
            nSOLSupportedTokenReserveAccount,
            ...nSOLSupportedTokenReserveAccounts,
            // jito
            jitoVaultProgram,
            jitoVaultProgramFeeWallet,
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

            // // TODO: deprecate (client must not know this)
            // fragSOLJitoVaultWithdrawalTicketAccount1,
            // fragSOLJitoVaultWithdrawalTicketTokenAccount1,
            // fragSOLJitoVaultWithdrawalTicketAccount2,
            // fragSOLJitoVaultWithdrawalTicketTokenAccount2,
            // vaultBaseAccount1,
            // vaultBaseAccount2,

            tokenProgram: spl.TOKEN_PROGRAM_ID,
            token2022Program: spl.TOKEN_2022_PROGRAM_ID,
            sysvarClock: new web3.PublicKey("SysvarC1ock11111111111111111111111111111111"),
            sysvarStakeHistory: new web3.PublicKey("SysvarStakeHistory1111111111111111111111111"),
            stakeProgram: new web3.PublicKey("Stake11111111111111111111111111111111111111"),
            systemProgram: new web3.PublicKey("11111111111111111111111111111111"),
            splStakePoolProgram: new web3.PublicKey("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy"),
        };
    }

    public readonly fragSOLDecimals = 9;
    public readonly nSOLDecimals = 9;
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

            // Later we will remove bSOL from mainnet.
            // If you also want to remove bSOL from localnet and from test,
            // you will have to fix tests.

            // // remove bSOL from mainnet
            // if (this.isMainnet) {
            //     delete metadata.bSOL;
            // }

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
                jito1: {
                    VSTMint: this.knownAddress.nSOLTokenMint,
                    VRTMint: this.knownAddress.fragSOLJitoNSOLVRTMint,
                    vault: this.getConstantAsPublicKey("fragsolJitoNsolVaultAccountAddress"),
                    program: this.getConstantAsPublicKey("jitoVaultProgramId"),
                    programFeeWalletTokenAccount: this.knownAddress.fragSOLJitoNSOLVaultProgramFeeWalletTokenAccount,
                    feeWalletTokenAccount: this.knownAddress.fragSOLJitoNSOLVaultFeeWalletTokenAccount,
                    vaultTokenAccount: this.knownAddress.fragSOLJitoNSOLVaultTokenAccount,
                    fundVRTAccount: this.knownAddress.fragSOLFundJitoNSOLVRTAccount,
                },
                // jito2: {
                //     VSTMint: this.knownAddress.jitoSOLTokenMint,
                //     VRTMint: this.knownAddress.fragSOLJitoJitoSOLVRTMint,
                //     vault: this.getConstantAsPublicKey("fragsolJitoJitosolVaultAccountAddress"),
                //     program: this.getConstantAsPublicKey("jitoVaultProgramId"),
                //     programFeeWalletTokenAccount: this.knownAddress.fragSOLJitoJitoSOLVaultProgramFeeWalletTokenAccount,
                //     feeWalletTokenAccount: this.knownAddress.fragSOLJitoJitoSOLVaultFeeWalletTokenAccount,
                //     vaultTokenAccount: this.knownAddress.fragSOLJitoJitoSOLVaultTokenAccount,
                //     fundVRTAccount: this.knownAddress.fragSOLFundJitoJitoSOLVRTAccount,
                // }
            };
        } else {
            // for 'localnet', it would be cloned from mainnet
            return {
                jito1: {
                    VSTMint: this.knownAddress.nSOLTokenMint,
                    VRTMint: this.knownAddress.fragSOLJitoNSOLVRTMint,
                    vault: this.getConstantAsPublicKey("fragsolJitoNsolVaultAccountAddress"),
                    program: this.getConstantAsPublicKey("jitoVaultProgramId"),
                    programFeeWalletTokenAccount: this.knownAddress.fragSOLJitoNSOLVaultProgramFeeWalletTokenAccount,
                    feeWalletTokenAccount: this.knownAddress.fragSOLJitoNSOLVaultFeeWalletTokenAccount,
                    vaultTokenAccount: this.knownAddress.fragSOLJitoNSOLVaultTokenAccount,
                    fundVRTAccount: this.knownAddress.fragSOLFundJitoNSOLVRTAccount,
                },
                // jito2: {
                //     VSTMint: this.knownAddress.jitoSOLTokenMint,
                //     VRTMint: this.knownAddress.fragSOLJitoJitoSOLVRTMint,
                //     vault: this.getConstantAsPublicKey("fragsolJitoJitosolVaultAccountAddress"),
                //     program: this.getConstantAsPublicKey("jitoVaultProgramId"),
                //     programFeeWalletTokenAccount: this.knownAddress.fragSOLJitoJitoSOLVaultProgramFeeWalletTokenAccount,
                //     feeWalletTokenAccount: this.knownAddress.fragSOLJitoJitoSOLVaultFeeWalletTokenAccount,
                //     vaultTokenAccount: this.knownAddress.fragSOLJitoJitoSOLVaultTokenAccount,
                //     fundVRTAccount: this.knownAddress.fragSOLFundJitoJitoSOLVRTAccount,
                // }
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
                            connection: this.connection as unknown as anchor.web3.Connection,
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
            // @ts-ignore
            this.connection,
            this.knownAddress.userSupportedTokenAccount(user, symbol),
            "confirmed",
            this.supportedTokenMetadata[symbol].program
        );
    }

    public getUserFragSOLAccount(user: web3.PublicKey) {
        return spl.getAccount(
            // @ts-ignore
            this.connection,
            this.knownAddress.fragSOLUserTokenAccount(user),
            "confirmed",
            spl.TOKEN_2022_PROGRAM_ID
        );
    }

    public getFragSOLSupportedTokenTreasuryAccountBalance(symbol: keyof typeof this.supportedTokenMetadata) {
        return spl.getAccount(
            // @ts-ignore
            this.connection,
            this.knownAddress.fragSOLFundTreasurySupportedTokenAccount(symbol),
            "confirmed",
            this.supportedTokenMetadata[symbol].program
        ).then(v => new BN(v.amount.toString()));
    }

    public getProgramSupportedTokenRevenueAccountBalance(symbol: keyof typeof this.supportedTokenMetadata) {
        return spl.getAccount(
            // @ts-ignore
            this.connection,
            this.knownAddress.programSupportedTokenRevenueAccount(symbol),
            "confirmed",
            this.supportedTokenMetadata[symbol].program
        ).then(v => new BN(v.amount.toString()));
    }

    public getProgramRevenueAccountBalance() {
        return this.connection.getAccountInfo(this.knownAddress.programRevenueAccount).then(v => new BN(v.lamports));
    }

    public getFragSOLSupportedTokenReserveAccount(symbol: keyof typeof this.supportedTokenMetadata) {
        return spl.getAccount(
            // @ts-ignore
            this.connection,
            this.knownAddress.fragSOLFundReserveSupportedTokenAccount(symbol),
            "confirmed",
            this.supportedTokenMetadata[symbol].program
        );
    }

    public getFragSOLRestakingVaultReceiptTokenReserveAccount(symbol: keyof typeof this.restakingVaultMetadata) {
        let vault = this.restakingVaultMetadata[symbol];
        let account = spl.getAssociatedTokenAddressSync(
            vault.VRTMint,
            this.knownAddress.fragSOLFundReserveAccount,
            true,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        return spl.getAccount(
            // @ts-ignore
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
                // @ts-ignore
                this.connection,
                this.knownAddress.fragSOLFundReserveSupportedTokenAccount(symbol as any),
                "confirmed",
                token.program,
            );
        }
        throw new Error("fund supported token account not found")
    }

    public getFragSOLFundReceiptTokenLockAccount() {
        return spl.getAccount(
            // @ts-ignore
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
            // @ts-ignore
            this.connection,
            this.knownAddress.nSOLTokenMint,
            "confirmed",
            spl.TOKEN_PROGRAM_ID
        );
    }

    public getFragSOLTokenMint() {
        return spl.getMint(
            // @ts-ignore
            this.connection,
            this.knownAddress.fragSOLTokenMint,
            "confirmed",
            spl.TOKEN_2022_PROGRAM_ID
        );
    }

    public async getSplStakePoolInfo(stakePoolAddress: web3.PublicKey) {
        return splStakePool.stakePoolInfo(
            this.connection,
            stakePoolAddress,
        );
    }

    public async runAdminInitializeFragSOLTokenMint() {
        const metadata: splTokenMetadata.TokenMetadata = {
            mint: this.keychain.getPublicKey("FRAGSOL_MINT"),
            name: "Fragmetric Restaked SOL",
            symbol: "fragSOL",
            uri: "https://quicknode.quicknode-ipfs.com/ipfs/QmcueajXkNzoYRhcCv323PMC8VVGiDvXaaVXkMyYcyUSRw",
            additionalMetadata: [["description", `fragSOL is Solana's first native LRT that provides optimized restaking rewards.`]],
            updateAuthority: this.keychain.getPublicKey("ADMIN"),
        };
        const fileForMetadataURI = JSON.stringify(
            {
                name: metadata.name,
                symbol: metadata.symbol,
                description: metadata.additionalMetadata[0][1],
                image: "https://fragmetric-assets.s3.ap-northeast-2.amazonaws.com/fragsol.png",
                // attributes: [],
            },
            null,
            0
        );
        logger.debug(`fragSOL metadata file:\n> ${metadata.uri}\n> ${fileForMetadataURI}`);

        const mintInitialSize = spl.getMintLen([spl.ExtensionType.TransferHook, spl.ExtensionType.MetadataPointer]);
        const mintMetadataExtensionSize = spl.TYPE_SIZE + spl.LENGTH_SIZE + splTokenMetadata.pack(metadata).length;
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
                    name: metadata.name,
                    symbol: metadata.symbol,
                    uri: metadata.uri,
                    mintAuthority: this.keychain.getPublicKey("ADMIN"),
                    updateAuthority: metadata.updateAuthority,
                }),
                ...metadata.additionalMetadata.map(([field, value]) =>
                    splTokenMetadata.createUpdateFieldInstruction({
                        programId: spl.TOKEN_2022_PROGRAM_ID,
                        metadata: this.knownAddress.fragSOLTokenMint,
                        updateAuthority: metadata.updateAuthority,
                        field,
                        value,
                    })
                ),
            ],
            signerNames: ["ADMIN", "FRAGSOL_MINT"],
        });
        const fragSOLMint = await spl.getMint(
            // @ts-ignore
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

    public async runAdminUpdateTokenMetadata() {
        const fragSOLTokenMetadataAddress = this.knownAddress.fragSOLTokenMint;

        let tokenMetadata = await spl.getTokenMetadata(this.connection, fragSOLTokenMetadataAddress, undefined, spl.TOKEN_2022_PROGRAM_ID);
        logger.debug(`current token metadata:\n> ${JSON.stringify(tokenMetadata, null, 0)}`);

        const updatedFileForMetadataURI = JSON.stringify(
            {
                name: tokenMetadata.name,
                symbol: tokenMetadata.symbol,
                description: tokenMetadata.additionalMetadata[0][1],
                image: "https://fragmetric-assets.s3.ap-northeast-2.amazonaws.com/fragsol.png",
                // attributes: [],
            },
            null,
            0
        );
        logger.debug(`fragSOL metadata file:\n> ${updatedFileForMetadataURI}`);

        const updatedMetadataUri = "https://quicknode.quicknode-ipfs.com/ipfs/QmcueajXkNzoYRhcCv323PMC8VVGiDvXaaVXkMyYcyUSRw";
        const updatedMetadata = spl.updateTokenMetadata(tokenMetadata, splTokenMetadata.Field.Uri, updatedMetadataUri);
        logger.debug(`will update token metadata:\n> ${JSON.stringify(updatedMetadata, null, 0)}`);

        await this.run({
            instructions: [
                splTokenMetadata.createUpdateFieldInstruction({
                    programId: spl.TOKEN_2022_PROGRAM_ID,
                    metadata: this.knownAddress.fragSOLTokenMint,
                    updateAuthority: tokenMetadata.updateAuthority,
                    field: splTokenMetadata.Field.Uri,
                    value: updatedMetadataUri,
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

    public async runFundManagerInitializeFundJitoRestakingVaults() {
        const {event, error} = await this.run({
            instructions: Object.entries(this.restakingVaultMetadata).flatMap(([symbol, v]) => {
                return [
                    // TODO v0.3/restaking: adjust authority of fee wallet
                    spl.createAssociatedTokenAccountIdempotentInstruction(
                        this.wallet.publicKey,
                        v.feeWalletTokenAccount,
                        this.keychain.getPublicKey('ADMIN'),
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
                    this.keychain.getPublicKey('ADMIN'),
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
            fragSOLFundJitoFeeVRTAccount,
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
        logger.notice("jito VRT fee account created".padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLJitoNSOLVaultFeeWalletTokenAccount.toString());
        logger.notice("jito vault VST account created".padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLJitoNSOLVaultTokenAccount.toString());
        logger.notice("jito fund VRT account created".padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLFundJitoNSOLVRTAccount.toString());
        logger.notice("jito VRT account (of program fee wallet) created".padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLJitoNSOLVaultProgramFeeWalletTokenAccount.toString());

        return {fragSOLFundJitoVRTAccount, fragSOLJitoVaultTokenAccount, fragSOLFundJitoFeeVRTAccount, fragSOLJitoVaultProgramFeeWalletTokenAccount, fragSOLFundAccount};
    }

    // need for operation
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
                        this.knownAddress.fragSOLFundReserveSupportedTokenAccount(symbol as any),
                        this.knownAddress.fragSOLFundReserveAccount,
                        v.mint,
                        v.program,
                    ),
                    spl.createAssociatedTokenAccountIdempotentInstruction(
                        this.wallet.publicKey,
                        this.knownAddress.fragSOLFundTreasurySupportedTokenAccount(symbol as any),
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
            WithdrawalFeedRateBPS: this.isDevnet ? 20 : 20,
            withdrawalBatchThresholdSeconds: new BN(this.isDevnet ? 60 : (this.isMainnet ? 86400 : 60)), // seconds

            solDepositable: true,
            solAccumulatedDepositCapacity: this.isDevnet
                ? new BN(1_000_000_000).mul(new BN(web3.LAMPORTS_PER_SOL)) : (
                    this.isMainnet ? new BN(185_844_305_400_574) : new BN(1_000_000_000).mul(new BN(web3.LAMPORTS_PER_SOL))
                ),
            solAccumulatedDepositAmount: null,
            solWithdrawalable: true,
            solWithdrawalNormalReserveRateBPS: this.isDevnet ? 5 : 5,
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
                            return true;
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
                withdrawalNormalReserveRateBPS: this.isDevnet ? 5 : 5,
                withdrawalNormalReserveMaxAmount: new BN(MAX_CAPACITY),
                tokenRebalancingAmount: null as BN | null,
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
                        case "jito1":
                            return new BN(this.isDevnet ? 0 : 1);
                        case "jito2":
                            return new BN(this.isDevnet ? 1 : 2);
                        default:
                            throw `invalid sol allocation weight for ${symbol}`;
                    }
                })(),
                solAllocationCapacityAmount: (() => {
                    switch (symbol) {
                        case "jito1":
                            return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                        case "jito2":
                            return new BN(this.isDevnet ? MAX_CAPACITY : MAX_CAPACITY);
                        default:
                            throw `invalid sol allocation cap for ${symbol}`;
                    }
                })(),
            })),
        };
    }

    public async runFundManagerAddSupportedTokens(symbol: keyof typeof this.supportedTokenMetadata) {
        const token = this.supportedTokenMetadata[symbol];
        const {event, error} = await this.run({
            instructions: [
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.fragSOLFundReserveSupportedTokenAccount(symbol as any),
                    this.knownAddress.fragSOLFundReserveAccount,
                    token.mint,
                    token.program,
                ),
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.fragSOLFundTreasurySupportedTokenAccount(symbol as any),
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
            this.getFragSOLSupportedTokenReserveAccount(tokenSymbol),
        ]);
        logger.notice(`operator donated: ${this.lamportsToX(amount, supportedToken.decimals, tokenSymbol)} (${this.lamportsToX(fragSOLFund.oneReceiptTokenAsSol, fragSOLFund.receiptTokenDecimals, 'SOL/fragSOL')})`.padEnd(LOG_PAD_LARGE), operatorSupportedTokenAddress.toString());

        return {
            event,
            error,
            fragSOLFund,
            fragSOLFundSupportedTokenAccount,
        };
    }

    public async runUserRequestWithdrawal(user: web3.Keypair, amount: BN, supported_token_mint: web3.PublicKey|null = null) {
        const {event, error} = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragSOLFundAndRewardAccounts(user)),
                this.program.methods
                    .userRequestWithdrawal(amount, supported_token_mint)
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
            `requested withdrawal: ${this.lamportsToFragSOL(event.userRequestedWithdrawalFromFund.requestedReceiptTokenAmount)} -> ${supported_token_mint ?? 'SOL'} #${event.userRequestedWithdrawalFromFund.requestId.toString()}/${event.userRequestedWithdrawalFromFund.batchId.toString()}`.padEnd(LOG_PAD_LARGE),
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
        tokenMint: web3.PublicKey | null = null,
    ) {
        const {event, error} = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragSOLFundAndRewardAccounts(user)),
                this.program.methods
                    .userCancelWithdrawalRequest(requestId, tokenMint)
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
        // const fundWithdrawalBatchAccount = this.knownAddress.fragSOLFundWithdrawalBatch(request.supportedTokenMint, request.batchId);

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
                                // fundWithdrawalBatchAccount,
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
                                // fundWithdrawalBatchAccount,
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
                ...Object.keys(this.supportedTokenMetadata).map(
                    symbol => this.getFragSOLSupportedTokenReserveAccount(symbol as keyof typeof this.supportedTokenMetadata)
                        .then(a => [a.mint, new BN(a.amount.toString())])
                ),
                ...Object.keys(this.restakingVaultMetadata).map(
                    symbol => this.getFragSOLRestakingVaultReceiptTokenReserveAccount(symbol as keyof typeof this.restakingVaultMetadata)
                        .then(a => [a.mint, new BN(a.amount.toString())])
                ),
            ]).then(([fund, fragSOLLocked, sol, nSOL, ...tokens]) => {
                console.log('fund asset state:', {
                    receiptToken: {
                        oneTokenAsSOL: fund.oneReceiptTokenAsSol,
                        supplyAmount: fund.receiptTokenSupplyAmount,
                        lockedAmount: fragSOLLocked,
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
                                total: fund.normalizedToken.operationReservedAmount,
                            },
                            ...Object.fromEntries(fund.supportedTokens.slice(0, fund.numSupportedTokens).map(supported => {
                                return [supported.mint, {
                                    reserved: supported.token.operationReservedAmount,
                                    receivable: supported.token.operationReceivableAmount,
                                    withdrawable: supported.token.withdrawalUserReservedAmount,
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
