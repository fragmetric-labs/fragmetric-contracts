import * as anchor from '@coral-xyz/anchor';
import { BN, web3 } from '@coral-xyz/anchor';
// @ts-ignore
import * as splTokenMetadata from '@solana/spl-token-metadata';
import * as splStakePool from '@solana/spl-stake-pool';
import * as marinade from '@marinade.finance/marinade-ts-sdk';
// @ts-ignore
import * as spl from '@solana/spl-token-3.x';
import * as ed25519 from 'ed25519';
import {AnchorPlayground, AnchorPlaygroundConfig, getLogger} from '../lib';
import {Restaking} from '../../target/types/restaking';
import {getKeychain, KEYCHAIN_ENV, KEYCHAIN_KEYS} from './keychain';
import {IdlTypes} from "@coral-xyz/anchor/dist/cjs/program/namespace/types";

const { logger, LOG_PAD_SMALL, LOG_PAD_LARGE } = getLogger('restaking');

export class RestakingPlayground extends AnchorPlayground<Restaking, KEYCHAIN_KEYS> {
    // The term "local" in the "KEYCHAIN_ENV" context doesn't necessarily refer to the localnet.
    // It can also be applied in devnet or mainnet environments while utilizing existing local keypairs.
    // and a different Anchor provider. This allows for flexibility in testing across various networks.
    public static create(env: KEYCHAIN_ENV, args?: Pick<AnchorPlaygroundConfig<Restaking, any>, 'provider'>) {
        return getKeychain(env)
            .then(keychain => {
                return new RestakingPlayground({
                    keychain,
                    provider: new anchor.AnchorProvider(
                        args?.provider?.connection ?? new web3.Connection(RestakingPlayground.clusterURL[env]),
                        new anchor.Wallet(keychain.wallet),
                    ),
                })
            });
    }

    private static readonly clusterURL: {[env in KEYCHAIN_ENV]: string} = {
        local: 'http://0.0.0.0:8899',
        devnet: web3.clusterApiUrl('devnet'),
        mainnet: web3.clusterApiUrl('mainnet-beta'),
    };

    private constructor(args: Pick<AnchorPlaygroundConfig<Restaking, any>, 'provider'|'keychain'>) {
        super({
            provider: args.provider,
            keychain: args.keychain,
            idl: require('../../target/idl/restaking.json') as Restaking,
        });
    }

    public BN(x: number | string | number[] | Uint8Array | Buffer | BN): BN {
        return new BN(x)
    }

    public get initializeSteps() {
        if (this._initializeSteps) return this._initializeSteps;
        return this._initializeSteps = this._getInitializeSteps();
    }
    private _initializeSteps: ReturnType<typeof this._getInitializeSteps>;
    private _getInitializeSteps() {
        return [
            () => this.runAdminInitializeTokenMint(), // 0
            () => this.runAdminInitializeFundAccounts(), // 1
            () => this.runAdminUpdateRewardAccounts(), // 2
            () => this.runAdminTransferMintAuthority(), // 3
            () => this.runFundManagerInitializeFundConfigurations(), // 4
            () => this.runFundManagerUpdateFundConfigurations(), // 5
            () => this.runFundManagerInitializeRewardPools(), // 6
            () => this.runFundManagerSettleReward({ // 7
                poolName: 'bonus',
                rewardName: 'fPoint',
                amount: new BN(0),
            }),
            () => this.runAdminInitializeNSOLTokenMint(), // 8*****
            () => this.runAdminInitializeNormalizeTokenPool(), // 9*****
            () => this.runFundManagerInitializeNormalizeTokenPoolConigurations(), // 10*******
        ]
    }

    public get knownAddress() {
        if (this._knownAddress) return this._knownAddress;
        return this._knownAddress = this._getKnownAddress();
    }
    private _knownAddress: ReturnType<typeof this._getKnownAddress>;
    private _getKnownAddress() {
        const nSOLTokenMint = this.getConstantAsPublicKey('nsolMintAddress');
        const nSOLTokenMintBuf = nSOLTokenMint.toBuffer();
        const [nSOLTokenPool] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from('normalized_token_pool'), nSOLTokenMintBuf],
            this.programId
        );
        const [nSOLTokenAccount] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from('normalized_token'), nSOLTokenMintBuf],
            this.programId
        );
        const nSOLSupportedTokenLockAccount = (symbol: keyof typeof this.supportedTokenMetadata) => web3.PublicKey.findProgramAddressSync(
            [Buffer.from('supported_token_lock'), nSOLTokenMintBuf, this.supportedTokenMetadata[symbol].mint.toBuffer()],
            this.programId
        );

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
        const [fragSOLFundExecutionReservedAccount] = web3.PublicKey.findProgramAddressSync(
            [Buffer.from("fund_execution_reserved"), fragSOLTokenMintBuf],
            this.programId,
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
            [Buffer.from('supported_token'), fragSOLTokenMintBuf, this.supportedTokenMetadata[symbol].mint.toBuffer()],
            this.programId
        )[0];
        const userSupportedTokenAccount = (user: web3.PublicKey, symbol: keyof typeof this.supportedTokenMetadata) => spl.getAssociatedTokenAddressSync(
            this.supportedTokenMetadata[symbol].mint,
            user,
            false,
            this.supportedTokenMetadata[symbol].program,
        );
        return {
            nSOLTokenMint,
            nSOLTokenAccount,
            nSOLTokenPool,
            nSOLSupportedTokenLockAccount,
            fragSOLTokenMint,
            fragSOLTokenLock,
            fragSOLFund,
            fragSOLFundExecutionReservedAccount,
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

    public readonly fragSOLDecimals = 9;
    public readonly nSOLDecimals = 9;

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
        return [
            ...Object.values(this.supportedTokenMetadata).map(v => {
                return {
                    pubkey: v.pricingSourceAddress,
                    isSigner: false,
                    isWritable: false,
                };
            }),
            {
                pubkey: this.knownAddress.nSOLTokenMint,
                isSigner: false,
                isWritable: false,
            },
            {
                pubkey: this.knownAddress.nSOLTokenPool,
                isSigner: false,
                isWritable: false,
            }
        ]
    }

    public get jitoStakePoolDepositAccounts() {
        if (this._jitoStakePoolDepositAccounts) return this._jitoStakePoolDepositAccounts;
        return this._jitoStakePoolDepositAccounts = this._getJitoStakePoolDepositAccounts();
    }
    private _jitoStakePoolDepositAccounts: ReturnType<typeof this._getJitoStakePoolDepositAccounts>;
    private _getJitoStakePoolDepositAccounts(): web3.AccountMeta[] {
        const [jitoStakePoolWithdrawAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
            [
                new anchor.web3.PublicKey("Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb").toBuffer(),
                Buffer.from("withdraw")
            ],
            new anchor.web3.PublicKey("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy")
        );

        return [
            { // stake pool program id
                pubkey: new anchor.web3.PublicKey("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy"),
                isSigner: false,
                isWritable: false,
            },
            { // jito stake pool address
                pubkey: new anchor.web3.PublicKey("Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb"),
                isSigner: false,
                isWritable: true,
            },
            { // stake_pool_withdraw_authority
                pubkey: jitoStakePoolWithdrawAuthority,
                isSigner: false,
                isWritable: false,
            },
            { // reserve_stake_account
                pubkey: new anchor.web3.PublicKey("BgKUXdS29YcHCFrPm5M8oLHiTzZaMDjsebggjoaQ6KFL"),
                isSigner: false,
                isWritable: true,
            },
            // { // lamports_from
            //   pubkey: new anchor.web3.PublicKey(""), // fundAccount
            //   isSigner: true,
            //   isWritable: true,
            // },
            // { // pool_tokens_to
            //   pubkey: new anchor.web3.PublicKey(""), // fundAccount의 token account
            //   isSigner: false,
            //   isWritable: true,
            // },
            { // manager_fee_account
                pubkey: new anchor.web3.PublicKey("feeeFLLsam6xZJFc6UQFrHqkvVt4jfmVvi2BRLkUZ4i"),
                isSigner: false,
                isWritable: true,
            },
            // { // referrer_pool_tokens_account
            //   pubkey: new anchor.web3.PublicKey(""), // pool_tokens_to랑 같게
            //   isSigner: false,
            //   isWritable: true,
            // },
            // { // pool_mint
            //   pubkey: new anchor.web3.PublicKey("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn"),
            //   isSigner: false,
            //   isWritable: true,
            // },
            // { // token_program_id
            //   pubkey: spl.TOKEN_PROGRAM_ID,
            //   isSigner: false,
            //   isWritable: false,
            // },
        ];
    }

    public async tryAirdropSupportedTokens(account: web3.PublicKey, amount = 100) {
        await this.tryAirdrop(account, amount);

        const signers: web3.Signer[] = [];
        await this.run({
            instructions: [
                ...(await Promise.all(
                    Object.values(this.supportedTokenMetadata).map(async (token) => {
                        const ata = await spl.getOrCreateAssociatedTokenAccount(
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
                        );
                        const splStakePoolAddress: web3.PublicKey | null = token.pricingSource['splStakePool']?.address ?? null;
                        if (splStakePoolAddress) {
                            const res = await splStakePool.depositSol(
                                this.connection,
                                splStakePoolAddress,
                                this.wallet.publicKey,
                                amount * web3.LAMPORTS_PER_SOL,
                                ata.address,
                            );
                            signers.push(...res.signers);
                            return res.instructions;
                        }

                        const marinadeStakePoolAddress: web3.PublicKey | null = token.pricingSource['marinadeStakePool']?.address ?? null;
                        if (marinadeStakePoolAddress) {
                            const marinadeStakePool = new marinade.Marinade(
                                new marinade.MarinadeConfig({
                                    connection: this.connection as unknown as anchor.web3.Connection,
                                    publicKey: this.wallet.publicKey,
                                })
                            )
                            const res = await marinadeStakePool.deposit(
                                new BN(amount * web3.LAMPORTS_PER_SOL),
                                {
                                    mintToOwnerAddress: account,
                                }
                            )
                            return res.transaction.instructions;
                        }

                        return [];
                        // return [
                        //     spl.createMintToCheckedInstruction(
                        //         token.mint,
                        //         ata.address,
                        //         this.keychain.getPublicKey('MOCK_ALL_MINT_AUTHORITY'),
                        //         amount * (10 ** token.decimals),
                        //         token.decimals,
                        //         [],
                        //         token.program,
                        //     ),
                        // ];
                    }),
                )).flat(),
            ],
            // signerNames: ['MOCK_ALL_MINT_AUTHORITY'],
            signers,
        });

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
            'confirmed',
            this.supportedTokenMetadata[symbol].program,
        );
    }

    public getUserFragSOLAccount(user: web3.PublicKey) {
        return spl.getAccount(
            // @ts-ignore
            this.connection,
            this.knownAddress.fragSOLUserTokenAccount(user),
            'confirmed',
            spl.TOKEN_2022_PROGRAM_ID,
        );
    }

    public getFragSOLLockAccount() {
        return spl.getAccount(
            // @ts-ignore
            this.connection,
            this.knownAddress.fragSOLTokenLock,
            'confirmed',
            spl.TOKEN_2022_PROGRAM_ID,
        );
    }

    public getUserFragSOLRewardAccount(user: web3.PublicKey) {
        return this.account.userRewardAccount.fetch(this.knownAddress.fragSOLUserReward(user))
    }

    public getFragSOLRewardAccount() {
        return this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward);
    }

    public getFragSOLFundAccount() {
        return this.account.fundAccount.fetch(this.knownAddress.fragSOLFund, "confirmed");
    }

    public getFragSOLFundExecutionReservedAccountBalance() {
        return this.connection.getBalance(this.knownAddress.fragSOLFundExecutionReservedAccount, "confirmed");
    }

    public getNSOLTokenPoolAccount() {
        return this.account.normalizedTokenPoolAccount.fetch(this.knownAddress.nSOLTokenPool);
    }

    public async runAdminInitializeTokenMint() {
        const metadata: splTokenMetadata.TokenMetadata = {
            mint: this.keychain.getPublicKey('FRAGSOL_MINT'),
            name: 'Fragmetric Restaked SOL',
            symbol: 'fragSOL',
            uri: 'https://quicknode.quicknode-ipfs.com/ipfs/QmcueajXkNzoYRhcCv323PMC8VVGiDvXaaVXkMyYcyUSRw',
            additionalMetadata: [['description', `fragSOL is Solana's first native LRT that provides optimized restaking rewards.`]],
            updateAuthority: this.keychain.getPublicKey('ADMIN'),
        };
        const fileForMetadataURI = JSON.stringify({
            name: metadata.name,
            symbol: metadata.symbol,
            description: metadata.additionalMetadata[0][1],
            image: 'https://fragmetric-assets.s3.ap-northeast-2.amazonaws.com/fragsol.png',
            // attributes: [],
        }, null, 0);
        logger.debug(`fragSOL metadata file:\n> ${metadata.uri}\n> ${fileForMetadataURI}`);

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
                    this.fragSOLDecimals,
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
            // @ts-ignore
            this.connection,
            this.knownAddress.fragSOLTokenMint,
            "confirmed",
            spl.TOKEN_2022_PROGRAM_ID,
        );
        logger.notice('fragSOL token mint created with extensions'.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLTokenMint.toString());
        return { fragSOLMint };
    }

    public async runAdminInitializeNSOLTokenMint() {
        // const metadata: splTokenMetadata.TokenMetadata = {
        //     mint: this.keychain.getPublicKey('FRAGSOL_MINT'),
        //     name: 'Fragmetric Restaked SOL',
        //     symbol: 'fragSOL',
        //     uri: 'https://quicknode.quicknode-ipfs.com/ipfs/QmcueajXkNzoYRhcCv323PMC8VVGiDvXaaVXkMyYcyUSRw',
        //     additionalMetadata: [['description', `fragSOL is Solana's first native LRT that provides optimized restaking rewards.`]],
        //     updateAuthority: this.keychain.getPublicKey('ADMIN'),
        // };
        // const fileForMetadataURI = JSON.stringify({
        //     name: metadata.name,
        //     symbol: metadata.symbol,
        //     description: metadata.additionalMetadata[0][1],
        //     image: 'https://fragmetric-assets.s3.ap-northeast-2.amazonaws.com/fragsol.png',
        //     // attributes: [],
        // }, null, 0);
        // logger.debug(`fragSOL metadata file:\n> ${metadata.uri}\n> ${fileForMetadataURI}`);

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
                    this.keychain.getPublicKey('ADMIN'),
                    null, // freeze authority to be null
                    spl.TOKEN_PROGRAM_ID,
                ),
            ],
            signerNames: ['NSOL_MINT'],
        });
        const nSOLMint = await spl.getMint(
            this.connection,
            this.knownAddress.nSOLTokenMint,
            "confirmed",
            spl.TOKEN_PROGRAM_ID,
        );
        logger.notice('nSOL token mint created'.padEnd(LOG_PAD_LARGE), this.knownAddress.nSOLTokenMint.toString());
        return { nSOLMint: nSOLMint };
    }

    public async runAdminUpdateTokenMetadata() {
        const fragSOLTokenMetadataAddress = this.knownAddress.fragSOLTokenMint;

        let tokenMetadata = await spl.getTokenMetadata(this.connection, fragSOLTokenMetadataAddress, undefined, spl.TOKEN_2022_PROGRAM_ID);
        logger.debug(`current token metadata:\n> ${JSON.stringify(tokenMetadata, null, 0)}`);

        const updatedFileForMetadataURI = JSON.stringify({
            name: tokenMetadata.name,
            symbol: tokenMetadata.symbol,
            description: tokenMetadata.additionalMetadata[0][1],
            image: 'https://fragmetric-assets.s3.ap-northeast-2.amazonaws.com/fragsol.png',
            // attributes: [],
        }, null, 0);
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
            signerNames: ['ADMIN'],
        });

        tokenMetadata = await spl.getTokenMetadata(
            this.connection,
            fragSOLTokenMetadataAddress,
            "confirmed",
            spl.TOKEN_2022_PROGRAM_ID,
        );
        logger.notice(`updated token metadata:\n> ${JSON.stringify(tokenMetadata, null, 2)}`);
    }

    public async runAdminUpdateRewardAccounts(batchSize = 35) {
        const currentVersion = await this.connection.getAccountInfo(this.knownAddress.fragSOLReward)
            .then(a => a.data.readInt16LE(8))
            .catch(err => 0);

        const targetVersion = parseInt(this.getConstant('rewardAccountCurrentVersion'));
        const instructions = [
            ...(currentVersion == 0 ? [
                    this.program.methods
                        .adminInitializeRewardAccount()
                        .accounts({ payer: this.wallet.publicKey })
                        .instruction()
            ] : []),
            ...new Array(targetVersion - currentVersion)
                .fill(null)
                .map((_, index, arr) =>
                    this.program.methods
                        .adminUpdateRewardAccountsIfNeeded(null)
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

        const fragSOLRewardAccount = await this.account.rewardAccount.fetch(this.knownAddress.fragSOLReward, "confirmed");
        logger.notice(`updated reward account version from=${currentVersion}, to=${fragSOLRewardAccount.dataVersion}, target=${targetVersion}`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLReward.toString());

        return { fragSOLRewardAccount };
    }

    public async runAdminInitializeFundAccounts() {
        await this.run({
            instructions: [
                this.program.methods
                    .adminInitializeReceiptTokenLockAuthority()
                    .accounts({payer: this.wallet.publicKey})
                    .instruction(),
                this.program.methods
                    .adminInitializeReceiptTokenLockAccount()
                    .accounts({payer: this.wallet.publicKey})
                    .instruction(),
                this.program.methods
                    .adminInitializeFundAccount()
                    .accounts({payer: this.wallet.publicKey})
                    .instruction(),
            ],
            signerNames: ['ADMIN'],
        });
        const fragSOLFundAccount = await this.account.fundAccount.fetch(this.knownAddress.fragSOLFund, "confirmed");
        logger.notice('fragSOL fund account created'.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLFund.toString());

        return { fragSOLFundAccount };
    }

    public async runAdminUpdateFundAccounts() {
        await this.run({
            instructions: [
                this.program.methods
                    .adminUpdateFundAccountIfNeeded()
                    .accounts({payer: this.wallet.publicKey})
                    .instruction(),
            ],
            signerNames: ['ADMIN'],
        });
        const fragSOLFundAccount = await this.account.fundAccount.fetch(this.knownAddress.fragSOLFund, "confirmed");
        logger.notice('fragSOL fund account updated'.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLFund.toString());

        return { fragSOLFundAccount };
    }

    public async runAdminInitializeNormalizeTokenPool() {
        await this.run({
            instructions: [
                this.program.methods
                    .adminInitializeNormalizedTokenPool()
                    .accounts({payer: this.wallet.publicKey})
                    .instruction(),
            ],
            signerNames: ['ADMIN'],
        });
        const nSOLTokenPoolAccount = await this.getNSOLTokenPoolAccount();
        logger.notice('nSOL token pool created'.padEnd(LOG_PAD_LARGE), this.knownAddress.nSOLTokenPool.toString());

        return { nSOLTokenPoolAccount }
    }

    public async runAdminTransferMintAuthority() {
        await this.run({
            instructions: [
                this.program.methods
                    .adminInitializeReceiptTokenMintAuthority()
                    .accounts({ payer: this.wallet.publicKey })
                    .instruction(),
                this.program.methods
                    .adminInitializeExtraAccountMetaList()
                    .accounts({ payer: this.wallet.publicKey })
                    .instruction(),
                this.program.methods
                    .adminUpdateExtraAccountMetaListIfNeeded()
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
                "confirmed",
                spl.TOKEN_2022_PROGRAM_ID,
            ),
            this.connection.getAccountInfo(spl.getExtraAccountMetaAddress(this.knownAddress.fragSOLTokenMint, this.programId))
                .then(acc => spl.getExtraAccountMetas(acc)),
        ]);
        logger.notice(`transferred fragSOL mint authority to the PDA`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLTokenMintAuthority.toString());

        return { fragSOLMint, fragSOLExtraAccountMetasAccount };
    }

    public async runFundManagerInitializeFundConfigurations() {
        const { event, error } = await this.run({
            instructions: [
                this.program.methods
                    .fundManagerUpdateSolCapacityAmount(new BN(0))
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
                ...Object.values(this.supportedTokenMetadata).flatMap(v => {
                    return [
                        this.program.methods
                            .fundManagerInitializeSupportedTokenAuthority()
                            .accounts({
                                payer: this.wallet.publicKey,
                                supportedTokenMint: v.mint,
                            })
                            .instruction(),
                        this.program.methods
                            .fundManagerInitializeSupportedTokenAccount()
                            .accountsPartial({
                                payer: this.wallet.publicKey,
                                supportedTokenMint: v.mint,
                                supportedTokenProgram: v.program,
                            })
                            .instruction(),
                        this.program.methods
                            .fundManagerAddSupportedToken(
                                new BN(0),
                                v.pricingSource,
                            )
                            .accountsPartial({
                                supportedTokenMint: v.mint,
                                supportedTokenProgram: v.program,
                            })
                            .remainingAccounts(this.pricingSourceAccounts)
                            .instruction(),
                        ];
                }),
            ],
            signerNames: ['FUND_MANAGER'],
            events: ['fundManagerUpdatedFund'],
        });

        logger.notice(`initialized fragSOL fund configuration`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLFund.toString());
        const fragSOLFund = await this.account.fundAccount.fetch(this.knownAddress.fragSOLFund);
        return { event, error, fragSOLFund };
    }

    public get targetFragSOLFundConfiguration() {
        return {
            solCapacity: (this.isMaybeMainnetBeta ? new BN(27_000) : new BN(1_000)).mul(new BN(web3.LAMPORTS_PER_SOL)),
            solWithdrawalFeedRateBPS: this.isMaybeMainnetBeta ? 10 : 10,
            withdrawalEnabled: this.isMaybeMainnetBeta ? false : true,
            withdrawalBatchProcessingThresholdAmount: new BN(this.isMaybeMainnetBeta ? 0 : 0),
            withdrawalBatchProcessingThresholdDuration: new BN(this.isMaybeMainnetBeta ? 60 : 60), // seconds
            supportedTokens: Object.entries(this.supportedTokenMetadata).map(([symbol, v]) => ({
                mint: v.mint,
                capacity: (() => {
                   switch (symbol) {
                       case 'bSOL':
                           return (new BN(this.isMaybeMainnetBeta ? 0 : 90)).mul(new BN(10 ** v.decimals));
                       case 'jitoSOL':
                           return (new BN(this.isMaybeMainnetBeta ? 13_500 : 80)).mul(new BN(10 ** v.decimals));
                       case 'mSOL':
                           return (new BN(this.isMaybeMainnetBeta ? 4_500 : 70)).mul(new BN(10 ** v.decimals));
                       default:
                           throw `invalid cap for ${symbol}`;
                   }
                })(),
            })),
        }
    }

    // update capacity and configurations
    public async runFundManagerUpdateFundConfigurations() {
        const config = this.targetFragSOLFundConfiguration;
        const { event, error } = await this.run({
            instructions: [
                this.program.methods
                    .fundManagerUpdateSolCapacityAmount(config.solCapacity)
                    .instruction(),
                this.program.methods
                    .fundManagerUpdateSolWithdrawalFeeRate(config.solWithdrawalFeedRateBPS) // 1 fee rate = 1bps = 0.01%
                    .instruction(),
                this.program.methods
                    .fundManagerUpdateWithdrawalEnabledFlag(config.withdrawalEnabled)
                    .instruction(),
                this.program.methods
                    .fundManagerUpdateBatchProcessingThreshold(
                        config.withdrawalBatchProcessingThresholdAmount,
                        config.withdrawalBatchProcessingThresholdDuration,
                    )
                    .instruction(),
                ...config.supportedTokens.flatMap(v => {
                    return [
                        this.program.methods
                            .fundManagerUpdateSupportedTokenCapacityAmount(
                                v.mint,
                                v.capacity,
                            )
                            .remainingAccounts(this.pricingSourceAccounts)
                            .instruction(),
                    ];
                }),
            ],
            signerNames: ['FUND_MANAGER'],
            events: ['fundManagerUpdatedFund'],
        });

        logger.notice(`updated fragSOL fund configuration`.padEnd(LOG_PAD_LARGE), this.knownAddress.fragSOLFund.toString());
        const fragSOLFund = await this.account.fundAccount.fetch(this.knownAddress.fragSOLFund);
        return { event, error, fragSOLFund };
    }

    public async runFundManagerInitializeNormalizeTokenPoolConigurations() {
        await this.run({
            instructions: [
                ...Object.values(this.supportedTokenMetadata).flatMap(v => {
                    return [
                        this.program.methods
                            .fundManagerInitializeSupportedTokenLockAccount()
                            .accounts({
                                payer: this.wallet.publicKey,
                                supportedTokenMint: v.mint,
                                supportedTokenProgram: v.program,
                            })
                            .instruction(),
                        this.program.methods
                            .fundManagerUpdateSupportedTokenLockAccountAuthority()
                            .accounts({
                                supportedTokenMint: v.mint,
                                supportedTokenProgram: v.program,
                            })
                            .instruction(),
                    ];
                }),
                this.program.methods
                    .fundManagerSyncNormalizedTokenPoolSupportedTokens()
                    .instruction(),
            ],
            signerNames: ['FUND_MANAGER'],
        });

        logger.notice(`configured nSOL supported tokens`.padEnd(LOG_PAD_LARGE), this.knownAddress.nSOLTokenPool.toString());
        const nSOLTokenPool = await this.account.normalizedTokenPoolAccount.fetch(this.knownAddress.nSOLTokenPool);
        return { nSOLTokenPool };
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

    public async runFundManagerSettleReward(args: {poolName: (typeof this.rewardPoolsMetadata)[number]['name'], rewardName: (typeof this.rewardsMetadata)[number]['name'], amount: BN}) {
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
        const fragSOLUserFundAddress = this.knownAddress.fragSOLUserFund(user.publicKey);
        const currentRewardVersion = await this.account.userRewardAccount
            .fetch(fragSOLUserRewardAddress)
            .then(a => a.dataVersion)
            .catch(err => 0);
        const currentFundVersion = await this.account.userFundAccount
            .fetch(fragSOLUserFundAddress)
            .then(a => a.dataVersion)
            .catch(err => 0);

        const targetRewardVersion = parseInt(this.getConstant('userRewardAccountCurrentVersion'));
        return [
            ...(currentRewardVersion == 0 ? [
                this.program.methods
                    .userInitializeRewardAccount()
                    .accounts({user: user.publicKey})
                    .instruction()
            ] : []),
            ...new Array(targetRewardVersion - currentRewardVersion)
                .fill(null)
                .map((_, index, arr) =>
                    this.program.methods
                        .userUpdateRewardAccountsIfNeeded(null, index == arr.length - 1)
                        .accounts({user: user.publicKey})
                        .instruction()
                ),
            ...(currentFundVersion == 0 ? [
                this.program.methods
                    .userInitializeFundAccount()
                    .accounts({user: user.publicKey})
                    .instruction(),
                this.program.methods
                    .userInitializeReceiptTokenAccount()
                    .accounts({
                        user: user.publicKey,
                    })
                    .instruction(),
            ] : [
                this.program.methods
                    .userUpdateFundAccountIfNeeded()
                    .accounts({user: user.publicKey})
                    .instruction()
            ]),
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
        return super.lamportsToX(lamports, this.fragSOLDecimals, 'fragSOL');
    }

    public async runOperatorUpdatePrices(operator: web3.Keypair = this.wallet) {
        const { event, error } = await this.run({
            instructions: [
                this.program.methods
                    .operatorUpdatePrices()
                    .accounts({ operator: operator.publicKey })
                    .remainingAccounts(this.pricingSourceAccounts)
                    .instruction(),
            ],
            events: ['operatorUpdatedFundPrice'],
            signers: [operator],
        });
        const [
            fragSOLFund,
            fragSOLFundBalance,
        ] = await Promise.all([
            this.account.fundAccount.fetch(this.knownAddress.fragSOLFund),
            this.connection.getBalance(this.knownAddress.fragSOLFund, 'confirmed').then(v => new BN(v)),
        ]);

        logger.notice(`operator updated prices: ${this.lamportsToSOL(event.operatorUpdatedFundPrice.fundAccount.oneReceiptTokenAsSol)}/fragSOL`);
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

    public async runOperatorProcessFundWithdrawalJob(operator: web3.Keypair = this.wallet, forced: boolean = false) {
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

    public async runTransfer(source: web3.Keypair, destination: web3.PublicKey, amount: BN) {
        const { event, error } = await this.run({
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
                    'confirmed',
                    spl.TOKEN_2022_PROGRAM_ID,
                ),
            ],
            signers: [source],
            events: ['userTransferredReceiptToken', 'userUpdatedRewardPool']
        });

        return { event, error };
    }

    public async runUserUpdateRewardPools(user: web3.Keypair) {
        const { event, error } = await this.run({
            instructions: [
                this.program.methods
                    .userUpdateRewardPools()
                    .accounts({
                        user: user.publicKey,
                    })
                    .instruction(),
            ],
            signers: [user],
            // events: ['userUpdatedRewardPool'], // won't emit it for such void update requests
        });

        logger.notice(`user manually updated user reward pool:`.padEnd(LOG_PAD_LARGE), user.publicKey.toString());
        const [
            fragSOLUserReward,
        ] = await Promise.all([
            this.account.userRewardAccount.fetch(this.knownAddress.fragSOLUserReward(user.publicKey)),
        ]);

        return { event, error, fragSOLUserReward };
    }

    public async runOperatorUpdateRewardPools(operator: web3.Keypair = this.wallet) {
        const { event, error } = await this.run({
            instructions: [
                this.program.methods
                    .operatorUpdateRewardPools()
                    .accounts({
                        operator: operator.publicKey,
                    })
                    .instruction(),
            ],
            signers: [operator],
            events: ['operatorUpdatedRewardPools'], // won't emit it for such void update requests
        });

        logger.notice(`operator manually updated global reward pool:`.padEnd(LOG_PAD_LARGE), operator.publicKey.toString());
        const [
            fragSOLReward,
        ] = await Promise.all([
            this.getFragSOLRewardAccount(),
        ]);

        return { event, error, fragSOLReward };
    }

    public async runOperatorRun(operator: web3.Keypair = this.wallet) {
        const accounts: web3.AccountMeta[] = [
            { // stake_pool_program
                pubkey: new anchor.web3.PublicKey("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy"),
                isSigner: false,
                isWritable: false,
            },
            { // stake_pool
                pubkey: new anchor.web3.PublicKey("Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb"),
                isSigner: false,
                isWritable: true,
            },
            { // stake_pool_withdraw_authority
                pubkey: new anchor.web3.PublicKey("6iQKfEyhr3bZMotVkW6beNZz5CPAkiwvgV2CTje9pVSS"),
                isSigner: false,
                isWritable: false,
            },
            { // reserve_stake_account
                pubkey: new anchor.web3.PublicKey("BgKUXdS29YcHCFrPm5M8oLHiTzZaMDjsebggjoaQ6KFL"),
                isSigner: false,
                isWritable: true,
            },
            { // manager_fee_account
                pubkey: new anchor.web3.PublicKey("feeeFLLsam6xZJFc6UQFrHqkvVt4jfmVvi2BRLkUZ4i"),
                isSigner: false,
                isWritable: true,
            },
            { // pool_mint
                pubkey: new anchor.web3.PublicKey("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn"),
                isSigner: false,
                isWritable: true,
            },
            { // token_program
                pubkey: new anchor.web3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
                isSigner: false,
                isWritable: false,
            },
            { // supported_token_account
                pubkey: this.knownAddress.fragSOLSupportedTokenAccount('jitoSOL'),
                isSigner: false,
                isWritable: true,
            },
        ];
        const { event, error } = await this.run({
            instructions: [
                this.program.methods
                    .operatorRun()
                    .accounts({
                        operator: operator.publicKey,
                    })
                    .remainingAccounts(accounts)
                    .instruction(),
                this.program.methods
                    .operatorRun()
                    .accounts({
                        operator: operator.publicKey,
                    })
                    .remainingAccounts(accounts)
                    .instruction(),
            ],
            signers: [operator],
            events: ['operatorProcessedJob'],
        });

        logger.notice(`operator moved sol fund to operation reserve account`.padEnd(LOG_PAD_LARGE), operator.publicKey.toString());
        logger.notice(`operator deposited sol to`.padEnd(LOG_PAD_LARGE), operator.publicKey.toString());

        const [
            fragSOLFund,
            fragSOLFundExecutionReservedAccountBalance,
        ] = await Promise.all([
            this.getFragSOLFundAccount(),
            this.getFragSOLFundExecutionReservedAccountBalance(),
        ]);

        return { event, error, fragSOLFund, fragSOLFundExecutionReservedAccountBalance };
    }
}
