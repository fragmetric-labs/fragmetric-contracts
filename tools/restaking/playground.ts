import * as anchor from "@coral-xyz/anchor";
import * as splTokenMetadata from "@solana/spl-token-metadata";
import * as spl from "@solana/spl-token";
import {AnchorPlayground, AnchorPlaygroundConfig, getLogger} from "../lib";
import {Restaking} from '../../target/types/restaking';
import {getKeychain, KEYCHAIN_KEYS} from "./keychain";
import {fragSOLTokenMintKeypair} from "../../tests/restaking/1_initialize";
const { logger, LOG_PAD_SMALL, LOG_PAD_LARGE } = getLogger('restaking');

export class RestakingPlayground extends AnchorPlayground<Restaking, KEYCHAIN_KEYS> {
    // The term "local" in this context doesn't necessarily refer to the localnet.
    // It can also be applied in devnet or mainnet environments while utilizing existing local keypairs.
    // and a different Anchor provider. This allows for flexibility in testing across various networks.
    public static local(provider?: anchor.AnchorProvider) {
        return getKeychain('local')
            .then(keychain => {
                return new RestakingPlayground({
                    keychain,
                    provider: new anchor.AnchorProvider(
                        provider?.connection ?? new anchor.web3.Connection('http://0.0.0.0:8899'),
                        new anchor.Wallet(keychain.wallet),
                    ),
                })
            });
    }

    public static devnet(args: Pick<AnchorPlaygroundConfig<Restaking, any>, 'provider'>) {
        return getKeychain('devnet')
            .then(keychain =>
                new RestakingPlayground({
                    keychain,
                    provider: new anchor.AnchorProvider(
                        new anchor.web3.Connection(anchor.web3.clusterApiUrl('devnet')),
                        new anchor.Wallet(keychain.wallet),
                    ),
                }),
            );
    }

    public static mainnet(args: Pick<AnchorPlaygroundConfig<Restaking, any>, 'provider'>) {
        return getKeychain('mainnet')
            .then(keychain =>
                new RestakingPlayground({
                    keychain,
                    provider: new anchor.AnchorProvider(
                        new anchor.web3.Connection(anchor.web3.clusterApiUrl('mainnet-beta')),
                        new anchor.Wallet(keychain.wallet),
                    ),
                }),
            );
    }

    private constructor(args: Pick<AnchorPlaygroundConfig<Restaking, any>, 'provider'|'keychain'>) {
        super({
            provider: args.provider,
            keychain: args.keychain,
            idl: require('../../target/idl/restaking.json') as Restaking,
        });


    }

    public async tryAirdropToMockAccounts() {
        if (!this.isMaybeLocalnet) return;
        await Promise.all(
            [
                this.tryAirdrop(this.keychain.wallet.publicKey),
                this.tryAirdrop(this.keychain.getPublicKey('MOCK_USER1')),
                this.tryAirdrop(this.keychain.getPublicKey('MOCK_USER2')),
                this.tryAirdrop(this.keychain.getPublicKey('MOCK_USER3')),
            ]
        );
    }

    public get knownAddress() {
        if (this._knownAddress) return this._knownAddress;
        return this._knownAddress = this._getKnownAddress();
    }
    private _knownAddress: ReturnType<typeof this._getKnownAddress>;
    private _getKnownAddress() {
        const fragSOLTokenMint = this.getConstantAsPublicKey('fragsolMintAddress');
        const fragSOLTokenMintBuf = fragSOLTokenMint.toBuffer();
        const [fragSOLTokenLock] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("receipt_token_lock"), fragSOLTokenMintBuf],
            this.programId
        );
        const [fragSOLFund] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("fund"), fragSOLTokenMintBuf],
            this.programId
        );
        const [fragSOLReward] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("reward"), fragSOLTokenMintBuf],
            this.programId
        );
        const [fragSOLTokenLockAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("receipt_token_lock_authority"), fragSOLTokenMintBuf],
            this.programId
        );
        const [fragSOLTokenMintAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("receipt_token_mint_authority"), fragSOLTokenMintBuf],
            this.programId
        );
        const [fragSOLSupportedTokenAuthority_bSOL] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("supported_token_authority"), fragSOLTokenMintBuf, this.supportedTokenMetadata.bSOL.mint.toBuffer()],
            this.programId
        );
        const [fragSOLSupportedTokenAccount_bSOL] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("supported_token_account"), fragSOLTokenMintBuf, this.supportedTokenMetadata.bSOL.mint.toBuffer()],
            this.programId
        );
        const [fragSOLSupportedTokenAuthority_mSOL] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("supported_token_authority"), fragSOLTokenMintBuf, this.supportedTokenMetadata.mSOL.mint.toBuffer()],
            this.programId
        );
        const [fragSOLSupportedTokenAccount_mSOL] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("supported_token_account"), fragSOLTokenMintBuf, this.supportedTokenMetadata.mSOL.mint.toBuffer()],
            this.programId
        );
        let fragSOLSupportedTokenAuthority_jitoSOL: anchor.web3.PublicKey | undefined;
        let fragSOLSupportedTokenAccount_jitoSOL: anchor.web3.PublicKey | undefined;
        if (this.supportedTokenMetadata.jitoSOL) {
            [fragSOLSupportedTokenAuthority_jitoSOL] = anchor.web3.PublicKey.findProgramAddressSync(
                [Buffer.from("supported_token_authority"), fragSOLTokenMintBuf, this.supportedTokenMetadata.jitoSOL.mint.toBuffer()],
                this.programId
            );
            [fragSOLSupportedTokenAccount_jitoSOL] = anchor.web3.PublicKey.findProgramAddressSync(
                [Buffer.from("supported_token_account"), fragSOLTokenMintBuf, this.supportedTokenMetadata.jitoSOL.mint.toBuffer()],
                this.programId
            );
        }
        return {
            fragSOLTokenMint,
            fragSOLTokenLock,
            fragSOLFund,
            fragSOLReward,
            fragSOLTokenLockAuthority,
            fragSOLTokenMintAuthority,
            fragSOLSupportedTokenAuthority_bSOL,
            fragSOLSupportedTokenAccount_bSOL,
            fragSOLSupportedTokenAuthority_mSOL,
            fragSOLSupportedTokenAccount_mSOL,
            fragSOLSupportedTokenAuthority_jitoSOL,
            fragSOLSupportedTokenAccount_jitoSOL,
        };
    }

    public get supportedTokenMetadata() {
        if (this._supportedTokenMetadata) return this._supportedTokenMetadata;
        return this._supportedTokenMetadata = this._getSupportedTokenMetadata();
    }
    private _supportedTokenMetadata: ReturnType<typeof this._getSupportedTokenMetadata>;
    public _getSupportedTokenMetadata() {
        if (this.isMaybeDevnet) {
            return {
                bSOL: {
                    mint: this.getConstantAsPublicKey('devnetBsolMintAddress'),
                    program: spl.TOKEN_PROGRAM_ID,
                    pricingSourceAddress: this.getConstantAsPublicKey('devnetBsolStakePoolAddress'),
                    pricingSource: {
                        splStakePool: {
                            address: this.getConstantAsPublicKey('devnetBsolStakePoolAddress'),
                        },
                    },
                    capacity: new anchor.BN(1_000_000_000).mul(new anchor.BN(1_000_000_000)),
                },
                mSOL: {
                    mint: this.getConstantAsPublicKey('devnetMsolMintAddress'),
                    program: spl.TOKEN_PROGRAM_ID,
                    pricingSourceAddress: this.getConstantAsPublicKey('devnetMsolStakePoolAddress'),
                    pricingSource: {
                        marinadeStakePool: {
                            address: this.getConstantAsPublicKey('devnetMsolStakePoolAddress'),
                        },
                    },
                    capacity: new anchor.BN(1_000_000_000).mul(new anchor.BN(1_000_000_000)),
                },
            };
        } else {
            // for 'localnet', it would be cloned from mainnet
            return {
                bSOL: {
                    mint: this.getConstantAsPublicKey('mainnetBsolMintAddress'),
                    program: spl.TOKEN_PROGRAM_ID,
                    pricingSourceAddress: this.getConstantAsPublicKey('mainnetBsolStakePoolAddress'),
                    pricingSource: {
                        splStakePool: {
                            address: this.getConstantAsPublicKey('mainnetBsolStakePoolAddress'),
                        },
                    },
                    capacity: new anchor.BN(1_000_000_000).mul(new anchor.BN(1_000_000_000)),
                },
                jitoSOL: {
                    mint: this.getConstantAsPublicKey('mainnetJitosolMintAddress'),
                    program: spl.TOKEN_PROGRAM_ID,
                    pricingSourceAddress: this.getConstantAsPublicKey('mainnetJitosolStakePoolAddress'),
                    pricingSource: {
                        splStakePool: {
                            address: this.getConstantAsPublicKey('mainnetJitosolStakePoolAddress'),
                        },
                    },
                    capacity: new anchor.BN(1_000_000_000).mul(new anchor.BN(1_000_000_000)),
                },
                mSOL: {
                    mint: this.getConstantAsPublicKey('mainnetMsolMintAddress'),
                    program: spl.TOKEN_PROGRAM_ID,
                    pricingSourceAddress: this.getConstantAsPublicKey('mainnetMsolStakePoolAddress'),
                    pricingSource: {
                        marinadeStakePool: {
                            address: this.getConstantAsPublicKey('mainnetMsolStakePoolAddress'),
                        },
                    },
                    capacity: new anchor.BN(1_000_000_000).mul(new anchor.BN(1_000_000_000)),
                },
            };
        }
    };

    public get pricingSourceAccounts() {
        if (this._pricingSourceAccounts) return this._pricingSourceAccounts;
        return this._pricingSourceAccounts = this.getPricingSourceAccounts();
    };
    private _pricingSourceAccounts: ReturnType<typeof this.getPricingSourceAccounts>;
    public getPricingSourceAccounts(): anchor.web3.AccountMeta[] {
        return Object.values(this.supportedTokenMetadata).map(v => {
            return {
                pubkey: v.pricingSourceAddress,
                isSigner: false,
                isWritable: false,
            };
        })
    }

    public async runInitializeFragSOLTokenMint() {
        const metadata: splTokenMetadata.TokenMetadata = {
            mint: this.keychain.getPublicKey('FRAGSOL_MINT'),
            name: "Fragmetric Restaked SOL",
            symbol: "fragSOL",
            uri: "https://quicknode.quicknode-ipfs.com/ipfs/Qme3xQUAKmtQHVu1hihKeBHuDW35zFPYfZdV6avEW6yRq1",
            additionalMetadata: [["description", "fragSOL is Solana's first native LRT that provides optimized restaking rewards."]],
            updateAuthority: this.keychain.getPublicKey('ADMIN'),
        };
        const fileForMetadataURI = JSON.stringify({
            name: metadata.name,
            symbol: metadata.symbol,
            description: metadata.additionalMetadata[0][1],
            image: "https://quicknode.quicknode-ipfs.com/ipfs/QmayYcry2mJGHmcYMn1mqiqxR9kkQXtE3uBEzR9y84vQVL",
            // attributes: [],
        }, null, 0);
        logger.debug(`fragSOL metadata file:\n> ${metadata.uri}\n> ${fileForMetadataURI}`);

        const decimals = 9;
        const mintInitialSize = spl.getMintLen([spl.ExtensionType.TransferHook, spl.ExtensionType.MetadataPointer]);
        const mintMetadataExtensionSize = (spl.TYPE_SIZE + spl.LENGTH_SIZE) + splTokenMetadata.pack(metadata).length;
        const mintTotalSize = mintInitialSize + mintMetadataExtensionSize;
        const lamports = await this.connection.getMinimumBalanceForRentExemption(mintTotalSize);

        await this.run({
            instructions: [
                anchor.web3.SystemProgram.createAccount({
                    fromPubkey: this.wallet.publicKey,
                    newAccountPubkey: this.knownAddress.fragSOLTokenMint,
                    lamports: lamports,
                    space: mintInitialSize,
                    programId: spl.TOKEN_2022_PROGRAM_ID,
                }),
                spl.createInitializeTransferHookInstruction(
                    this.knownAddress.fragSOLTokenMint,
                    this.keychain.getPublicKey('ADMIN'),
                    this.programId,
                    spl.TOKEN_2022_PROGRAM_ID,
                ),
                spl.createInitializeMetadataPointerInstruction(
                    this.knownAddress.fragSOLTokenMint,
                    this.keychain.getPublicKey('ADMIN'),
                    this.knownAddress.fragSOLTokenMint,
                    spl.TOKEN_2022_PROGRAM_ID,
                ),
                spl.createInitializeMintInstruction(
                    this.knownAddress.fragSOLTokenMint,
                    decimals,
                    this.keychain.getPublicKey('ADMIN'),
                    null, // freeze authority to be null
                    spl.TOKEN_2022_PROGRAM_ID,
                ),
                splTokenMetadata.createInitializeInstruction({
                    programId: spl.TOKEN_2022_PROGRAM_ID,
                    mint: this.knownAddress.fragSOLTokenMint,
                    metadata: this.knownAddress.fragSOLTokenMint,
                    name: metadata.name,
                    symbol: metadata.symbol,
                    uri: metadata.uri,
                    mintAuthority: this.keychain.getPublicKey('ADMIN'),
                    updateAuthority: metadata.updateAuthority,
                }),
                ...metadata.additionalMetadata
                    .map(([field, value]) =>
                        splTokenMetadata.createUpdateFieldInstruction({
                            programId: spl.TOKEN_2022_PROGRAM_ID,
                            metadata: this.knownAddress.fragSOLTokenMint,
                            updateAuthority: metadata.updateAuthority,
                            field,
                            value,
                        }),
                    ),
            ],
            signerNames: ['ADMIN', 'FRAGSOL_MINT'],
        });
        const fragSOLMint = await spl.getMint(
            this.connection,
            this.knownAddress.fragSOLTokenMint,
            undefined,
            spl.TOKEN_2022_PROGRAM_ID,
        );
        logger.info('fragSOL token mint created with extensions'.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLTokenMint.toString());
        return { fragSOLMint };
    }

    public async runUpdateFragSOLRewardAccount(batchSize = 35) {
        const currentVersion = await this.account.rewardAccount
            .fetch(this.knownAddress.fragSOLReward)
            .then(a => a.dataVersion)
            .catch(err => 0);

        const targetVersion = parseInt(this.getConstant('rewardAccountCurrentVersion'));
        const instructions = [
            ...(currentVersion == 0 ? [
                    this.program.methods
                        .adminInitializeRewardAccounts()
                        .accounts({ payer: this.wallet.publicKey })
                        .instruction()
            ] : []),
            ...new Array(targetVersion - currentVersion)
                .fill(null)
                .map((_, index, arr) =>
                    this.program.methods
                        .adminUpdateRewardAccountsIfNeeded(null, index == arr.length - 1)
                        .accounts({ payer: this.wallet.publicKey })
                        .instruction()
                ),
        ];
        if (instructions.length > 0) {
            for (let i=0; i<instructions.length/batchSize; i++) {
                const batchedInstructions = [];
                for (let j=i*batchSize; j<instructions.length && batchedInstructions.length < batchSize; j++) {
                    batchedInstructions.push(instructions[j]);
                }
                logger.debug(`running batched instructions`.padEnd(LOG_PAD_LARGE), `${i*batchSize + batchedInstructions.length}/${instructions.length}`);
                await this.run({
                    instructions: batchedInstructions,
                    signerNames: ['ADMIN'],
                });
            }
        }

        const fragSOLRewardAccount = await this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward);
        logger.info(`updated reward account version from=${currentVersion}, to=${fragSOLRewardAccount.dataVersion}, target=${targetVersion}`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLReward.toString());

        return { fragSOLRewardAccount };
    }

    public async runInitializeFragSOLFundAndRewardAccounts() {
        await this.run({
            instructions: [
                this.program.methods
                    .adminInitializeFundAccounts()
                    .accounts({payer: this.wallet.publicKey})
                    .instruction(),
            ],
            signerNames: ['ADMIN'],
        });
        const fragSOLFundAccount = await this.account.fundAccount.fetch(this.knownAddress.fragSOLFund);
        logger.info('fragSOL fund account created'.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLFund.toString());

        const { fragSOLRewardAccount } = await this.runUpdateFragSOLRewardAccount();

        await this.run({
            instructions: [
                this.program.methods
                    .adminInitializeReceiptTokenMintAuthorityAndExtraAccountMetaList()
                    .accounts({ payer: this.wallet.publicKey })
                    .instruction(),
                this.program.methods
                    .adminUpdateReceiptTokenMintExtraAccountMetaList()
                    .accounts({ payer: this.wallet.publicKey })
                    .instruction(),
            ],
            signerNames: ['ADMIN'],
        });
        const fragSOLMint = await spl.getMint(
            this.connection,
            this.knownAddress.fragSOLTokenMint,
            undefined,
            spl.TOKEN_2022_PROGRAM_ID,
        );
        logger.info(`transferred fragSOL mint authority to the PDA`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLTokenMintAuthority.toString());

        return {
            fragSOLFundAccount,
            fragSOLRewardAccount,
            fragSOLMint,
        };
    }

    public async runInitializeFragSOLFundConfiguration() {
        await this.run({
            instructions: [
                this.program.methods
                    .fundManagerUpdateSolCapacityAmount(new anchor.BN(1_000_000_000 * 10000))
                    .instruction(),
                this.program.methods
                    .fundManagerUpdateSolWithdrawalFeeRate(10) // 1 fee rate = 1bps = 0.01%
                    .instruction(),
                this.program.methods
                    .fundManagerUpdateWithdrawalEnabledFlag(true)
                    .instruction(),
                this.program.methods
                    .fundManagerUpdateBatchProcessingThreshold(
                        new anchor.BN(0), // batchProcessingThresholdAmount
                        new anchor.BN(0), // batchProcessingThresholdDuration
                    )
                    .instruction(),
                ...Object.values(this.supportedTokenMetadata).map(v => {
                    return this.program.methods
                        .fundManagerAddSupportedToken(
                            v.capacity,
                            v.pricingSource,
                        )
                        .accounts({
                            supportedTokenMint: v.mint,
                            supportedTokenProgram: v.program,
                        })
                        .remainingAccounts(this.getPricingSourceAccounts())
                        .instruction();
                }),
            ],
            signerNames: ['FUND_MANAGER'],
        });

        logger.info(`configured fragSOL supported tokens`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLFund.toString());
        const fragSOLFund = await this.account.fundAccount.fetch(this.knownAddress.fragSOLFund);
        return { fragSOLFund };
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
                type: { point: { decimals: 4 } },
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

    public async runInitializeFragSOLRewardConfiguration() {
        await this.run({
            instructions: [
                ...this.rewardPoolsMetadata.map(v => {
                    return this.program.methods
                        .fundManagerAddRewardPool(
                            v.name,
                            v.holderId,
                            v.customAccrualRateEnabled,
                        )
                        .accounts({
                            receiptTokenMint: this.knownAddress.fragSOLTokenMint,
                        })
                        .instruction();
                }),
                ...this.rewardsMetadata.map(v => {
                    return this.program.methods
                        .fundManagerAddReward(
                            v.name,
                            v.description,
                            v.type,
                        )
                        .accounts({
                            rewardTokenMint: v.tokenMint ?? this.programId,
                            rewardTokenProgram: v.tokenProgram ?? this.programId,
                        })
                        .instruction();
                }),
            ],
            signerNames: ['FUND_MANAGER'],
        });

        logger.info(`configured fragSOL reward pools and reward`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLReward.toString());
        const fragSOLReward = await this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward);
        return { fragSOLReward };
    }

    public static binToString(buf: Uint8Array | number[]) {
        const codes = [];
        for (let v of buf) {
            if (v == 0) break;
            codes.push(v);
        }
        return String.fromCharCode.apply(null, codes)
    }

    public readonly binToString = RestakingPlayground.binToString;
}