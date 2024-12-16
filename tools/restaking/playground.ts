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
            () => this.runAdminInitializeFragSOLTokenMint(), // 0
            () => this.runAdminInitializeOrUpdateFundAccount(), // 1
            () => this.runAdminInitializeOrUpdateRewardAccount(), // 2
            () => this.runAdminInitializeFragSOLExtraAccountMetaList(), // 3
            () => this.runAdminInitializeNSOLTokenMint(), // 4
            () => this.runAdminInitializeNormalizedTokenPoolAccounts(), // 5
            () => this.runFundManagerInitializeNormalizeTokenPoolSupportedTokens(), // 6
            () => this.runFundManagerInitializeRewardPools(), // 7
            () => this.runFundManagerSettleReward({ poolName: "bonus", rewardName: "fPoint", amount: new BN(0) }), // 8
            () => this.runFundManagerInitializeFundNormalizedToken(), // 9
            () => this.runFundManagerInitializeFundJitoRestakingVault(), // 10
            () => this.runFundManagerUpdateFundConfigurations(), // 11
        ];
    }

    public async getOrCreateKnownAddressLookupTable() {
        if (this._knownAddressLookupTableAddress) {
            return this._knownAddressLookupTableAddress;
        }

        const authority = this.keychain.getKeypair('ADMIN').publicKey;
        const payer = this.wallet.publicKey;
        const [createIx, lookupTable] = web3.AddressLookupTableProgram.createLookupTable({
            authority,
            payer,
            recentSlot: 0, // for fragSOL: 0
        });
        const existingLookupTable = await this.connection.getAccountInfo(lookupTable).catch(() => null);
        if (!existingLookupTable) {
            await this.run({
                instructions: [createIx],
                signerNames: ['ADMIN']
            });
            logger.notice('created a lookup table for known addresses:'.padEnd(LOG_PAD_LARGE), lookupTable.toString());
        }
        this._knownAddressLookupTableAddress = lookupTable;

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
        // fragSOL
        const fragSOLTokenMint = this.getConstantAsPublicKey("fragsolMintAddress");
        const fragSOLTokenMintBuf = fragSOLTokenMint.toBuffer();
        const fragSOLExtraAccountMetasAccount = spl.getExtraAccountMetaAddress(fragSOLTokenMint, this.programId);

        // nSOL
        const nSOLTokenMint = this.getConstantAsPublicKey("fragsolNormalizedTokenMintAddress");
        const nSOLTokenMintBuf = nSOLTokenMint.toBuffer();

        // fragSOL jito VRT
        const fragSOLJitoVRTMint = this.getConstantAsPublicKey('fragsolJitoVaultReceiptTokenMintAddress');

        // fragSOL fund & ATAs
        const [fragSOLFund] = web3.PublicKey.findProgramAddressSync([Buffer.from("fund"), fragSOLTokenMintBuf], this.programId);
        const [fragSOLFundReserveAccount] = web3.PublicKey.findProgramAddressSync([Buffer.from("fund_reserve"), fragSOLTokenMintBuf], this.programId);
        const [fragSOLFundTreasuryAccount] = web3.PublicKey.findProgramAddressSync([Buffer.from("fund_treasury"), fragSOLTokenMintBuf], this.programId);
        const fragSOLFundReceiptTokenLockAccount = spl.getAssociatedTokenAddressSync(
            fragSOLTokenMint,
            fragSOLFund,
            true,
            spl.TOKEN_2022_PROGRAM_ID,
        );
        const fragSOLSupportedTokenAccount = (symbol: keyof typeof this.supportedTokenMetadata) =>
            spl.getAssociatedTokenAddressSync(this.supportedTokenMetadata[symbol].mint, fragSOLFund, true, this.supportedTokenMetadata[symbol].program);
        const fragSOLSupportedTokenAccounts = Object.keys(this.supportedTokenMetadata).reduce((obj, symbol) => ({
            [`nSOLSupportedTokenLockAccount_${symbol}`]: fragSOLSupportedTokenAccount(symbol as any),
            ...obj,
        }), {} as { string: web3.PublicKey });

        const fragSOLFundNSOLAccount = spl.getAssociatedTokenAddressSync(
            nSOLTokenMint,
            fragSOLFund,
            true,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        );
        const fragSOLFundJitoVRTAccount = spl.getAssociatedTokenAddressSync(
            fragSOLJitoVRTMint,
            fragSOLFund,
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
        const nSOLSupportedTokenLockAccount = (symbol: keyof typeof this.supportedTokenMetadata) =>
            spl.getAssociatedTokenAddressSync(this.supportedTokenMetadata[symbol].mint, nSOLTokenPool, true, this.supportedTokenMetadata[symbol].program);
        const nSOLSupportedTokenLockAccounts = Object.keys(this.supportedTokenMetadata).reduce((obj, symbol) => ({
            [`nSOLSupportedTokenLockAccount_${symbol}`]: nSOLSupportedTokenLockAccount(symbol as any),
            ...obj,
        }), {} as { string: web3.PublicKey });

        // staking
        const fundStakeAccounts = [...Array(5).keys()].map((i) =>
            web3.PublicKey.findProgramAddressSync(
                [
                    fragSOLFund.toBuffer(),
                    this.supportedTokenMetadata.jitoSOL.pricingSourceAddress.toBuffer(),
                    Buffer.from([i]),
                ],
                this.programId,
            )[0]
        );
        // console.log(`fundStakeAccounts:`, fundStakeAccounts);

        // Restaking
        const vaultBaseAccount1 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_base_account1"), fragSOLTokenMintBuf], this.programId)[0];
        const vaultBaseAccount2 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_base_account2"), fragSOLTokenMintBuf], this.programId)[0];
        // const vaultBaseAccount3 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_base_account3"), fragSOLTokenMintBuf], this.programId)[0];
        // const vaultBaseAccount4 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_base_account4"), fragSOLTokenMintBuf], this.programId)[0];
        // const vaultBaseAccount5 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_base_account5"), fragSOLTokenMintBuf], this.programId)[0];

        // jito
        const jitoVaultProgram = this.getConstantAsPublicKey('jitoVaultProgramId');
        const jitoVaultProgramFeeWallet = this.getConstantAsPublicKey('jitoVaultProgramFeeWallet');
        const jitoVaultConfig = this.getConstantAsPublicKey('fragsolJitoVaultConfigAddress');

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
        const fragSOLJitoVaultWithdrawalTicketAccount1 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_staker_withdrawal_ticket"), fragSOLJitoVaultAccount.toBuffer(), vaultBaseAccount1.toBuffer()], jitoVaultProgram)[0];
        const fragSOLJitoVaultWithdrawalTicketTokenAccount1 = spl.getAssociatedTokenAddressSync(
            fragSOLJitoVRTMint,
            fragSOLJitoVaultWithdrawalTicketAccount1,
            true,
            spl.TOKEN_PROGRAM_ID,
            spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        )
        const fragSOLJitoVaultWithdrawalTicketAccount2 = web3.PublicKey.findProgramAddressSync([Buffer.from("vault_staker_withdrawal_ticket"), fragSOLJitoVaultAccount.toBuffer(), vaultBaseAccount2.toBuffer()], jitoVaultProgram)[0];
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

        return {
            nSOLTokenMint,
            fragSOLFundNSOLAccount,
            nSOLTokenPool,
            nSOLSupportedTokenLockAccount,
            ...nSOLSupportedTokenLockAccounts,
            fragSOLTokenMint,
            fragSOLFundReceiptTokenLockAccount,
            fragSOLFund,
            fragSOLFundReserveAccount,
            fragSOLFundTreasuryAccount,
            fragSOLExtraAccountMetasAccount,
            fragSOLUserFund,
            fragSOLUserTokenAccount,
            fragSOLReward,
            fragSOLUserReward,
            fragSOLSupportedTokenAccount,
            ...fragSOLSupportedTokenAccounts,
            userSupportedTokenAccount,
            fragSOLFundWithdrawalBatch,
            fundStakeAccounts,
            jitoVaultProgram,
            jitoVaultProgramFeeWallet,
            fragSOLJitoVaultProgramFeeWalletTokenAccount,
            jitoVaultConfig,
            fragSOLJitoVaultAccount,
            fragSOLJitoVRTMint,
            fragSOLJitoVaultFeeWalletTokenAccount,
            fragSOLFundJitoVRTAccount,
            fragSOLJitoVaultNSOLAccount,
            fragSOLJitoVaultUpdateStateTracker,
            vaultBaseAccount1,
            fragSOLJitoVaultWithdrawalTicketAccount1,
            fragSOLJitoVaultWithdrawalTicketTokenAccount1,
            vaultBaseAccount2,
            fragSOLJitoVaultWithdrawalTicketAccount2,
            fragSOLJitoVaultWithdrawalTicketTokenAccount2
        };
    }

    public readonly fragSOLDecimals = 9;
    public readonly nSOLDecimals = 9;

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
            };
        } else {
            // for 'localnet', it would be cloned from mainnet
            return {
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
                    vault: this.getConstantAsPublicKey("fragsolJitoVaultAccountAddress"),
                    program: this.getConstantAsPublicKey("jitoVaultProgramId"),
                },
            };
        } else {
            // for 'localnet', it would be cloned from mainnet
            return {
                jito1: {
                    vault: this.getConstantAsPublicKey("fragsolJitoVaultAccountAddress"),
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
                pubkey: this.knownAddress.nSOLTokenPool,
                isSigner: false,
                isWritable: false,
            },
            {
                pubkey: this.knownAddress.fragSOLJitoVaultAccount,
                isSigner: false,
                isWritable: false,
            },
        ];
    }

    public async tryAirdropSupportedTokens(account: web3.PublicKey, amount = 100) {
        await this.tryAirdrop(this.wallet.publicKey, amount * Object.keys(this.supportedTokenMetadata).length);

        const txData = await Promise.all(
            Object.values(this.supportedTokenMetadata).map(async (token) => {
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
                    const res = await splStakePool.depositSol(this.connection, splStakePoolAddress, this.wallet.publicKey, amount * web3.LAMPORTS_PER_SOL, ata.address);
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
                    const res = await marinadeStakePool.deposit(new BN(amount * web3.LAMPORTS_PER_SOL), {
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
            logger.debug(`${symbol} airdropped (+${amount}): ${this.lamportsToX(balance, token.decimals, symbol)}`.padEnd(LOG_PAD_LARGE), ata.address.toString());
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

    public getFragSOLSupportedTokenAccount(symbol: keyof typeof this.supportedTokenMetadata) {
        return spl.getAccount(
            // @ts-ignore
            this.connection,
            this.knownAddress.fragSOLSupportedTokenAccount(symbol),
            "confirmed",
            this.supportedTokenMetadata[symbol].program
        );
    }

    public getFragSOLSupportedTokenAccountByMintAddress(mint: web3.PublicKey) {
        for (const [symbol, token] of Object.entries(this.supportedTokenMetadata)) {
            if (mint.toString() != token.mint.toString()) continue;
            return spl.getAccount(
                // @ts-ignore
                this.connection,
                this.knownAddress.fragSOLSupportedTokenAccount(symbol as any),
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

    public getNSOLTokenPoolAccount() {
        return this.account.normalizedTokenPoolAccount.fetch(this.knownAddress.nSOLTokenPool, "confirmed");
    }

    public getNSOLSupportedTokenLockAccountBalance(symbol: keyof typeof this.supportedTokenMetadata) {
        return this.connection.getTokenAccountBalance(this.knownAddress.nSOLSupportedTokenLockAccount(symbol), "confirmed")
            .then(v => new BN(v.value.amount));
    }

    public getFragSOLFundNSOLAccountBalance() {
        return this.connection.getTokenAccountBalance(this.knownAddress.fragSOLFundNSOLAccount)
            .then(v => new BN(v.value.amount));
    }

    public getFragSOLJitoVaultNSOLAccountBalance() {
        return this.connection.getTokenAccountBalance(this.knownAddress.fragSOLJitoVaultNSOLAccount, "confirmed")
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
            ...(currentVersion == 0 ? [this.program.methods.adminInitializeRewardAccount().accounts({payer: this.wallet.publicKey}).instruction()] : []),
            ...new Array(targetVersion - currentVersion).fill(null).map((_, index, arr) => this.program.methods.adminUpdateRewardAccountIfNeeded(null).accounts({payer: this.wallet.publicKey}).instruction()),
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
            ...(currentVersion == 0 ? [this.program.methods.adminInitializeFundAccount().accounts({payer: this.wallet.publicKey}).instruction()] : []),
            ...new Array(targetVersion - currentVersion).fill(null).map((_, index, arr) => this.program.methods.adminUpdateFundAccountIfNeeded(null).accounts({payer: this.wallet.publicKey}).instruction()),
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
                    .accounts({payer: this.wallet.publicKey})
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
                    .accounts({payer: this.wallet.publicKey})
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
                    this.knownAddress.fragSOLFund,
                    this.knownAddress.nSOLTokenMint,
                ),
                this.program.methods.fundManagerInitializeFundNormalizedToken()
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

    public async runFundManagerInitializeFundJitoRestakingVault() {
        await this.run({
            instructions: [
                // TODO v0.3/restaking: adjust authority of fee wallet
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.fragSOLJitoVaultFeeWalletTokenAccount,
                    this.keychain.getPublicKey('ADMIN'),
                    this.knownAddress.fragSOLJitoVRTMint,
                ),
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.fragSOLJitoVaultNSOLAccount,
                    this.knownAddress.fragSOLJitoVaultAccount,
                    this.knownAddress.nSOLTokenMint,
                ),
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.fragSOLFundJitoVRTAccount,
                    this.knownAddress.fragSOLFund,
                    this.knownAddress.fragSOLJitoVRTMint,
                ),
                spl.createAssociatedTokenAccountIdempotentInstruction(
                    this.wallet.publicKey,
                    this.knownAddress.fragSOLJitoVaultProgramFeeWalletTokenAccount,
                    this.knownAddress.jitoVaultProgramFeeWallet,
                    this.knownAddress.fragSOLJitoVRTMint,
                ),
                this.program.methods.fundManagerInitializeFundJitoRestakingVault()
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signerNames: ["FUND_MANAGER"],
            events: ['fundManagerUpdatedFund'],
        });

        const [
            fragSOLFundJitoFeeVRTAccount,
            fragSOLJitoVaultNSOLAccount,
            fragSOLFundJitoVRTAccount,
            fragSOLJitoVaultProgramFeeWalletTokenAccount,
            fragSOLFundAccount,
        ] = await Promise.all([
            spl.getAccount(this.connection, this.knownAddress.fragSOLJitoVaultFeeWalletTokenAccount, 'confirmed'),
            spl.getAccount(this.connection, this.knownAddress.fragSOLJitoVaultNSOLAccount, 'confirmed'),
            spl.getAccount(this.connection, this.knownAddress.fragSOLFundJitoVRTAccount, 'confirmed'),
            spl.getAccount(this.connection, this.knownAddress.fragSOLJitoVaultProgramFeeWalletTokenAccount, 'confirmed'),
            this.getFragSOLFundAccount(),
        ]);
        logger.notice("jito VRT fee account created".padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLJitoVaultFeeWalletTokenAccount.toString());
        logger.notice("jito nSOL account created".padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLJitoVaultNSOLAccount.toString());
        logger.notice("jito VRT account created".padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLFundJitoVRTAccount.toString());
        logger.notice("jito VRT account (of program fee wallet) created".padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLJitoVaultProgramFeeWalletTokenAccount.toString());

        return {fragSOLFundJitoVRTAccount, fragSOLJitoVaultNSOLAccount, fragSOLFundJitoFeeVRTAccount, fragSOLJitoVaultProgramFeeWalletTokenAccount, fragSOLFundAccount};
    }

    public async runAdminInitializeFragSOLExtraAccountMetaList() {
        await this.run({
            instructions: [
                this.program.methods.adminInitializeExtraAccountMetaList().accounts({payer: this.wallet.publicKey}).instruction(),
                this.program.methods.adminUpdateExtraAccountMetaListIfNeeded().accounts({payer: this.wallet.publicKey}).instruction(),
            ],
            signerNames: ["ADMIN"],
        });
        const fragSOLExtraAccountMetasAccount = await this.connection.getAccountInfo(spl.getExtraAccountMetaAddress(this.knownAddress.fragSOLTokenMint, this.programId)).then((acc) => spl.getExtraAccountMetas(acc));
        logger.notice(`initialized fragSOL extra account meta list`.padEnd(LOG_PAD_LARGE));

        return {fragSOLExtraAccountMetasAccount};
    }

    public async runFundManagerInitializeFundSupportedTokens() {
        const {event, error} = await this.run({
            instructions: Object.entries(this.supportedTokenMetadata).flatMap(([symbol, v]) => {
                return [
                    spl.createAssociatedTokenAccountIdempotentInstruction(
                        this.wallet.publicKey,
                        this.knownAddress.fragSOLSupportedTokenAccount(symbol as any),
                        this.knownAddress.fragSOLFund,
                        v.mint,
                        v.program,
                    ),
                    this.program.methods
                        .fundManagerAddSupportedToken(v.pricingSource)
                        .accountsPartial({
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
            withdrawalEnabled: this.isMainnet ? false : true,
            WithdrawalFeedRateBPS: this.isMainnet ? 10 : 10,
            withdrawalBatchThresholdSeconds: new BN(this.isMainnet ? 60 : 60), // seconds

            solAccumulatedDepositCapacity: (this.isMainnet ? new BN(44_196_940) : new BN(1_000_000_000)).mul(new BN(web3.LAMPORTS_PER_SOL/1_000)),
            solAccumulatedDepositAmount:  null,
            solWithdrawalable: this.isMainnet ? true : true,
            solWithdrawalNormalReserveRateBPS: this.isMainnet ? 100 : 0,
            solWithdrawalNormalReserveMaxAmount: new BN(this.isMainnet ? 40_000 : 100).mul(new BN(web3.LAMPORTS_PER_SOL)),

            supportedTokens: Object.entries(this.supportedTokenMetadata).map(([symbol, v]) => ({
                tokenMint: v.mint,
                tokenAccumulatedDepositCapacity: (() => {
                    switch (symbol) {
                        case "bSOL":
                            return new BN(this.isMainnet ? 0 : 90_000).mul(new BN(10 ** (v.decimals - 3)));
                        case "jitoSOL":
                            return new BN(this.isMainnet ? 22_680_370 : 80_000).mul(new BN(10 ** (v.decimals - 3)));
                        case "mSOL":
                            return new BN(this.isMainnet ? 4_500_000 : 70_000).mul(new BN(10 ** (v.decimals - 3)));
                        case "BNSOL":
                            return new BN(this.isMainnet ? 2_617_170 : 60_000).mul(new BN(10 ** (v.decimals - 3)));
                        default:
                            throw `invalid accumulated deposit cap for ${symbol}`;
                    }
                })(),
                tokenAccumulatedDepositAmount : null,
                withdrawalable: this.isMainnet ? false : false,
                withdrawalNormalReserveRateBPS: this.isMainnet ? 100 : 0,
                withdrawalNormalReserveMaxAmount: new BN(this.isMainnet ? 40_000 : 100).mul(new BN(10 ** v.decimals)),
                tokenRebalancingAmount: null as BN | null,
                solAllocationWeight: (() => {
                    switch (symbol) {
                        case "bSOL":
                            return new BN(this.isMainnet ? 0 : 0);
                        case "jitoSOL":
                            return new BN(this.isMainnet ? 90 : 90);
                        case "mSOL":
                            return new BN(this.isMainnet ? 5 : 5);
                        case "BNSOL":
                            return new BN(this.isMainnet ? 5 : 5);
                        default:
                            throw `invalid sol allocation weight for ${symbol}`;
                    }
                })(),
                solAllocationCapacityAmount: (() => {
                    switch (symbol) {
                        case "bSOL":
                            return new BN(this.isMainnet ? MAX_CAPACITY : MAX_CAPACITY);
                        case "jitoSOL":
                            return new BN(this.isMainnet ? MAX_CAPACITY : MAX_CAPACITY);
                        case "mSOL":
                            return new BN(this.isMainnet ? MAX_CAPACITY : MAX_CAPACITY);
                        case "BNSOL":
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
                    this.knownAddress.fragSOLSupportedTokenAccount(symbol as any),
                    this.knownAddress.fragSOLFund,
                    token.mint,
                    token.program,
                ),
                this.methods
                    .fundManagerAddSupportedToken(token.pricingSource)
                    .accountsPartial({
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
                    config.withdrawalEnabled,
                    config.WithdrawalFeedRateBPS, // 1 fee rate = 1bps = 0.01%
                    config.withdrawalBatchThresholdSeconds,
                ).instruction(),
                this.program.methods.fundManagerUpdateSolStrategy(
                    config.solAccumulatedDepositCapacity,
                    config.solAccumulatedDepositAmount,
                    config.solWithdrawalable,
                    config.solWithdrawalNormalReserveRateBPS,
                    config.solWithdrawalNormalReserveMaxAmount,
                ).instruction(),
                ...config.supportedTokens.flatMap((v) => {
                    return [
                        this.program.methods.fundManagerUpdateSupportedTokenStrategy(
                            v.tokenMint,
                            v.tokenAccumulatedDepositCapacity,
                            v.tokenAccumulatedDepositAmount,
                            v.withdrawalable,
                            v.withdrawalNormalReserveRateBPS,
                            v.withdrawalNormalReserveMaxAmount,
                            v.tokenRebalancingAmount,
                            v.solAllocationWeight,
                            v.solAllocationCapacityAmount,
                        )
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
                        )
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
                        this.knownAddress.nSOLSupportedTokenLockAccount(symbol as any),
                        this.knownAddress.nSOLTokenPool,
                        v.mint,
                        v.program,
                    ),
                    this.program.methods
                        .fundManagerAddNormalizedTokenPoolSupportedToken(
                            v.pricingSource,
                        )
                        .accounts({
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
                    this.knownAddress.nSOLSupportedTokenLockAccount(symbol as any),
                    this.knownAddress.nSOLTokenPool,
                    token.mint,
                    token.program,
                ),
                this.program.methods
                    .fundManagerAddNormalizedTokenPoolSupportedToken(
                        token.pricingSource,
                    )
                    .accounts({
                        supportedTokenMint: token.mint,
                        supportedTokenProgram: token.program,
                    })
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
                        .accounts({
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
                    .accounts({
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
        const fragSOLUserRewardAddress = this.knownAddress.fragSOLUserReward(user.publicKey);
        const fragSOLUserFundAddress = this.knownAddress.fragSOLUserFund(user.publicKey);
        const currentRewardVersion = await this.account.userRewardAccount
            .fetch(fragSOLUserRewardAddress)
            .then((a) => a.dataVersion)
            .catch((err) => 0);
        const currentFundVersion = await this.account.userFundAccount
            .fetch(fragSOLUserFundAddress)
            .then((a) => a.dataVersion)
            .catch((err) => 0);

        const targetRewardVersion = parseInt(this.getConstant("userRewardAccountCurrentVersion"));
        return [
            ...(currentRewardVersion == 0 ? [
                this.program.methods.userInitializeRewardAccount()
                    .accounts({user: user.publicKey})
                    .instruction()
            ] : [
                // ...
            ]),
            ...new Array(targetRewardVersion - currentRewardVersion).fill(null).map((_, index, arr) =>
                this.program.methods
                    .userUpdateRewardAccountIfNeeded(null)
                    .accounts({user: user.publicKey})
                    .instruction()
            ),
            ...(currentFundVersion == 0
                ? [
                    spl.createAssociatedTokenAccountIdempotentInstruction(
                        user.publicKey,
                        this.knownAddress.fragSOLUserTokenAccount(user.publicKey),
                        user.publicKey,
                        this.knownAddress.fragSOLTokenMint,
                        spl.TOKEN_2022_PROGRAM_ID,
                    ),
                    this.program.methods.userInitializeFundAccount()
                        .accounts({user: user.publicKey})
                        .instruction(),
                ]
                : [
                    this.program.methods.userUpdateFundAccountIfNeeded()
                        .accountsPartial({user: user.publicKey})
                        .instruction(),
                ]),
        ];
    }

    public async runUserDepositSOL(user: web3.Keypair, amount: BN, depositMetadata?: IdlTypes<Restaking>["depositMetadata"], depositMetadataSigningKeypair?: web3.Keypair) {
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
        depositMetadata?: IdlTypes<Restaking>["depositMetadata"],
        depositMetadataSigningKeypair?: web3.Keypair
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

    public async runUserRequestWithdrawal(user: web3.Keypair, amount: BN, supported_token_mint: web3.PublicKey|null = null) {
        const {event, error} = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragSOLFundAndRewardAccounts(user)),
                this.program.methods
                    .userRequestWithdrawal(amount, supported_token_mint)
                    .accountsPartial({
                        user: user.publicKey,
                    })
                    .instruction(),
            ],
            signers: [user],
            events: ["userRequestedWithdrawalFromFund"],
        });

        logger.notice(
            `requested withdrawal: ${this.lamportsToFragSOL(amount)} -> ${supported_token_mint ?? 'SOL'} #${event.userRequestedWithdrawalFromFund.requestId.toString()}/${event.userRequestedWithdrawalFromFund.batchId.toString()}`.padEnd(LOG_PAD_LARGE),
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

    public async runUserCancelWithdrawalRequest(user: web3.Keypair, requestId: BN) {
        const {event, error} = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragSOLFundAndRewardAccounts(user)),
                this.program.methods
                    .userCancelWithdrawalRequest(requestId)
                    .accountsPartial({
                        user: user.publicKey,
                    })
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

    public async runOperatorProcessWithdrawalBatches(operator: web3.Keypair = this.keychain.getKeypair('FUND_MANAGER'), forced: boolean = false) {
        const {event: _event, error: _error} = await this.runOperatorRun({
            command: {
                enqueueWithdrawalBatch: {
                    0: {
                        forced: forced,
                    }
                }
            },
            requiredAccounts: [],
        }, operator);

        const {event, error} = await this.runOperatorRun({
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
        logger.info(`operator processed withdrawal batches up to #${fragSOLFund.solFlow.withdrawalLastProcessedBatchId.toString()}`.padEnd(LOG_PAD_LARGE), operator.publicKey.toString());

        return {event, error, fragSOLFund, fragSOLFundReserveAccountBalance, fragSOLReward, fragSOLLockAccount};
    }

    public async runUserWithdraw(user: web3.Keypair, requestId: BN) {
        const request = await this.getUserFragSOLFundAccount(user.publicKey)
            .then(userFundAccount => userFundAccount.withdrawalRequests.find(req => req.requestId.eq(requestId)));
        if (!request) {
            throw "request not found";
        }
        const fundWithdrawalBatchAccount = this.knownAddress.fragSOLFundWithdrawalBatch(request.supportedTokenMint, request.batchId);

        // TODO: branch based on request.supportedTokenMint
        const {event, error} = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragSOLFundAndRewardAccounts(user)),
                this.program.methods
                    .userWithdrawSol(requestId)
                    .accountsPartial({
                        user: user.publicKey,
                        fundWithdrawalBatchAccount,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
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
        logger.notice(`user withdrew: ${this.lamportsToSOL(event.userWithdrewFromFund.withdrawnAmount)} #${requestId.toString()}, (${this.lamportsToX(fragSOLFund.oneReceiptTokenAsSol, fragSOLFund.receiptTokenDecimals, 'SOL/fragSOL')})`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());

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
                    .accounts({
                        operator: operator.publicKey,
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

    public async runOperatorRun(resetCommand: Parameters<typeof this.program.methods.operatorRun>[0] = null, operator: web3.Keypair = this.keychain.getKeypair('FUND_MANAGER'), setComputeUnitLimitUnits?: number, setComputeUnitPriceMicroLamports?: number) {
        let txCount = 0;
        while (txCount < 100) {
            const {event, error} = await this.runOperatorRunSingle(operator, txCount == 0 ? resetCommand : null, setComputeUnitLimitUnits, setComputeUnitPriceMicroLamports);
            txCount++;
            if (txCount == 100 || event.operatorRanFund.nextOperationSequence == 0) {
                return {event, error}
            }
        }
    }

    private async runOperatorRunSingle(operator: web3.Keypair, resetCommand?: Parameters<typeof this.program.methods.operatorRun>[0], setComputeUnitLimitUnits: number = 800_000, setComputeUnitPriceMicroLamports?: number) {
        // prepare accounts according to the current state of operation.
        // - can contain 27/32 accounts with reserved four accounts and payer.
        // - order doesn't matter, no need to put duplicate.
        const requiredAccounts: Map<web3.PublicKey, web3.AccountMeta> = new Map();
        this.pricingSourceAccounts.forEach(accoutMeta => {
            requiredAccounts.set(accoutMeta.pubkey, accoutMeta);
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
        requiredAccounts.set(this.knownAddress.fragSOLTokenMint, {
            pubkey: this.knownAddress.fragSOLTokenMint,
            isWritable: true,
            isSigner: false,
        });
        requiredAccounts.set(this.knownAddress.fragSOLFund, {
            pubkey: this.knownAddress.fragSOLFund,
            isWritable: true,
            isSigner: false,
        });

        let fragSOLFund = await this.getFragSOLFundAccount();
        let nextOperationCommand = resetCommand ?? fragSOLFund.operation.nextCommand;
        let nextOperationSequence = resetCommand ? 0 : fragSOLFund.operation.nextSequence;
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
                    .operatorRun(resetCommand)
                    .accounts({
                        operator: operator.publicKey,
                    })
                    .remainingAccounts(Array.from(requiredAccounts.values()))
                    .instruction(),
            ],
            signers: [operator],
            events: ["operatorRanFund"],
            skipPreflight: true,
            // TODO: why is requestHeapFrameBytes not working?
            // requestHeapFrameBytes, : 64 * 1024,
            setComputeUnitLimitUnits,
            setComputeUnitPriceMicroLamports,
        });

        let executedCommand = tx.event.operatorRanFund.executedCommand;
        const commandName = Object.keys(executedCommand)[0];
        const commandArgs = executedCommand[commandName][0];
        logger.notice(`operator ran command#${nextOperationSequence}: ${commandName}`.padEnd(LOG_PAD_LARGE), JSON.stringify(commandArgs));

        return {
            event: tx.event,
            error: tx.error,
        };
    }
}
