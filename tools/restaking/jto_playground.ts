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

const MAX_CAPACITY = "18_000_000_000_000_000_000".replace(/_/g, '');

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
            // () => this.runAdminInitializeNSOLTokenMint(), // 4
            // () => this.runAdminInitializeNormalizedTokenPoolAccounts(), // 5
            // () => this.runFundManagerInitializeNormalizeTokenPoolSupportedTokens(), // 6
            () => this.runFundManagerInitializeRewardPools(), // 7
            () => this.runFundManagerSettleReward({ poolName: "bonus", rewardName: "fPoint", amount: new BN(0) }), // 8
            () => this.runFundManagerInitializeFundSupportedTokens(), // 9
            // () => this.runFundManagerInitializeFundNormalizedToken(), // 10
            () => this.runFundManagerInitializeFundJitoRestakingVault(), // 11
            () => this.runFundManagerUpdateFundConfigurations(), // 12
            () => this.runOperatorUpdateFundPrices(), // 13
            // () => this.runOperatorUpdateNormalizedTokenPoolPrices(), // 14
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

        // prepare update
        const addresses = (Object.values(this.knownAddress)
            .filter(address => typeof address != 'function').flat() as web3.PublicKey[])
            .filter(address => !existingAddresses.has(address.toString()));

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

        // // nSOL
        // const nSOLTokenMint = this.getConstantAsPublicKey("fragsolNormalizedTokenMintAddress");
        // const nSOLTokenMintBuf = nSOLTokenMint.toBuffer();

        // fragJTO jito VRT
        const fragJTOJitoVRTMint = this.getConstantAsPublicKey('fragjtoJitoVaultReceiptTokenMintAddress');

        // fragJTO fund & ATAs
        const [fragJTOFund] = web3.PublicKey.findProgramAddressSync([Buffer.from("fund"), fragJTOTokenMintBuf], this.programId);
        const [fragJTOFundReserveAccount] = web3.PublicKey.findProgramAddressSync([Buffer.from("fund_reserve"), fragJTOTokenMintBuf], this.programId);
        const [fragJTOFundTreasuryAccount] = web3.PublicKey.findProgramAddressSync([Buffer.from("fund_treasury"), fragJTOTokenMintBuf], this.programId);
        const fragJTOFundTreasurySupportedTokenAccount = (symbol: keyof typeof this.supportedTokenMetadata) =>
            spl.getAssociatedTokenAddressSync(this.supportedTokenMetadata[symbol].mint, fragJTOFundTreasuryAccount, true, this.supportedTokenMetadata[symbol].program);
        const fragJTOFundTreasurySupportedTokenAccounts = Object.keys(this.supportedTokenMetadata).reduce((obj, symbol) => ({
            [`fragJTOFundTreasurySupportedTokenAccount_${symbol}`]: fragJTOFundTreasurySupportedTokenAccount(symbol as any),
            ...obj,
        }), {} as { string: web3.PublicKey });

        const fragJTOFundReceiptTokenLockAccount = spl.getAssociatedTokenAddressSync(
            fragJTOTokenMint,
            fragJTOFund,
            true,
            spl.TOKEN_2022_PROGRAM_ID,
        );
        const fragJTOSupportedTokenAccount = (symbol: keyof typeof this.supportedTokenMetadata) =>
            spl.getAssociatedTokenAddressSync(this.supportedTokenMetadata[symbol].mint, fragJTOFund, true, this.supportedTokenMetadata[symbol].program);
        const fragJTOSupportedTokenAccounts = Object.keys(this.supportedTokenMetadata).reduce((obj, symbol) => ({
            [`fragJTOFundReservedSupportedTokenAccount_${symbol}`]: fragJTOSupportedTokenAccount(symbol as any),
            ...obj,
        }), {} as { string: web3.PublicKey });

        // const fragJTOFundNSOLAccount = spl.getAssociatedTokenAddressSync(
        //     nSOLTokenMint,
        //     fragJTOFund,
        //     true,
        //     spl.TOKEN_PROGRAM_ID,
        //     spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        // );
        const fragJTOFundJitoVRTAccount = spl.getAssociatedTokenAddressSync(
            fragJTOJitoVRTMint,
            fragJTOFund,
            true,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        const fragJTOUserFund = (user: web3.PublicKey) => web3.PublicKey.findProgramAddressSync([Buffer.from("user_fund"), fragJTOTokenMintBuf, user.toBuffer()], this.programId)[0];
        const fragJTOUserTokenAccount = (user: web3.PublicKey) => spl.getAssociatedTokenAddressSync(fragJTOTokenMint, user, false, spl.TOKEN_2022_PROGRAM_ID);
        const userSupportedTokenAccount = (user: web3.PublicKey, symbol: keyof typeof this.supportedTokenMetadata) =>
            spl.getAssociatedTokenAddressSync(this.supportedTokenMetadata[symbol].mint, user, false, this.supportedTokenMetadata[symbol].program);

        const fragJTOFundWithdrawalBatch = (supportedTokenMint: web3.PublicKey|null, batchId: BN) => web3.PublicKey.findProgramAddressSync([Buffer.from("withdrawal_batch"), fragJTOTokenMintBuf, (supportedTokenMint || web3.PublicKey.default).toBuffer(), batchId.toBuffer('le', 8)], this.programId)[0];

        // reward
        const [fragJTOReward] = web3.PublicKey.findProgramAddressSync([Buffer.from("reward"), fragJTOTokenMintBuf], this.programId);
        const fragJTOUserReward = (user: web3.PublicKey) => web3.PublicKey.findProgramAddressSync([Buffer.from("user_reward"), fragJTOTokenMintBuf, user.toBuffer()], this.programId)[0];

        // // NTP
        // const [nSOLTokenPool] = web3.PublicKey.findProgramAddressSync([Buffer.from("nt_pool"), nSOLTokenMintBuf], this.programId);
        // const nSOLSupportedTokenLockAccount = (symbol: keyof typeof this.supportedTokenMetadata) =>
        //     spl.getAssociatedTokenAddressSync(this.supportedTokenMetadata[symbol].mint, nSOLTokenPool, true, this.supportedTokenMetadata[symbol].program);
        // const nSOLSupportedTokenLockAccounts = Object.keys(this.supportedTokenMetadata).reduce((obj, symbol) => ({
        //     [`nSOLSupportedTokenLockAccount_${symbol}`]: nSOLSupportedTokenLockAccount(symbol as any),
        //     ...obj,
        // }), {} as { string: web3.PublicKey });

        // // staking
        // const fundStakeAccounts = [...Array(5).keys()].map((i) =>
        //     web3.PublicKey.findProgramAddressSync(
        //         [
        //             fragJTOFund.toBuffer(),
        //             this.supportedTokenMetadata.jitoSOL.pricingSourceAddress.toBuffer(),
        //             Buffer.from([i]),
        //         ],
        //         this.programId,
        //     )[0]
        // );
        // // console.log(`fundStakeAccounts:`, fundStakeAccounts);

        // Restaking
        const vaultBaseAccount1 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_base_account1"), fragJTOTokenMintBuf], this.programId)[0];
        const vaultBaseAccount2 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_base_account2"), fragJTOTokenMintBuf], this.programId)[0];
        // const vaultBaseAccount3 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_base_account3"), fragJTOTokenMintBuf], this.programId)[0];
        // const vaultBaseAccount4 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_base_account4"), fragJTOTokenMintBuf], this.programId)[0];
        // const vaultBaseAccount5 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_base_account5"), fragJTOTokenMintBuf], this.programId)[0];

        // jito
        const jitoVaultProgram = this.getConstantAsPublicKey('jitoVaultProgramId');
        const jitoVaultProgramFeeWallet = this.getConstantAsPublicKey('jitoVaultProgramFeeWallet');
        const jitoVaultConfig = this.getConstantAsPublicKey('jitoVaultConfigAddress');

        // fragJTO jito vault
        const fragJTOJitoVaultAccount = this.getConstantAsPublicKey('fragjtoJitoVaultAccountAddress');
        const fragJTOJitoVaultUpdateStateTracker = (slot: anchor.BN, epoch_length: anchor.BN) => {
            let ncn_epoch = slot.div(epoch_length).toBuffer('le', 8);
            return web3.PublicKey.findProgramAddressSync([Buffer.from('vault_update_state_tracker'), fragJTOJitoVaultAccount.toBuffer(), ncn_epoch], jitoVaultProgram)[0];
        };
        const fragJTOJitoVaultJTOAccount = spl.getAssociatedTokenAddressSync(
            this.supportedTokenMetadata['JTO'].mint,
            fragJTOJitoVaultAccount,
            true,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        // const fragJTOJitoVaultNSOLAccount = spl.getAssociatedTokenAddressSync(
        //     nSOLTokenMint,
        //     fragJTOJitoVaultAccount,
        //     true,
        //     spl.TOKEN_PROGRAM_ID,
        //     spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        // );
        const fragJTOJitoVaultWithdrawalTicketAccount1 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_staker_withdrawal_ticket"), fragJTOJitoVaultAccount.toBuffer(), vaultBaseAccount1.toBuffer()], jitoVaultProgram)[0];
        const fragJTOJitoVaultWithdrawalTicketTokenAccount1 = spl.getAssociatedTokenAddressSync(
            fragJTOJitoVRTMint,
            fragJTOJitoVaultWithdrawalTicketAccount1,
            true,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        )
        const fragJTOJitoVaultWithdrawalTicketAccount2 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_staker_withdrawal_ticket"), fragJTOJitoVaultAccount.toBuffer(), vaultBaseAccount2.toBuffer()], jitoVaultProgram)[0];
        const fragJTOJitoVaultWithdrawalTicketTokenAccount2 = spl.getAssociatedTokenAddressSync(
            fragJTOJitoVRTMint,
            fragJTOJitoVaultWithdrawalTicketAccount2,
            true,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        const fragJTOJitoVaultProgramFeeWalletTokenAccount = spl.getAssociatedTokenAddressSync(
            fragJTOJitoVRTMint,
            jitoVaultProgramFeeWallet,
            true,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        const fragJTOJitoVaultFeeWalletTokenAccount = spl.getAssociatedTokenAddressSync(
            fragJTOJitoVRTMint,
            this.keychain.getPublicKey('ADMIN'),
            false,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        const programRevenueAccount = new web3.PublicKey(this.getConstant('programRevenueAddress'));
        const programSupportedTokenRevenueAccount = (symbol: keyof typeof this.supportedTokenMetadata) =>
            spl.getAssociatedTokenAddressSync(this.supportedTokenMetadata[symbol].mint, programRevenueAccount, true, this.supportedTokenMetadata[symbol].program);
        const programSupportedTokenRevenueAccounts = Object.keys(this.supportedTokenMetadata).reduce((obj, symbol) => ({
            [`programSupportedTokenRevenueAccount_${symbol}`]: programSupportedTokenRevenueAccount(symbol as any),
            ...obj,
        }), {} as { string: web3.PublicKey });

        return {
            programEventAuthority,
            // nSOLTokenMint,
            // fragJTOFundNSOLAccount,
            // nSOLTokenPool,
            // nSOLSupportedTokenLockAccount,
            // ...nSOLSupportedTokenLockAccounts,
            fragJTOTokenMint,
            fragJTOFundReceiptTokenLockAccount,
            fragJTOFund,
            fragJTOFundReserveAccount,
            fragJTOFundTreasuryAccount,
            fragJTOFundTreasurySupportedTokenAccount,
            ...fragJTOFundTreasurySupportedTokenAccounts,
            fragJTOExtraAccountMetasAccount,
            fragJTOUserFund,
            fragJTOUserTokenAccount,
            fragJTOReward,
            fragJTOUserReward,
            fragJTOSupportedTokenAccount,
            ...fragJTOSupportedTokenAccounts,
            userSupportedTokenAccount,
            fragJTOFundWithdrawalBatch,
            // fundStakeAccounts,
            jitoVaultProgram,
            jitoVaultProgramFeeWallet,
            fragJTOJitoVaultProgramFeeWalletTokenAccount,
            jitoVaultConfig,
            fragJTOJitoVaultAccount,
            fragJTOJitoVRTMint,
            fragJTOJitoVaultFeeWalletTokenAccount,
            fragJTOFundJitoVRTAccount,
            fragJTOJitoVaultJTOAccount, // fragJTOJitoVaultNSOLAccount,
            fragJTOJitoVaultUpdateStateTracker,
            vaultBaseAccount1,
            fragJTOJitoVaultWithdrawalTicketAccount1,
            fragJTOJitoVaultWithdrawalTicketTokenAccount1,
            vaultBaseAccount2,
            fragJTOJitoVaultWithdrawalTicketAccount2,
            fragJTOJitoVaultWithdrawalTicketTokenAccount2,
            programRevenueAccount,
            programSupportedTokenRevenueAccount,
            ...programSupportedTokenRevenueAccounts,
        };
    }

    public readonly fragJTODecimals = 9;
    // public readonly nSOLDecimals = 9;

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
                jito1: {
                    vault: this.getConstantAsPublicKey("fragjtoJitoVaultAccountAddress"),
                    program: this.getConstantAsPublicKey("jitoVaultProgramId"),
                },
            };
        } else {
            // for 'localnet', it would be cloned from mainnet
            return {
                jito1: {
                    vault: this.getConstantAsPublicKey("fragjtoJitoVaultAccountAddress"),
                    program: this.getConstantAsPublicKey("jitoVaultProgramId"),
                },
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
                pubkey: this.knownAddress.fragJTOJitoVaultAccount,
                isSigner: false,
                isWritable: false,
            },
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

                const orcaDEXLiquidityPoolAddress: web3.PublicKey | null = token.pricingSource["orcaDexLiquidityPool"]?.address ?? null;
                if (orcaDEXLiquidityPoolAddress) {
                    return {
                        instructions: [
                            spl.createMintToInstruction(
                                token.mint,
                                ata.address,
                                this.keychain.getPublicKey('ALL_MINT_AUTHORITY'),
                                lamports.toNumber(),
                                [],
                                token.program,
                            ),
                        ],
                        signers: [
                            this.keychain.getKeypair('ALL_MINT_AUTHORITY')
                        ]
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

    public getUserFragJTOAccount(user: web3.PublicKey) {
        return spl.getAccount(
            // @ts-ignore
            this.connection,
            this.knownAddress.fragJTOUserTokenAccount(user),
            "confirmed",
            spl.TOKEN_2022_PROGRAM_ID
        );
    }

    public getFragJTOSupportedTokenTreasuryAccountBalance(symbol: keyof typeof this.supportedTokenMetadata) {
        return spl.getAccount(
            // @ts-ignore
            this.connection,
            this.knownAddress.fragJTOFundTreasurySupportedTokenAccount(symbol),
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

    public getFragJTOSupportedTokenAccount(symbol: keyof typeof this.supportedTokenMetadata) {
        return spl.getAccount(
            // @ts-ignore
            this.connection,
            this.knownAddress.fragJTOSupportedTokenAccount(symbol),
            "confirmed",
            this.supportedTokenMetadata[symbol].program
        );
    }

    public getFragJTOSupportedTokenAccountByMintAddress(mint: web3.PublicKey) {
        for (const [symbol, token] of Object.entries(this.supportedTokenMetadata)) {
            if (mint.toString() != token.mint.toString()) continue;
            return spl.getAccount(
                // @ts-ignore
                this.connection,
                this.knownAddress.fragJTOSupportedTokenAccount(symbol as any),
                "confirmed",
                token.program,
            );
        }
        throw new Error("fund supported token account not found")
    }

    public getFragJTOFundReceiptTokenLockAccount() {
        return spl.getAccount(
            // @ts-ignore
            this.connection,
            this.knownAddress.fragJTOFundReceiptTokenLockAccount,
            "confirmed",
            spl.TOKEN_2022_PROGRAM_ID
        );
    }

    public getUserFragJTOFundAccount(user: web3.PublicKey) {
        return this.account.userFundAccount.fetch(this.knownAddress.fragJTOUserFund(user));
    }

    public getUserFragJTORewardAccount(user: web3.PublicKey) {
        return this.account.userRewardAccount.fetch(this.knownAddress.fragJTOUserReward(user));
    }

    public getFragJTORewardAccount() {
        return this.account.rewardAccount.fetch(this.knownAddress.fragJTOReward);
    }

    public getFragJTOFundAccount() {
        return this.account.fundAccount.fetch(this.knownAddress.fragJTOFund, "confirmed");
    }

    public getFragJTOFundReserveAccountBalance() {
        return this.connection.getBalance(this.knownAddress.fragJTOFundReserveAccount, "confirmed")
            .then(v => new BN(v));
    }

    // public getNSOLTokenPoolAccount() {
    //     return this.account.normalizedTokenPoolAccount.fetch(this.knownAddress.nSOLTokenPool, "confirmed");
    // }

    // public getNSOLSupportedTokenLockAccountBalance(symbol: keyof typeof this.supportedTokenMetadata) {
    //     return this.connection.getTokenAccountBalance(this.knownAddress.nSOLSupportedTokenLockAccount(symbol), "confirmed")
    //         .then(v => new BN(v.value.amount));
    // }

    // public getFragSOLFundNSOLAccountBalance() {
    //     return this.connection.getTokenAccountBalance(this.knownAddress.fragJTOFundNSOLAccount)
    //         .then(v => new BN(v.value.amount));
    // }

    public getFragJTOJitoVaultJTOAccountBalance() {
        return this.connection.getTokenAccountBalance(this.knownAddress.fragJTOJitoVaultJTOAccount, "confirmed")
            .then(v => new BN(v.value.amount));
    }

    // public getNSOLTokenMint() {
    //     return spl.getMint(
    //         // @ts-ignore
    //         this.connection,
    //         this.knownAddress.nSOLTokenMint,
    //         "confirmed",
    //         spl.TOKEN_PROGRAM_ID
    //     );
    // }

    public getFragJTOTokenMint() {
        return spl.getMint(
            // @ts-ignore
            this.connection,
            this.knownAddress.fragJTOTokenMint,
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

    public async runAdminInitializeFragJTOTokenMint() {
        const metadata: splTokenMetadata.TokenMetadata = {
            mint: this.keychain.getPublicKey("FRAGJTO_MINT"),
            name: "Fragmetric Staked JTO",
            symbol: "fragJTO",
            uri: "https://quicknode.quicknode-ipfs.com/ipfs/Qmc6UWCMKwXrH6C6JEUNM2NnGQMWFBSpQSisjY3AwRtHFZ",
            additionalMetadata: [["description", `fragJTO is the staked Jito governance token that provides optimized restaking rewards.`]],
            updateAuthority: this.keychain.getPublicKey("ADMIN"),
        };
        const fileForMetadataURI = JSON.stringify(
            {
                name: metadata.name,
                symbol: metadata.symbol,
                description: metadata.additionalMetadata[0][1],
                image: "https://fragmetric-assets.s3.ap-northeast-2.amazonaws.com/fragJTO.png",
                // attributes: [],
            },
            null,
            0
        );
        logger.debug(`fragJTO metadata file:\n> ${metadata.uri}\n> ${fileForMetadataURI}`);

        const mintInitialSize = spl.getMintLen([spl.ExtensionType.TransferHook, spl.ExtensionType.MetadataPointer]);
        const mintMetadataExtensionSize = spl.TYPE_SIZE + spl.LENGTH_SIZE + splTokenMetadata.pack(metadata).length;
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
                    name: metadata.name,
                    symbol: metadata.symbol,
                    uri: metadata.uri,
                    mintAuthority: this.keychain.getPublicKey("ADMIN"),
                    updateAuthority: metadata.updateAuthority,
                }),
                ...metadata.additionalMetadata.map(([field, value]) =>
                    splTokenMetadata.createUpdateFieldInstruction({
                        programId: spl.TOKEN_2022_PROGRAM_ID,
                        metadata: this.knownAddress.fragJTOTokenMint,
                        updateAuthority: metadata.updateAuthority,
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

    // public async runAdminInitializeNSOLTokenMint(createMetadata = false) {
    //     const mintSize = spl.getMintLen([]);
    //     const lamports = await this.connection.getMinimumBalanceForRentExemption(mintSize);

    //     await this.run({
    //         instructions: [
    //             web3.SystemProgram.createAccount({
    //                 fromPubkey: this.wallet.publicKey,
    //                 newAccountPubkey: this.knownAddress.nSOLTokenMint,
    //                 lamports: lamports,
    //                 space: mintSize,
    //                 programId: spl.TOKEN_PROGRAM_ID,
    //             }),
    //             spl.createInitializeMintInstruction(
    //                 this.knownAddress.nSOLTokenMint,
    //                 this.nSOLDecimals,
    //                 this.keychain.getPublicKey("ADMIN"),
    //                 null, // freeze authority to be null
    //                 spl.TOKEN_PROGRAM_ID
    //             ),
    //         ],
    //         signerNames: ["FRAGSOL_NORMALIZED_TOKEN_MINT"],
    //     });

    //     if (this.isLocalnet) {
    //         const txSig = await this.connection.requestAirdrop(this.keychain.getKeypair("ADMIN").publicKey, 1_000_000_000)
    //         await this.connection.confirmTransaction(txSig, 'confirmed');
    //     }

    //     if (createMetadata) {
    //         const umiInstance = umi2.createUmi(this.connection.rpcEndpoint).use(mpl.mplTokenMetadata());
    //         const keypair = this.keychain.getKeypair('FRAGSOL_NORMALIZED_TOKEN_MINT');
    //         const umiKeypair = umiInstance.eddsa.createKeypairFromSecretKey(keypair.secretKey);
    //         const mint = umi.createSignerFromKeypair(umiInstance, umiKeypair);

    //         const authKeypair = umiInstance.eddsa.createKeypairFromSecretKey(this.keychain.getKeypair("ADMIN").secretKey);
    //         const authority = umi.createSignerFromKeypair(umiInstance, authKeypair);
    //         umiInstance.use(umi.signerIdentity(authority));

    //         await mpl.createV1(umiInstance, {
    //             mint,
    //             authority,
    //             name: "normalized Liquid Staked Solana",
    //             symbol: "nSOL",
    //             decimals: 9,
    //             uri: "https://quicknode.quicknode-ipfs.com/ipfs/QmR5pP6Zo65XWCEXgixY8UtZjWbYPKmYHcyxzUq4p1KZt5",
    //             sellerFeeBasisPoints: umi.percentAmount(0),
    //             tokenStandard: mpl.TokenStandard.Fungible,
    //         }).sendAndConfirm(umiInstance);

    //         const assets = await mpl.fetchAllDigitalAssetByUpdateAuthority(umiInstance, authority.publicKey);
    //         logger.notice("nSOL token mint metadata created".padEnd(LOG_PAD_LARGE), assets);
    //     }

    //     const nSOLMint = await spl.getMint(this.connection, this.knownAddress.nSOLTokenMint, "confirmed", spl.TOKEN_PROGRAM_ID);
    //     logger.notice("nSOL token mint created".padEnd(LOG_PAD_LARGE), this.knownAddress.nSOLTokenMint.toString());
    //     return {nSOLMint};
    // }

    public async runAdminUpdateTokenMetadata() {
        const fragJTOTokenMetadataAddress = this.knownAddress.fragJTOTokenMint;

        let tokenMetadata = await spl.getTokenMetadata(this.connection, fragJTOTokenMetadataAddress, undefined, spl.TOKEN_2022_PROGRAM_ID);
        logger.debug(`current token metadata:\n> ${JSON.stringify(tokenMetadata, null, 0)}`);

        const updatedFileForMetadataURI = JSON.stringify(
            {
                name: tokenMetadata.name,
                symbol: tokenMetadata.symbol,
                description: tokenMetadata.additionalMetadata[0][1],
                image: "https://fragmetric-assets.s3.ap-northeast-2.amazonaws.com/fragJTO.png",
                // attributes: [],
            },
            null,
            0
        );
        logger.debug(`fragJTO metadata file:\n> ${updatedFileForMetadataURI}`);

        const updatedMetadataUri = "https://quicknode.quicknode-ipfs.com/ipfs/Qmc6UWCMKwXrH6C6JEUNM2NnGQMWFBSpQSisjY3AwRtHFZ";
        const updatedMetadata = spl.updateTokenMetadata(tokenMetadata, splTokenMetadata.Field.Uri, updatedMetadataUri);
        logger.debug(`will update token metadata:\n> ${JSON.stringify(updatedMetadata, null, 0)}`);

        await this.run({
            instructions: [
                splTokenMetadata.createUpdateFieldInstruction({
                    programId: spl.TOKEN_2022_PROGRAM_ID,
                    metadata: this.knownAddress.fragJTOTokenMint,
                    updateAuthority: tokenMetadata.updateAuthority,
                    field: splTokenMetadata.Field.Uri,
                    value: updatedMetadataUri,
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

    // TODO: migration v0.3.2
    public async runFundManagerCloseFundAccount() {
        await this.run({
            instructions: [
                this.program.methods.fundManagerCloseFundAccount().instruction(),
            ],
            signerNames: ['FUND_MANAGER'],
        });

        logger.notice("fragJTO fund account closed".padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOFund);
    }

    // TODO: migration v0.3.3
    public async runFundManagerClearUserSOLWithdrawalRequests(
        user: web3.PublicKey,
        numExpectedRequestsLeft: number,
    ) {
        await this.run({
            instructions: [
                this.program.methods.fundManagerClearUserSolWithdrawalRequests(
                    user,
                    numExpectedRequestsLeft,
                )
                .instruction(),
            ],
            signerNames: ['FUND_MANAGER'],
        });

        const userFundAccount = await this.getUserFragJTOFundAccount(user);
        logger.notice("old SOL withdrawal requests cleared".padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOUserFund(user));

        return {userFundAccount};
    }

    // public async runAdminInitializeNormalizedTokenPoolAccounts() {
    //     await this.run({
    //         instructions: [
    //             this.program.methods.adminInitializeNormalizedTokenPoolAccount()
    //                 .accounts({
    //                     payer: this.wallet.publicKey,
    //                     normalizedTokenMint: this.knownAddress.nSOLTokenMint,
    //                 })
    //                 .instruction(),
    //         ],
    //         signerNames: ["ADMIN"],
    //     });
    //     const nSOLTokenPoolAccount = await this.getNSOLTokenPoolAccount();
    //     logger.notice("nSOL token pool account created".padEnd(LOG_PAD_LARGE), this.knownAddress.nSOLTokenPool.toString());

    //     return {nSOLTokenPoolAccount};
    // }

    // public async runAdminUpdateNormalizedTokenPoolAccounts() {
    //     await this.run({
    //         instructions: [
    //             this.program.methods.adminUpdateNormalizedTokenPoolAccountIfNeeded()
    //                 .accountsPartial({
    //                     payer: this.wallet.publicKey,
    //                     normalizedTokenMint: this.knownAddress.nSOLTokenMint,
    //                 })
    //                 .instruction(),
    //         ],
    //         signerNames: ["ADMIN"],
    //     });
    //     const nSOLTokenPoolAccount = await this.getNSOLTokenPoolAccount();
    //     logger.notice("nSOL token pool account updated".padEnd(LOG_PAD_LARGE), this.knownAddress.nSOLTokenPool.toString());

    //     return {nSOLTokenPoolAccount};
    // }

    // public async runFundManagerInitializeFundNormalizedToken() {
    //     await this.run({
    //         instructions: [
    //             spl.createAssociatedTokenAccountIdempotentInstruction(
    //                 this.wallet.publicKey,
    //                 this.knownAddress.fragJTOFundNSOLAccount,
    //                 this.knownAddress.fragJTOFund,
    //                 this.knownAddress.nSOLTokenMint,
    //             ),
    //             this.program.methods.fundManagerInitializeFundNormalizedToken()
    //                 .accountsPartial({
    //                     receiptTokenMint: this.knownAddress.fragJTOTokenMint,
    //                     normalizedTokenMint: this.knownAddress.nSOLTokenMint,
    //                 })
    //                 .remainingAccounts(this.pricingSourceAccounts)
    //                 .instruction(),
    //         ],
    //         signerNames: ["FUND_MANAGER"],
    //         events: ['fundManagerUpdatedFund'],
    //     });

    //     const fragJTOFundAccount = await this.getFragJTOFundAccount();
    //     logger.notice("set fragJTO fund normalized token".padEnd(LOG_PAD_LARGE), this.knownAddress.nSOLTokenMint.toString());

    //     return {fragJTOFundAccount};
    // }

    public async runFundManagerInitializeFundJitoRestakingVault() {
        await this.run({
            instructions: [
                // TODO v0.3/restaking: adjust authority of fee wallet
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.fragJTOJitoVaultFeeWalletTokenAccount,
                    this.keychain.getPublicKey('ADMIN'),
                    this.knownAddress.fragJTOJitoVRTMint,
                ),
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.fragJTOJitoVaultJTOAccount,
                    this.knownAddress.fragJTOJitoVaultAccount,
                    this.supportedTokenMetadata['JTO'].mint,
                ),
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.fragJTOFundJitoVRTAccount,
                    this.knownAddress.fragJTOFund,
                    this.knownAddress.fragJTOJitoVRTMint,
                ),
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.fragJTOJitoVaultProgramFeeWalletTokenAccount,
                    this.knownAddress.jitoVaultProgramFeeWallet,
                    this.knownAddress.fragJTOJitoVRTMint,
                ),
                this.program.methods.fundManagerInitializeFundJitoRestakingVault()
                    .accountsPartial({
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        vaultAccount: this.knownAddress.fragJTOJitoVaultAccount,
                        vaultReceiptTokenMint: this.knownAddress.fragJTOJitoVRTMint,
                        vaultSupportedTokenMint: this.supportedTokenMetadata['JTO'].mint,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signerNames: ["FUND_MANAGER"],
            events: ['fundManagerUpdatedFund'],
        });

        const [
            fragJTOFundJitoFeeVRTAccount,
            fragJTOJitoVaultJTOAccount,
            fragJTOFundJitoVRTAccount,
            fragJTOJitoVaultProgramFeeWalletTokenAccount,
            fragJTOFundAccount,
        ] = await Promise.all([
            spl.getAccount(this.connection, this.knownAddress.fragJTOJitoVaultFeeWalletTokenAccount, 'confirmed'),
            spl.getAccount(this.connection, this.supportedTokenMetadata['JTO'].mint, 'confirmed'),
            spl.getAccount(this.connection, this.knownAddress.fragJTOFundJitoVRTAccount, 'confirmed'),
            spl.getAccount(this.connection, this.knownAddress.fragJTOJitoVaultProgramFeeWalletTokenAccount, 'confirmed'),
            this.getFragJTOFundAccount(),
        ]);
        logger.notice("jito VRT fee account created".padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOJitoVaultFeeWalletTokenAccount.toString());
        logger.notice("jito JTO account created".padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOJitoVaultJTOAccount.toString());
        logger.notice("jito VRT account created".padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOFundJitoVRTAccount.toString());
        logger.notice("jito VRT account (of program fee wallet) created".padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOJitoVaultProgramFeeWalletTokenAccount.toString());

        return {fragJTOFundJitoVRTAccount, fragJTOJitoVaultJTOAccount, fragJTOFundJitoFeeVRTAccount, fragJTOJitoVaultProgramFeeWalletTokenAccount, fragJTOFundAccount};
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

    public async runFundManagerInitializeFundSupportedTokens() {
        const {event, error} = await this.run({
            instructions: Object.entries(this.supportedTokenMetadata).flatMap(([symbol, v]) => {
                return [
                    spl.createAssociatedTokenAccountIdempotentInstruction(
                        this.wallet.publicKey,
                        this.knownAddress.fragJTOSupportedTokenAccount(symbol as any),
                        this.knownAddress.fragJTOFund,
                        v.mint,
                        v.program,
                    ),
                    spl.createAssociatedTokenAccountIdempotentInstruction(
                        this.wallet.publicKey,
                        this.knownAddress.fragJTOFundTreasurySupportedTokenAccount(symbol as any),
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
            depositEnabled: true,
            withdrawalEnabled: this.isMainnet ? false : true,
            WithdrawalFeedRateBPS: this.isMainnet ? 10 : 10,
            withdrawalBatchThresholdSeconds: new BN(this.isMainnet ? 60 : 60), // seconds

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
                            return new BN(this.isMainnet ? 0 : 100_000).mul(new BN(10 ** (v.decimals - 3)));
                        default:
                            throw `invalid accumulated deposit cap for ${symbol}`;
                    }
                })(),
                tokenAccumulatedDepositAmount: null,
                withdrawable: this.isMainnet ? false : true,
                withdrawalNormalReserveRateBPS: this.isMainnet ? 100 : 0,
                withdrawalNormalReserveMaxAmount: new BN(this.isMainnet ? 40_000 : 100).mul(new BN(10 ** v.decimals)),
                tokenRebalancingAmount: null,
                solAllocationWeight: (() => {
                    switch (symbol) {
                        case "JTO":
                            return new BN(this.isMainnet ? 1 : 1);
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
                        case "jito1":
                            return new BN(this.isMainnet ? 1 : 1);
                        default:
                            throw `invalid sol allocation weight for ${symbol}`;
                    }
                })(),
                solAllocationCapacityAmount: (() => {
                    switch (symbol) {
                        case "jito1":
                            return new BN(this.isMainnet ? MAX_CAPACITY : MAX_CAPACITY);
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
                    this.knownAddress.fragJTOSupportedTokenAccount(symbol as any),
                    this.knownAddress.fragJTOFund,
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
                    config.withdrawalEnabled,
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

        logger.notice(`updated fragJTO fund configuration`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragJTOFund.toString());
        const fragJTOFund = await this.account.fundAccount.fetch(this.knownAddress.fragJTOFund);
        return {event, error, fragJTOFund};
    }

    // public async runFundManagerInitializeNormalizeTokenPoolSupportedTokens() {
    //     await this.run({
    //         instructions: Object.entries(this.supportedTokenMetadata).flatMap(([symbol, v]) => {
    //             return [
    //                 spl.createAssociatedTokenAccountIdempotentInstruction(
    //                     this.wallet.publicKey,
    //                     this.knownAddress.nSOLSupportedTokenLockAccount(symbol as any),
    //                     this.knownAddress.nSOLTokenPool,
    //                     v.mint,
    //                     v.program,
    //                 ),
    //                 this.program.methods
    //                     .fundManagerAddNormalizedTokenPoolSupportedToken(
    //                         v.pricingSource,
    //                     )
    //                     .accountsPartial({
    //                         normalizedTokenMint: this.knownAddress.nSOLTokenMint,
    //                         supportedTokenMint: v.mint,
    //                         supportedTokenProgram: v.program,
    //                     })
    //                     .remainingAccounts(this.pricingSourceAccounts)
    //                     .instruction(),
    //             ];
    //         }),
    //         signerNames: ["FUND_MANAGER"],
    //     });

    //     logger.notice(`configured nSOL supported tokens`.padEnd(LOG_PAD_LARGE), this.knownAddress.nSOLTokenPool.toString());
    //     const nSOLTokenPool = await this.account.normalizedTokenPoolAccount.fetch(this.knownAddress.nSOLTokenPool);
    //     return {nSOLTokenPool};
    // }

    // public async runFundManagerAddNormalizeTokenPoolSupportedToken(symbol: keyof typeof this.supportedTokenMetadata) {
    //     const token = this.supportedTokenMetadata[symbol];
    //     await this.run({
    //         instructions: [
    //             spl.createAssociatedTokenAccountIdempotentInstruction(
    //                 this.wallet.publicKey,
    //                 this.knownAddress.nSOLSupportedTokenLockAccount(symbol as any),
    //                 this.knownAddress.nSOLTokenPool,
    //                 token.mint,
    //                 token.program,
    //             ),
    //             this.program.methods
    //                 .fundManagerAddNormalizedTokenPoolSupportedToken(
    //                     token.pricingSource,
    //                 )
    //                 .accountsPartial({
    //                     normalizedTokenMint: this.knownAddress.nSOLTokenMint,
    //                     supportedTokenMint: token.mint,
    //                     supportedTokenProgram: token.program,
    //                 })
    //                 .remainingAccounts(this.pricingSourceAccounts)
    //                 .instruction(),
    //         ],
    //         signerNames: ["FUND_MANAGER"],
    //     });

    //     logger.notice(`added nSOL supported tokens`.padEnd(LOG_PAD_LARGE), this.knownAddress.nSOLTokenPool.toString());
    //     const nSOLTokenPool = await this.account.normalizedTokenPoolAccount.fetch(this.knownAddress.nSOLTokenPool);
    //     return {nSOLTokenPool};
    // }

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
                            receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        })
                        .instruction();
                }),
                ...this.rewardsMetadata.map((v) => {
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
        const fragJTOUserRewardAddress = this.knownAddress.fragJTOUserReward(user.publicKey);
        const fragJTOUserFundAddress = this.knownAddress.fragJTOUserFund(user.publicKey);
        const currentRewardVersion = await this.account.userRewardAccount
            .fetch(fragJTOUserRewardAddress)
            .then((a) => a.dataVersion)
            .catch((err) => 0);
        const currentFundVersion = await this.account.userFundAccount
            .fetch(fragJTOUserFundAddress)
            .then((a) => a.dataVersion)
            .catch((err) => 0);

        const targetRewardVersion = parseInt(this.getConstant("userRewardAccountCurrentVersion"));
        return [
            spl.createAssociatedTokenAccountIdempotentInstruction(
                user.publicKey,
                this.knownAddress.fragJTOUserTokenAccount(user.publicKey),
                user.publicKey,
                this.knownAddress.fragJTOTokenMint,
                spl.TOKEN_2022_PROGRAM_ID,
            ),
            ...(currentFundVersion == 0
                ? [
                    this.program.methods.userInitializeFundAccount()
                        .accounts({
                            user: user.publicKey,
                            receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        })
                        .instruction(),
                ]
                : [
                    this.program.methods.userUpdateFundAccountIfNeeded()
                        .accountsPartial({
                            user: user.publicKey,
                            receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                        })
                        .instruction(),
                ]),
            ...(currentRewardVersion == 0 ? [
                this.program.methods.userInitializeRewardAccount()
                    .accountsPartial({
                        user: user.publicKey,
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                    })
                    .instruction(),
                ]
                : [
                    ...new Array(targetRewardVersion - currentRewardVersion).fill(null).map((_, index, arr) =>
                        this.program.methods
                            .userUpdateRewardAccountIfNeeded(null)
                            .accountsPartial({
                                user: user.publicKey,
                                receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                            })
                            .instruction(),
                    ),
                ]),
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
                ...(await this.getInstructionsToUpdateUserFragJTOFundAndRewardAccounts(user)),
                ...depositMetadataInstruction,
                this.program.methods
                    .userDepositSol(amount, depositMetadata)
                    .accountsPartial({
                        user: user.publicKey,
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signers: [user],
            events: ["userDepositedToFund"],
        });

        const [fragJTOFund, fragJTOFundReserveAccountBalance, fragJTOReward, fragJTOUserFund, fragJTOUserReward, fragJTOUserTokenAccount] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragJTOFund),
            this.getFragJTOFundReserveAccountBalance(),
            this.account.rewardAccount.fetch(this.knownAddress.fragJTOReward),
            this.account.userFundAccount.fetch(this.knownAddress.fragJTOUserFund(user.publicKey)),
            this.account.userRewardAccount.fetch(this.knownAddress.fragJTOUserReward(user.publicKey)),
            this.getUserFragJTOAccount(user.publicKey),
        ]);
        logger.notice(`deposited: ${this.lamportsToSOL(amount)} (${this.lamportsToX(fragJTOFund.oneReceiptTokenAsSol, fragJTOFund.receiptTokenDecimals, 'SOL/fragJTO')})`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());
        logger.info(`user fragJTO balance: ${this.lamportsToFragJTO(new BN(fragJTOUserTokenAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());

        return {
            event,
            error,
            fragJTOFund,
            fragJTOFundReserveAccountBalance,
            fragJTOReward,
            fragJTOUserFund,
            fragJTOUserReward,
            fragJTOUserTokenAccount
        };
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
                        receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signers: [operator],
            events: ["operatorDonatedToFund"],
        });

        const [fragJTOFund, fragJTOFundReserveAccountBalance] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragJTOFund),
            this.getFragJTOFundReserveAccountBalance(),
        ]);
        logger.notice(`operator donated: ${this.lamportsToSOL(amount)} (${this.lamportsToX(fragJTOFund.oneReceiptTokenAsSol, fragJTOFund.receiptTokenDecimals, 'SOL/fragJTO')})`.padEnd(LOG_PAD_LARGE), operator.publicKey.toString());

        return {
            event,
            error,
            fragJTOFund,
            fragJTOFundReserveAccountBalance,
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
            this.getFragJTOSupportedTokenAccount(tokenSymbol),
        ]);
        logger.notice(`operator donated: ${this.lamportsToX(amount, supportedToken.decimals, tokenSymbol)} (${this.lamportsToX(fragJTOFund.oneReceiptTokenAsSol, fragJTOFund.receiptTokenDecimals, 'SOL/fragJTO')})`.padEnd(LOG_PAD_LARGE), operatorSupportedTokenAddress.toString());

        return {
            event,
            error,
            fragJTOFund,
            fragJTOFundSupportedTokenAccount,
        };
    }

    public async runUserRequestWithdrawal(user: web3.Keypair, amount: BN, supported_token_mint: web3.PublicKey|null = null) {
        const {event, error} = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragJTOFundAndRewardAccounts(user)),
                this.program.methods
                    .userRequestWithdrawal(amount, supported_token_mint)
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
            `requested withdrawal: ${this.lamportsToFragJTO(event.userRequestedWithdrawalFromFund.requestedReceiptTokenAmount)} -> ${supported_token_mint ?? 'SOL'} #${event.userRequestedWithdrawalFromFund.requestId.toString()}/${event.userRequestedWithdrawalFromFund.batchId.toString()}`.padEnd(LOG_PAD_LARGE),
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

    public async runUserCancelWithdrawalRequest(user: web3.Keypair, requestId: BN) {
        const {event, error} = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragJTOFundAndRewardAccounts(user)),
                this.program.methods
                    .userCancelWithdrawalRequest(requestId)
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

    public async runOperatorUpdateFundPrices(operator: web3.Keypair = this.wallet) {
        const {event, error} = await this.run({
            instructions: [
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

    // public async runOperatorUpdateNormalizedTokenPoolPrices(operator: web3.Keypair = this.wallet) {
    //     const {event, error} = await this.run({
    //         instructions: [
    //             this.program.methods
    //                 .operatorUpdateNormalizedTokenPoolPrices()
    //                 .accountsPartial({
    //                     operator: operator.publicKey,
    //                     normalizedTokenMint: this.knownAddress.nSOLTokenMint,
    //                 })
    //                 .remainingAccounts(this.pricingSourceAccounts)
    //                 .instruction(),
    //         ],
    //         signers: [operator],
    //         events: ["operatorUpdatedNormalizedTokenPoolPrices"],
    //     });

    //     const [nSOLTokenPoolAccount] = await Promise.all([
    //         this.getNSOLTokenPoolAccount(),
    //     ]);
    //     logger.notice(`operator updated normalized token pool prices: ${this.lamportsToSOL(nSOLTokenPoolAccount.oneNormalizedTokenAsSol)}/nSOL`.padEnd(LOG_PAD_LARGE), operator.publicKey.toString());

    //     return {
    //         event,
    //         error,
    //         nSOLTokenPoolAccount,
    //     };
    // }

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

        const [fragJTOFund, fragJTOFundReserveAccountBalance, fragJTOReward, fragJTOLockAccount] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragJTOFund),
            this.getFragJTOFundReserveAccountBalance(),
            this.account.rewardAccount.fetch(this.knownAddress.fragJTOReward),
            this.getFragJTOFundReceiptTokenLockAccount(),
        ]);
        logger.info(`operator processed withdrawal batches up to #${fragJTOFund.sol.withdrawalLastProcessedBatchId.toString()}`.padEnd(LOG_PAD_LARGE), operator.publicKey.toString());

        return {event, error, fragJTOFund, fragJTOFundReserveAccountBalance, fragJTOReward, fragJTOLockAccount};
    }

    public async runUserWithdraw(user: web3.Keypair, requestId: BN) {
        const request = await this.getUserFragJTOFundAccount(user.publicKey)
            .then(userFundAccount => userFundAccount.withdrawalRequests.find(req => req.requestId.eq(requestId)));
        if (!request) {
            throw "request not found";
        }
        const userSupportedTokenAccount = request.supportedTokenMint ? spl.getAssociatedTokenAddressSync(request.supportedTokenMint, user.publicKey, true, request.supportedTokenProgram) : null;
        // const fundWithdrawalBatchAccount = this.knownAddress.fragJTOFundWithdrawalBatch(request.supportedTokenMint, request.batchId);

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
                                receiptTokenMint: this.knownAddress.fragJTOTokenMint,
                                // fundWithdrawalBatchAccount,
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
        console.log(executedCommand[commandName][0], commandResult && commandResult[commandName][0]);

        return {
            event: tx.event,
            error: tx.error,
        };
    }
}
