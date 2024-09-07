import * as anchor from '@coral-xyz/anchor';
import { BN } from '@coral-xyz/anchor';
import * as web3 from '@solana/web3.js';
// @ts-ignore
import * as splTokenMetadata from '@solana/spl-token-metadata';
// @ts-ignore
import * as spl from '@solana/spl-token';
import {AnchorPlayground, AnchorPlaygroundConfig, getLogger} from '../lib';
import {Restaking} from '../../target/types/restaking';
import {getKeychain, KEYCHAIN_KEYS} from './keychain';
import {IdlTypes} from "@coral-xyz/anchor/dist/cjs/program/namespace/types";
import * as ed25519 from "ed25519";

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
                        provider?.connection ?? new web3.Connection('http://0.0.0.0:8899'),
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
                        new web3.Connection(web3.clusterApiUrl('devnet')),
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
                        new web3.Connection(web3.clusterApiUrl('mainnet-beta')),
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

    public get knownAddress() {
        if (this._knownAddress) return this._knownAddress;
        return this._knownAddress = this._getKnownAddress();
    }
    private _knownAddress: ReturnType<typeof this._getKnownAddress>;
    private _getKnownAddress() {
        const fragSOLTokenMint = this.getConstantAsPublicKey('fragsolMintAddress');
        const fragSOLTokenMintBuf = fragSOLTokenMint.toBuffer();
        const [fragSOLTokenLock] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from('receipt_token_lock'), fragSOLTokenMintBuf],
            this.programId
        );
        const fragSOLExtraAccountMetasAccount = spl.getExtraAccountMetaAddress(fragSOLTokenMint, this.programId);
        const [fragSOLFund] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from('fund'), fragSOLTokenMintBuf],
            this.programId
        );
        const fragSOLUserFund = (user: web3.PublicKey) => web3.PublicKey.findProgramAddressSync(
            [Buffer.from('user_fund'), fragSOLTokenMintBuf, user.toBuffer()],
            this.programId
        )[0];
        const [fragSOLReward] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from('reward'), fragSOLTokenMintBuf],
            this.programId
        );
        const fragSOLUserReward = (user: web3.PublicKey) => web3.PublicKey.findProgramAddressSync(
            [Buffer.from('user_reward'), fragSOLTokenMintBuf, user.toBuffer()],
            this.programId
        )[0];
        const fragSOLUserTokenAccount = (user: web3.PublicKey) => spl.getAssociatedTokenAddressSync(
            fragSOLTokenMint,
            user,
            false,
            spl.TOKEN_2022_PROGRAM_ID,
        );
        const [fragSOLTokenLockAuthority] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from('receipt_token_lock_authority'), fragSOLTokenMintBuf],
            this.programId
        );
        const [fragSOLTokenMintAuthority] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from('receipt_token_mint_authority'), fragSOLTokenMintBuf],
            this.programId
        );
        const fragSOLSupportedTokenAuthority = (symbol: keyof typeof this.supportedTokenMetadata) => web3.PublicKey.findProgramAddressSync(
            [Buffer.from('supported_token_authority'), fragSOLTokenMintBuf, this.supportedTokenMetadata[symbol].mint.toBuffer()],
            this.programId
        )[0];
        const fragSOLSupportedTokenAccount = (symbol: keyof typeof this.supportedTokenMetadata) => web3.PublicKey.findProgramAddressSync(
            [Buffer.from('supported_token_account'), fragSOLTokenMintBuf, this.supportedTokenMetadata[symbol].mint.toBuffer()],
            this.programId
        );
        const userSupportedTokenAccount = (user: web3.PublicKey, symbol: keyof typeof this.supportedTokenMetadata) => spl.getAssociatedTokenAddressSync(
            this.supportedTokenMetadata[symbol].mint,
            user,
            false,
            this.supportedTokenMetadata[symbol].program,
        );
        return {
            fragSOLTokenMint,
            fragSOLTokenLock,
            fragSOLFund,
            fragSOLExtraAccountMetasAccount,
            fragSOLUserFund,
            fragSOLUserTokenAccount,
            fragSOLReward,
            fragSOLUserReward,
            fragSOLTokenLockAuthority,
            fragSOLTokenMintAuthority,
            fragSOLSupportedTokenAuthority,
            fragSOLSupportedTokenAccount,
            userSupportedTokenAccount,
        };
    }

    public get supportedTokenMetadata() {
        if (this._supportedTokenMetadata) return this._supportedTokenMetadata;
        return this._supportedTokenMetadata = this._getSupportedTokenMetadata();
    }
    private _supportedTokenMetadata: ReturnType<typeof this._getSupportedTokenMetadata>;
    private _getSupportedTokenMetadata() {
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
                    capacity: new BN(10 ** 9).mul(new BN(10_000)),
                    decimals: 9,
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
                    capacity: new BN(10 ** 9).mul(new BN(10_000)),
                    decimals: 9,
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
                    capacity: new BN(10 ** 9).mul(this.isMaybeLocalnet ? new BN(1_000) : new BN(10_000)), // TODO: set number
                    decimals: 9,
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
                    capacity: new BN(10 ** 9).mul(this.isMaybeLocalnet ? new BN(1_000) : new BN(10_000)), // TODO: set number
                    decimals: 9,
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
                    capacity: new BN(10 ** 9).mul(this.isMaybeLocalnet ? new BN(1_000) : new BN(10_000)), // TODO: set number
                    decimals: 9,
                },
            };
        }
    };

    public get pricingSourceAccounts() {
        if (this._pricingSourceAccounts) return this._pricingSourceAccounts;
        return this._pricingSourceAccounts = this._getPricingSourceAccounts();
    }
    private _pricingSourceAccounts: ReturnType<typeof this._getPricingSourceAccounts>;
    private _getPricingSourceAccounts(): web3.AccountMeta[] {
        return Object.values(this.supportedTokenMetadata).map(v => {
            return {
                pubkey: v.pricingSourceAddress,
                isSigner: false,
                isWritable: false,
            };
        })
    }

    public async tryAirdropSupportedTokens(account: web3.PublicKey, amount = 100) {
        await this.run({
            instructions: [
                ...Object.values(this.supportedTokenMetadata).map(token => {
                    return spl.getOrCreateAssociatedTokenAccount(
                        this.connection,
                        this.wallet,
                        token.mint,
                        account,
                        false,
                        'confirmed',
                        {
                            skipPreflight: false,
                            commitment: 'confirmed',
                        },
                        token.program,
                    ).then(ata => spl.createMintToCheckedInstruction(
                        token.mint,
                        ata.address,
                        this.keychain.getPublicKey('MOCK_ALL_MINT_AUTHORITY'),
                        BigInt(amount * (10 ** token.decimals)),
                        token.decimals,
                        [],
                        token.program,
                    ));
                }),
            ],
            signerNames: ['MOCK_ALL_MINT_AUTHORITY'],
        });

        for (const [symbol, token] of Object.entries(this.supportedTokenMetadata)) {
            const ata = await this.getUserSupportedTokenAccount(account, symbol as any);
            const balance = new BN(ata.amount.toString());
            logger.debug(`${symbol} airdropped (+${amount}): ${this.lamportsToX(balance, token.decimals, symbol)}`.padEnd(LOG_PAD_LARGE), ata.address.toString());
        }
    }

    public getUserSupportedTokenAccount(user: web3.PublicKey, symbol: keyof typeof this.supportedTokenMetadata) {
        return spl.getAccount(
            this.connection,
            this.knownAddress.userSupportedTokenAccount(user, symbol),
            'confirmed',
            this.supportedTokenMetadata[symbol].program,
        );
    }

    public getUserFragSOLAccount(user: web3.PublicKey) {
        return spl.getAccount(
            this.connection,
            this.knownAddress.fragSOLUserTokenAccount(user),
            'confirmed',
            spl.TOKEN_2022_PROGRAM_ID,
        );
    }

    public getFragSOLLockAccount() {
        return spl.getAccount(
            this.connection,
            this.knownAddress.fragSOLTokenLock,
            'confirmed',
            spl.TOKEN_2022_PROGRAM_ID,
        );
    }

    public async skipSlots(signer: web3.Keypair, skip: number) {
        let currentSlot = await this.program.provider.connection.getSlot();
        logger.debug(`BEFORE skip slots, current slot: ${currentSlot}`);
    
        for (let i = 0; i < skip; i++) {
            await this.program.methods
                .emptyIx()
                .accounts({})
                .signers([signer])
                .rpc();
        }
    
        currentSlot = await this.program.provider.connection.getSlot();
        logger.debug(`AFTER skip slots, current slot: ${currentSlot}`);
    }

    public async runAdminInitializeTokenMint() {
        const metadata: splTokenMetadata.TokenMetadata = {
            mint: this.keychain.getPublicKey('FRAGSOL_MINT'),
            name: 'Fragmetric Restaked SOL',
            symbol: 'fragSOL',
            uri: 'https://quicknode.quicknode-ipfs.com/ipfs/Qme3xQUAKmtQHVu1hihKeBHuDW35zFPYfZdV6avEW6yRq1',
            additionalMetadata: [['description', `fragSOL is Solana's first native LRT that provides optimized restaking rewards.`]],
            updateAuthority: this.keychain.getPublicKey('ADMIN'),
        };
        const fileForMetadataURI = JSON.stringify({
            name: metadata.name,
            symbol: metadata.symbol,
            description: metadata.additionalMetadata[0][1],
            image: 'https://quicknode.quicknode-ipfs.com/ipfs/QmayYcry2mJGHmcYMn1mqiqxR9kkQXtE3uBEzR9y84vQVL',
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
                web3.SystemProgram.createAccount({
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
        logger.notice('fragSOL token mint created with extensions'.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLTokenMint.toString());
        return { fragSOLMint };
    }

    public async runAdminUpdateRewardAccounts(batchSize = 35) {
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
        logger.notice(`updated reward account version from=${currentVersion}, to=${fragSOLRewardAccount.dataVersion}, target=${targetVersion}`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLReward.toString());

        return { fragSOLRewardAccount };
    }

    public async runAdminInitializeFundAndRewardAccounts() {
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
        logger.notice('fragSOL fund account created'.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLFund.toString());

        const { fragSOLRewardAccount } = await this.runAdminUpdateRewardAccounts();

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
        const [
            fragSOLMint,
            fragSOLExtraAccountMetasAccount,
        ] = await Promise.all([
            spl.getMint(
                this.connection,
                this.knownAddress.fragSOLTokenMint,
                undefined,
                spl.TOKEN_2022_PROGRAM_ID,
            ),
            this.connection.getAccountInfo(spl.getExtraAccountMetaAddress(this.knownAddress.fragSOLTokenMint, this.programId))
                .then(acc => spl.getExtraAccountMetas(acc)),
        ]);
        logger.notice(`transferred fragSOL mint authority to the PDA`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLTokenMintAuthority.toString());

        return {
            fragSOLFundAccount,
            fragSOLRewardAccount,
            fragSOLExtraAccountMetasAccount,
            fragSOLMint,
        };
    }

    public async runFundManagerInitializeFundConfigurations() {
        const { event, error } = await this.run({
            instructions: [
                this.program.methods
                    .fundManagerUpdateSolCapacityAmount(new BN(10 ** 9).mul(this.isMaybeLocalnet ? new BN(1_000) : new BN(10_000))) // TODO: set number
                    .instruction(),
                this.program.methods
                    .fundManagerUpdateSolWithdrawalFeeRate(10) // 1 fee rate = 1bps = 0.01%
                    .instruction(),
                this.program.methods
                    .fundManagerUpdateWithdrawalEnabledFlag(true)
                    .instruction(),
                this.program.methods
                    .fundManagerUpdateBatchProcessingThreshold(
                        new BN(0), // batchProcessingThresholdAmount
                        new BN(10), // batchProcessingThresholdDuration (seconds)
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
                        .remainingAccounts(this.pricingSourceAccounts)
                        .instruction();
                }),
            ],
            signerNames: ['FUND_MANAGER'],
            events: ['fundManagerUpdatedFund'],
        });

        logger.notice(`configured fragSOL supported tokens`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLFund.toString());
        const fragSOLFund = await this.account.fundAccount.fetch(this.knownAddress.fragSOLFund);
        return { event, error, fragSOLFund };
    }

    public get rewardsMetadata() {
        return this._getRewardsMetadata;
    }
    private readonly _getRewardsMetadata = this.getRewardsMetadata();
    private getRewardsMetadata() {
        return [
            {
                name: 'fPoint',
                description: 'Airdrop point for fToken',
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
                name: 'base',
                holderId: null,
                customAccrualRateEnabled: false,
            },
            {
                name: 'bonus',
                holderId: null,
                customAccrualRateEnabled: true,
            },
        ];
    }

    public async runFundManagerInitializeRewardPools() {
        const { event, error } = await this.run({
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
            events: ['fundManagerUpdatedRewardPool'],
        });

        logger.notice(`configured fragSOL reward pools and reward`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLReward.toString());
        const fragSOLReward = await this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward);
        return { event, error, fragSOLReward };
    }

    public async runFundManagerSettleReward(args: {poolName: string, rewardName: string, amount: BN}) {
        let fragSOLReward = await this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward);
        let rewardPool = fragSOLReward.rewardPools1.find(r => this.binToString(r.name) == args.poolName);
        let reward = fragSOLReward.rewards1.find(r => this.binToString(r.name) == args.rewardName);

        const rewardTokenMint = this.binIsEmpty(reward.tokenMint.toBuffer()) ? this.programId : reward.tokenMint;
        const rewardTokenProgram = this.binIsEmpty(reward.tokenProgram.toBuffer()) ? this.programId : reward.tokenProgram;
        const { event, error } = await this.run({
            instructions: [
                this.program.methods
                    .fundManagerSettleReward(rewardPool.id, reward.id, args.amount)
                    .accounts({
                        rewardTokenMint,
                        rewardTokenProgram,
                    })
                    .instruction(),
            ],
            signerNames: ['FUND_MANAGER'],
            events: ['fundManagerUpdatedRewardPool'],
        });

        logger.notice(`settled fragSOL reward to pool=${rewardPool.id}/${args.poolName}, rewardId=${reward.id}/${args.rewardName}, amount=${args.amount.toString()} (decimals=${reward.decimals})`);
        fragSOLReward = await this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward);
        rewardPool = fragSOLReward.rewardPools1.find(r => this.binToString(r.name) == args.poolName);
        reward = fragSOLReward.rewards1.find(r => this.binToString(r.name) == args.rewardName);

        return { event, error, fragSOLReward, rewardPool, reward };
    }

    private async getInstructionsToUpdateUserFragSOLFundAndRewardAccounts(user: web3.Keypair) {
        const fragSOLUserRewardAddress = this.knownAddress.fragSOLUserReward(user.publicKey);
        const currentVersion = await this.account.userRewardAccount
            .fetch(fragSOLUserRewardAddress)
            .then(a => a.dataVersion)
            .catch(err => 0);

        const targetVersion = parseInt(this.getConstant('userRewardAccountCurrentVersion'));
        return [
            ...(currentVersion == 0 ? [
                this.program.methods
                    .userInitializeRewardAccounts()
                    .accounts({user: user.publicKey})
                    .instruction()
            ] : []),
            ...new Array(targetVersion - currentVersion)
                .fill(null)
                .map((_, index, arr) =>
                    this.program.methods
                        .userUpdateRewardAccountsIfNeeded(null, index == arr.length - 1)
                        .accounts({user: user.publicKey})
                        .instruction()
                ),
            this.program.methods
                .userUpdateFundAccountsIfNeeded()
                .accounts({
                    user: user.publicKey,
                })
                .instruction(),
        ];
    }

    public async runUserDepositSOL(user: web3.Keypair, amount: BN, depositMetadata?: IdlTypes<Restaking>['depositMetadata'], depositMetadataSigningKeypair?: web3.Keypair) {
        let depositMetadataInstruction: web3.TransactionInstruction[] = [];
        if (depositMetadata) {
            depositMetadataSigningKeypair = depositMetadataSigningKeypair ?? this.keychain.getKeypair('ADMIN');
            const message = this.program.coder.types.encode('depositMetadata', depositMetadata);
            const signature = ed25519.Sign(message, Buffer.from(depositMetadataSigningKeypair.secretKey));
            depositMetadataInstruction.push(
                web3.Ed25519Program.createInstructionWithPublicKey({
                    publicKey: depositMetadataSigningKeypair.publicKey.toBytes(),
                    message,
                    signature,
                }),
            );
        }
        const { event, error } = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragSOLFundAndRewardAccounts(user)),
                ...depositMetadataInstruction,
                this.program.methods
                    .userDepositSol(amount, depositMetadata)
                    .accounts({
                        user: user.publicKey,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signers: [user],
            events: ['userDepositedSolToFund', 'userUpdatedRewardPool']
        });

        logger.notice(`user deposited: ${this.lamportsToSOL(amount)}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());
        const [
            fragSOLFund,
            fragSOLFundBalance,
            fragSOLReward,
            fragSOLUserFund,
            fragSOLUserReward,
            fragSOLUserTokenAccount,
        ] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragSOLFund),
            this.connection.getBalance(this.knownAddress.fragSOLFund, 'confirmed').then(v => new BN(v)),
            this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward),
            this.account.userFundAccount.fetch(this.knownAddress.fragSOLUserFund(user.publicKey)),
            this.account.userRewardAccount.fetch(this.knownAddress.fragSOLUserReward(user.publicKey)),
            this.getUserFragSOLAccount(user.publicKey),
        ]);
        logger.info(`user fragSOL balance: ${this.lamportsToFragSOL(new BN(fragSOLUserTokenAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());

        return { event, error, fragSOLFund, fragSOLFundBalance, fragSOLReward, fragSOLUserFund, fragSOLUserReward, fragSOLUserTokenAccount };
    }

    public lamportsToFragSOL(lamports: BN): string {
        return super.lamportsToX(lamports, 9, 'fragSOL');
    }

    public async runOperatorUpdatePrices() {
        const { event, error } = await this.run({
            instructions: [
                this.program.methods
                    .operatorUpdatePrices()
                    .accounts({ operator: this.wallet.publicKey })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            events: ['operatorUpdatedFundPrice'],
        });
        const [
            fragSOLFund,
            fragSOLFundBalance,
        ] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragSOLFund),
            this.connection.getBalance(this.knownAddress.fragSOLFund, 'confirmed').then(v => new BN(v)),
        ]);

        logger.notice(`operator updated prices: ${this.lamportsToSOL(event.operatorUpdatedFundPrice.fundAccount.receiptTokenPrice)}/fragSOL`);
        return { event, error, fragSOLFund, fragSOLFundBalance };
    }

    public async runUserDepositSupportedToken(user: web3.Keypair, tokenSymbol: keyof typeof this.supportedTokenMetadata, amount: BN, depositMetadata?: IdlTypes<Restaking>['depositMetadata'], depositMetadataSigningKeypair?: web3.Keypair) {
        let depositMetadataInstruction: web3.TransactionInstruction[] = [];
        if (depositMetadata) {
            depositMetadataSigningKeypair = depositMetadataSigningKeypair ?? this.keychain.getKeypair('ADMIN');
            const message = this.program.coder.types.encode('depositMetadata', depositMetadata);
            const signature = ed25519.Sign(message, Buffer.from(depositMetadataSigningKeypair.secretKey));
            depositMetadataInstruction.push(
                web3.Ed25519Program.createInstructionWithPublicKey({
                    publicKey: depositMetadataSigningKeypair.publicKey.toBytes(),
                    message,
                    signature,
                }),
            );
        }

        const supportedToken = this.supportedTokenMetadata[tokenSymbol];
        const userSupportedTokenAddress = this.knownAddress.userSupportedTokenAccount(user.publicKey, tokenSymbol);

        const { event, error } = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragSOLFundAndRewardAccounts(user)),
                ...depositMetadataInstruction,
                this.program.methods
                    .userDepositSupportedToken(amount, depositMetadata)
                    .accounts({
                        user: user.publicKey,
                        supportedTokenMint: supportedToken.mint,
                        supportedTokenProgram: supportedToken.program,
                        userSupportedTokenAccount: userSupportedTokenAddress,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signers: [user],
            events: ['userDepositedSupportedTokenToFund', 'userUpdatedRewardPool']
        });

        logger.notice(`user deposited: ${this.lamportsToX(amount, supportedToken.decimals, tokenSymbol)}`.padEnd(LOG_PAD_LARGE), userSupportedTokenAddress.toString());
        const [
            fragSOLFund,
            fragSOLFundBalance,
            fragSOLReward,
            fragSOLUserFund,
            fragSOLUserReward,
            fragSOLUserTokenAccount,
            userSupportedTokenAccount,
        ] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragSOLFund),
            this.connection.getBalance(this.knownAddress.fragSOLFund, 'confirmed').then(v => new BN(v)),
            this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward),
            this.account.userFundAccount.fetch(this.knownAddress.fragSOLUserFund(user.publicKey)),
            this.account.userRewardAccount.fetch(this.knownAddress.fragSOLUserReward(user.publicKey)),
            this.getUserFragSOLAccount(user.publicKey),
            this.getUserSupportedTokenAccount(user.publicKey, tokenSymbol),
        ]);
        logger.info(`user fragSOL balance: ${this.lamportsToFragSOL(new BN(fragSOLUserTokenAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());

        return { event, error, fragSOLFund, fragSOLFundBalance, fragSOLReward, fragSOLUserFund, fragSOLUserReward, fragSOLUserTokenAccount, userSupportedTokenAccount };
    }

    public async runUserRequestWithdrawal(user: web3.Keypair, amount: BN) {
        const { event, error } = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragSOLFundAndRewardAccounts(user)),
                this.program.methods
                    .userRequestWithdrawal(amount)
                    .accounts({
                        user: user.publicKey,
                    })
                    .instruction(),
            ],
            signers: [user],
            events: ['userRequestedWithdrawalFromFund', 'userUpdatedRewardPool']
        });

        logger.notice(`user requested withdrawal: ${this.lamportsToFragSOL(amount)} #${event.userRequestedWithdrawalFromFund.requestId.toString()}/${event.userRequestedWithdrawalFromFund.batchId.toString()}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());
        const [
            fragSOLFund,
            fragSOLFundBalance,
            fragSOLReward,
            fragSOLUserFund,
            fragSOLUserReward,
            fragSOLUserTokenAccount,
            fragSOLLockAccount,
        ] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragSOLFund),
            this.connection.getBalance(this.knownAddress.fragSOLFund, 'confirmed').then(v => new BN(v)),
            this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward),
            this.account.userFundAccount.fetch(this.knownAddress.fragSOLUserFund(user.publicKey)),
            this.account.userRewardAccount.fetch(this.knownAddress.fragSOLUserReward(user.publicKey)),
            this.getUserFragSOLAccount(user.publicKey),
            this.getFragSOLLockAccount(),
        ]);
        logger.info(`user fragSOL balance: ${this.lamportsToFragSOL(new BN(fragSOLUserTokenAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());

        return { event, error, fragSOLFund, fragSOLFundBalance, fragSOLReward, fragSOLUserFund, fragSOLUserReward, fragSOLUserTokenAccount, fragSOLLockAccount };
    }

    public async runUserCancelWithdrawalRequest(user: web3.Keypair, requestId: BN) {
        const { event, error } = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragSOLFundAndRewardAccounts(user)),
                this.program.methods
                    .userCancelWithdrawalRequest(requestId)
                    .accounts({
                        user: user.publicKey,
                    })
                    .instruction(),
            ],
            signers: [user],
            events: ['userCanceledWithdrawalRequestFromFund', 'userUpdatedRewardPool']
        });

        logger.notice(`user canceled withdrawal request: #${requestId.toString()}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());
        const [
            fragSOLFund,
            fragSOLFundBalance,
            fragSOLReward,
            fragSOLUserFund,
            fragSOLUserReward,
            fragSOLUserTokenAccount,
            fragSOLLockAccount,
        ] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragSOLFund),
            this.connection.getBalance(this.knownAddress.fragSOLFund, 'confirmed').then(v => new BN(v)),
            this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward),
            this.account.userFundAccount.fetch(this.knownAddress.fragSOLUserFund(user.publicKey)),
            this.account.userRewardAccount.fetch(this.knownAddress.fragSOLUserReward(user.publicKey)),
            this.getUserFragSOLAccount(user.publicKey),
            this.getFragSOLLockAccount(),
        ]);
        logger.info(`user fragSOL balance: ${this.lamportsToFragSOL(new BN(fragSOLUserTokenAccount.amount.toString()))}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());

        return { event, error, fragSOLFund, fragSOLFundBalance, fragSOLReward, fragSOLUserFund, fragSOLUserReward, fragSOLUserTokenAccount, fragSOLLockAccount };
    }

    public async runOperatorProcessFundWithdrawalJob(operator: web3.Keypair, forced: boolean = false) {
        const { event, error } = await this.run({
            instructions: [
                this.program.methods
                    .operatorProcessFundWithdrawalJob(forced)
                    .accounts({
                        operator: operator.publicKey,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signers: [operator],
            events: ['operatorProcessedJob'] // , 'operatorUpdatedFundPrice'
        });

        logger.notice(`operator processed withdrawal job: #${event.operatorProcessedJob.fundAccount.withdrawalLastCompletedBatchId.toString()}`.padEnd(LOG_PAD_LARGE), operator.publicKey.toString());
        const [
            fragSOLFund,
            fragSOLFundBalance,
            fragSOLReward,
            fragSOLLockAccount,
        ] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragSOLFund),
            this.connection.getBalance(this.knownAddress.fragSOLFund, 'confirmed').then(v => new BN(v)),
            this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward),
            this.getFragSOLLockAccount(),
        ]);

        return { event, error, fragSOLFund, fragSOLFundBalance, fragSOLReward, fragSOLLockAccount };
    }

    public async runUserWithdraw(user: web3.Keypair, requestId: BN) {
        const { event, error } = await this.run({
            instructions: [
                ...(await this.getInstructionsToUpdateUserFragSOLFundAndRewardAccounts(user)),
                this.program.methods
                    .userWithdraw(requestId)
                    .accounts({
                        user: user.publicKey,
                    })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            signers: [user],
            events: ['userWithdrewSolFromFund']
        });

        logger.notice(`user withdrew: ${this.lamportsToSOL(event.userWithdrewSolFromFund.withdrawnSolAmount)} #${requestId.toString()}`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());
        const [
            fragSOLFund,
            fragSOLFundBalance,
            fragSOLReward,
            fragSOLUserFund,
            fragSOLUserReward,
            fragSOLUserTokenAccount,
            fragSOLLockAccount,
        ] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragSOLFund),
            this.connection.getBalance(this.knownAddress.fragSOLFund, 'confirmed').then(v => new BN(v)),
            this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward),
            this.account.userFundAccount.fetch(this.knownAddress.fragSOLUserFund(user.publicKey)),
            this.account.userRewardAccount.fetch(this.knownAddress.fragSOLUserReward(user.publicKey)),
            this.getUserFragSOLAccount(user.publicKey),
            this.getFragSOLLockAccount(),
        ]);

        return { event, error, fragSOLFund, fragSOLFundBalance, fragSOLReward, fragSOLUserFund, fragSOLUserReward, fragSOLUserTokenAccount, fragSOLLockAccount };
    }
}