import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import type { TokenMetadata } from "@solana/spl-token-metadata";
import * as splTokenMetadata from "@solana/spl-token-metadata";
import { expect } from "chai";
import { Restaking } from "../../target/types/restaking";
import { before } from "mocha";
import * as utils from "../utils";
import {RestakingPlayground} from "../../tools/restaking/playground";

export let wallet: anchor.Wallet;
export let adminKeypair: anchor.web3.Keypair;
export let fundManagerKeypair: anchor.web3.Keypair;
export let fragSOLTokenMintMetadata: TokenMetadata;
export let fragSOLTokenMintKeypair: anchor.web3.Keypair;
export let fragSOLFundAddress: anchor.web3.PublicKey;
export let fragSOLRewardAddress: anchor.web3.PublicKey;
export let fragSOLTokenLockAuthorityAddress: anchor.web3.PublicKey;
export let fragSOLTokenMintAuthorityAddress: anchor.web3.PublicKey;
export let fragSOLTokenLockAddress: anchor.web3.PublicKey;
export let fragSOLSupportedTokenAuthorityAddress_bSOL: anchor.web3.PublicKey;
export let fragSOLSupportedTokenAuthorityAddress_mSOL: anchor.web3.PublicKey;
export let fragSOLSupportedTokenAuthorityAddress_jitoSOL: anchor.web3.PublicKey;
export let fragSOLSupportedTokenAuthorityAddress_inf: anchor.web3.PublicKey;

export let tokenMintAddress_bSOL: anchor.web3.PublicKey;
export let tokenMintAddress_mSOL: anchor.web3.PublicKey;
export let tokenMintAddress_jitoSOL: anchor.web3.PublicKey;
export let tokenMintAddress_INF: anchor.web3.PublicKey;
export let tokenMint_bSOL: spl.Mint;
export let tokenMint_mSOL: spl.Mint;
export let tokenMint_jitoSOL: spl.Mint;
export let tokenMint_INF: spl.Mint;
export let tokenMintAuthorityKeypair_all: anchor.web3.Keypair;
export let stakePoolAddress_bSOL: anchor.web3.PublicKey;
export let stakePoolAddress_mSOL: anchor.web3.PublicKey;
export let stakePoolAddress_jitoSOL: anchor.web3.PublicKey;
export let stakePoolAccounts: anchor.web3.AccountMeta[];

export const initialize = describe("Initialize program accounts", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.Restaking as Program<Restaking>;
    wallet = (program.provider as anchor.AnchorProvider).wallet as anchor.Wallet;
    adminKeypair = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../../keypairs/restaking/devnet_admin_fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP.json")));
    fundManagerKeypair = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../../keypairs/restaking/devnet_fund_manager_fragHx7xwt9tXZEHv2bNo3hGTtcHP9geWkqc2Ka6FeX.json")));

    console.log({
        programId: program.programId.toString(),
        rpcEndpoint: program.provider.connection.rpcEndpoint,
        walletPublicKey: wallet.publicKey.toString(),
        fundManagerPublicKey: fundManagerKeypair.publicKey.toString(),
        adminPublicKey: adminKeypair.publicKey.toString(),
    });

    if (utils.isLocalnet(program.provider.connection)) {
        before("Airdrop SOL to payer", async () => {
            await utils.requestAirdrop(program.provider, wallet.payer, 10);
        });
    }

    // Set supported tokens' mint, stake pool accounts
    tokenMintAddress_bSOL = new anchor.web3.PublicKey(require("../mocks/mainnet/bSOL_mint_address.json"));
    stakePoolAddress_bSOL = new anchor.web3.PublicKey(require("../mocks/devnet/bSOL_stake_pool_address.json"));
    tokenMintAddress_mSOL = new anchor.web3.PublicKey(require("../mocks/mainnet/mSOL_mint_address.json"));
    stakePoolAddress_mSOL = new anchor.web3.PublicKey(require("../mocks/mainnet/mSOL_stake_pool_address.json"));
    tokenMintAddress_jitoSOL = new anchor.web3.PublicKey(require("../mocks/mainnet/jitoSOL_mint_address.json"));
    stakePoolAddress_jitoSOL = new anchor.web3.PublicKey(require("../mocks/mainnet/jitoSOL_stake_pool_adress.json"));
    stakePoolAccounts = [
        {
            pubkey: stakePoolAddress_mSOL,
            isSigner: false,
            isWritable: false,
        },
        {
            pubkey: stakePoolAddress_bSOL,
            isSigner: false,
            isWritable: false,
        },
        {
            pubkey: stakePoolAddress_jitoSOL,
            isSigner: false,
            isWritable: false,
        },
    ];
    tokenMintAddress_INF = new anchor.web3.PublicKey(require("../mocks/mainnet/INF_mint_address.json"));

    tokenMintAuthorityKeypair_all = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../mocks/mainnet/all_mint_authority.json")));
    // const tokenMintAuthorityPublicKey_all = tokenMintAuthorityKeypair_all.publicKey;
    // require("../mocks").changeMintAuthority(tokenMintAuthorityPublicKey_all, "./tests/mocks/mainnet/bSOL_mint.json");
    // require("../mocks").changeMintAuthority(tokenMintAuthorityPublicKey_all, "./tests/mocks/mainnet/jitoSOL_mint.json");
    // require("../mocks").changeMintAuthority(tokenMintAuthorityPublicKey_all, "./tests/mocks/mainnet/mSOL_mint.json");
    // require("../mocks").changeMintAuthority(tokenMintAuthorityPublicKey_all, "./tests/mocks/mainnet/INF_mint.json");

    before("Prepare program accounts initialization", async () => {
        const playground = await RestakingPlayground.env();

        console.log("new wallet", (program.provider as anchor.AnchorProvider).wallet.publicKey.toString());

        fragSOLTokenMintKeypair = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../../keypairs/restaking/fragsol_mint_FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo.json")));
        fragSOLTokenMintMetadata = fragSOLTokenMintMetadata = {
            mint: fragSOLTokenMintKeypair.publicKey,
            name: "Fragmetric Restaked SOL",
            symbol: "fragSOL",
            uri: "https://quicknode.quicknode-ipfs.com/ipfs/Qme3xQUAKmtQHVu1hihKeBHuDW35zFPYfZdV6avEW6yRq1",
            additionalMetadata: [["description", "fragSOL is Solana's first native LRT that provides optimized restaking rewards."]],
            updateAuthority: adminKeypair.publicKey,
        };;
        [fragSOLTokenLockAuthorityAddress] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("receipt_token_lock_authority"), fragSOLTokenMintKeypair.publicKey.toBuffer()],
            program.programId
        );
        [fragSOLTokenMintAuthorityAddress] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("receipt_token_mint_authority"), fragSOLTokenMintKeypair.publicKey.toBuffer()],
            program.programId
        );
        [fragSOLSupportedTokenAuthorityAddress_bSOL] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("supported_token_authority"), fragSOLTokenMintKeypair.publicKey.toBuffer(), tokenMintAddress_bSOL.toBuffer()],
            program.programId
        );
        [fragSOLSupportedTokenAuthorityAddress_mSOL] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("supported_token_authority"), fragSOLTokenMintKeypair.publicKey.toBuffer(), tokenMintAddress_mSOL.toBuffer()],
            program.programId
        );
        [fragSOLSupportedTokenAuthorityAddress_jitoSOL] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("supported_token_authority"), fragSOLTokenMintKeypair.publicKey.toBuffer(), tokenMintAddress_jitoSOL.toBuffer()],
            program.programId
        );
        [fragSOLSupportedTokenAuthorityAddress_inf] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("supported_token_authority"), fragSOLTokenMintKeypair.publicKey.toBuffer(), tokenMintAddress_INF.toBuffer()],
            program.programId
        );
        [fragSOLFundAddress] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("fund"), fragSOLTokenMintKeypair.publicKey.toBuffer()],
            program.programId
        );
        [fragSOLTokenLockAddress] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("receipt_token_lock"), fragSOLTokenMintKeypair.publicKey.toBuffer()],
            program.programId
        );
        [fragSOLRewardAddress] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("reward"), fragSOLTokenMintKeypair.publicKey.toBuffer()],
            program.programId
        );

        // NEED TO CHECK: receiptTokenMint == createMint result account
        console.log({
            fragSOLTokenMintPublicKey: fragSOLTokenMintKeypair.publicKey.toString(),
            fragSOLTokenMintMetadata,
            fragSOLFundAddress: fragSOLFundAddress.toString(),
            fragSOLRewardAddress: fragSOLRewardAddress.toString(),
            fragSOLTokenLockAddress: fragSOLTokenLockAddress.toString(),
            fragSOLTokenLockAuthorityAddress: fragSOLTokenLockAuthorityAddress.toString(),
            fragSOLTokenMintAuthorityAddress: fragSOLTokenMintAuthorityAddress.toString(),
            fragSOLSupportedTokenAuthorityAddress_bSOL: fragSOLSupportedTokenAuthorityAddress_bSOL.toString(),
            fragSOLSupportedTokenAuthorityAddress_mSOL: fragSOLSupportedTokenAuthorityAddress_mSOL.toString(),
            fragSOLSupportedTokenAuthorityAddress_jitoSOL: fragSOLSupportedTokenAuthorityAddress_jitoSOL.toString(),
            fragSOLSupportedTokenAuthorityAddress_inf: fragSOLSupportedTokenAuthorityAddress_inf.toString(),
        });
    });

    it("Populate fragSOL token metadata file to upload to IPFS", async () => {
        console.log(JSON.stringify({
            name: fragSOLTokenMintMetadata.name,
            symbol: fragSOLTokenMintMetadata.symbol,
            description: fragSOLTokenMintMetadata.additionalMetadata[0][1],
            image: "https://quicknode.quicknode-ipfs.com/ipfs/QmayYcry2mJGHmcYMn1mqiqxR9kkQXtE3uBEzR9y84vQVL",
            // attributes: [],
        }, null, 2));
    });

    it("Create fragSOL token mint with Transfer Hook extension", async function () {
        // generate keypair to use as address for the transfer-hook enabled mint account
        const decimals = 9;
        const mintInitialSize = spl.getMintLen([spl.ExtensionType.TransferHook, spl.ExtensionType.MetadataPointer]);
        const mintMetadataExtensionSize = (spl.TYPE_SIZE + spl.LENGTH_SIZE) + splTokenMetadata.pack(fragSOLTokenMintMetadata).length;
        const mintTotalSize = mintInitialSize + mintMetadataExtensionSize;
        const lamports = await program.provider.connection.getMinimumBalanceForRentExemption(mintTotalSize);

        const mintTx = new anchor.web3.Transaction().add(
            anchor.web3.SystemProgram.createAccount({
                fromPubkey: wallet.payer.publicKey,
                newAccountPubkey: fragSOLTokenMintKeypair.publicKey,
                lamports: lamports,
                space: mintInitialSize,
                programId: TOKEN_2022_PROGRAM_ID,
            }),
            spl.createInitializeTransferHookInstruction(
                fragSOLTokenMintKeypair.publicKey,
                adminKeypair.publicKey,
                program.programId,
                TOKEN_2022_PROGRAM_ID,
            ),
            spl.createInitializeMetadataPointerInstruction(
                fragSOLTokenMintKeypair.publicKey,
                adminKeypair.publicKey,
                fragSOLTokenMintKeypair.publicKey,
                spl.TOKEN_2022_PROGRAM_ID,
            ),
            spl.createInitializeMintInstruction(
                fragSOLTokenMintKeypair.publicKey,
                decimals,
                adminKeypair.publicKey,
                null, // freeze authority to be null
                spl.TOKEN_2022_PROGRAM_ID,
            ),
            splTokenMetadata.createInitializeInstruction({
                programId: spl.TOKEN_2022_PROGRAM_ID,
                mint: fragSOLTokenMintKeypair.publicKey,
                metadata: fragSOLTokenMintKeypair.publicKey,
                name: fragSOLTokenMintMetadata.name,
                symbol: fragSOLTokenMintMetadata.symbol,
                uri: fragSOLTokenMintMetadata.uri,
                mintAuthority: adminKeypair.publicKey,
                updateAuthority: fragSOLTokenMintMetadata.updateAuthority,
            }),
            ...fragSOLTokenMintMetadata.additionalMetadata
                .map(([field, value]) =>
                    splTokenMetadata.createUpdateFieldInstruction({
                        programId: spl.TOKEN_2022_PROGRAM_ID,
                        metadata: fragSOLTokenMintKeypair.publicKey,
                        updateAuthority: fragSOLTokenMintMetadata.updateAuthority,
                        field,
                        value,
                    }),
                ),
        );
        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            mintTx,
            [wallet.payer, adminKeypair, fragSOLTokenMintKeypair],
        );

        const fragSOLTokenMint = await spl.getMint(
            program.provider.connection,
            fragSOLTokenMintKeypair.publicKey,
            undefined,
            TOKEN_2022_PROGRAM_ID,
        );
        expect(fragSOLTokenMint.address.toString()).to.equal(fragSOLTokenMintKeypair.publicKey.toString());
        expect(fragSOLTokenMint.mintAuthority.toString()).to.equal(adminKeypair.publicKey.toString()); // shall be transferred to a PDA
        expect(fragSOLTokenMint.freezeAuthority).to.null;
    });

    it("Mock supported token mints for localnet", async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        tokenMint_bSOL = await spl.getMint(
            program.provider.connection,
            tokenMintAddress_bSOL,
        );
        tokenMint_mSOL = await spl.getMint(
            program.provider.connection,
            tokenMintAddress_mSOL,
        );
        tokenMint_jitoSOL = await spl.getMint(
            program.provider.connection,
            tokenMintAddress_jitoSOL,
        );
        tokenMint_INF = await spl.getMint(
            program.provider.connection,
            tokenMintAddress_INF,
        );

        expect(tokenMint_bSOL.mintAuthority.toString()).to.equal(tokenMintAuthorityKeypair_all.publicKey.toString());
        expect(tokenMint_mSOL.mintAuthority.toString()).to.equal(tokenMintAuthorityKeypair_all.publicKey.toString());
        expect(tokenMint_jitoSOL.mintAuthority.toString()).to.equal(tokenMintAuthorityKeypair_all.publicKey.toString());
        expect(tokenMint_INF.mintAuthority.toString()).to.equal(tokenMintAuthorityKeypair_all.publicKey.toString());
    });

    it("Initialize fund, reward accounts and configure token mint", async function () {

        // initialization of fund and reward accounts by admin
        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            new anchor.web3.Transaction().add(
                ...await Promise.all([
                    program.methods
                        .adminInitializeFundAccounts()
                        .accounts({ payer: wallet.payer.publicKey })
                        .instruction(),
                    program.methods
                        .adminInitializeRewardAccounts()
                        .accounts({ payer: wallet.payer.publicKey })
                        .instruction(),
                    ...new Array(3).fill(null).map((_, index, arr) =>
                        program.methods
                            .adminUpdateRewardAccountsIfNeeded(null, index == arr.length - 1)
                            .accounts({ payer: wallet.payer.publicKey })
                            .instruction()
                    ),
                    program.methods
                        .adminInitializeReceiptTokenMintAuthorityAndExtraAccountMetaList()
                        .accounts({ payer: wallet.payer.publicKey })
                        .instruction(),
                    program.methods
                        .adminUpdateReceiptTokenMintExtraAccountMetaList()
                        .accounts({ payer: wallet.payer.publicKey })
                        .instruction(),
                ])
            ),
            [wallet.payer, adminKeypair],
        );

        // configuration of fund by fund manager
        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            new anchor.web3.Transaction().add(
                ...await Promise.all([
                    program.methods
                        .fundManagerUpdateSolCapacityAmount(new anchor.BN(1_000_000_000 * 10000))
                        .instruction(),
                    program.methods
                        .fundManagerUpdateSolWithdrawalFeeRate(10) // 1 fee rate = 1bps = 0.01%
                        .instruction(),
                    program.methods
                        .fundManagerUpdateWithdrawalEnabledFlag(true)
                        .instruction(),
                    program.methods
                        .fundManagerUpdateBatchProcessingThreshold(
                            new anchor.BN(0), // batchProcessingThresholdAmount
                            new anchor.BN(0), // batchProcessingThresholdDuration
                        )
                        .instruction(),
                    ...[
                        {
                            supportedTokenMint: tokenMint_bSOL.address,
                            supportedTokenProgram: spl.TOKEN_PROGRAM_ID,
                            capacity: new anchor.BN(1_000_000_000).mul(new anchor.BN(1_000_000_000)),
                            pricingSource: {
                                marinadeStakePool: {
                                    address: stakePoolAddress_mSOL,
                                }
                            },
                        },
                        {
                            supportedTokenMint: tokenMint_mSOL.address,
                            supportedTokenProgram: spl.TOKEN_PROGRAM_ID,
                            capacity: new anchor.BN(1_000_000_000).mul(new anchor.BN(1_000_000_000)),
                            pricingSource: {
                                splStakePool: {
                                    address: stakePoolAddress_bSOL,
                                }
                            },
                        },
                        {
                            supportedTokenMint: tokenMint_jitoSOL.address,
                            supportedTokenProgram: spl.TOKEN_PROGRAM_ID,
                            capacity: new anchor.BN(1_000_000_000).mul(new anchor.BN(1_000_000_000)),
                            pricingSource: {
                                "splStakePool": {
                                    address: stakePoolAddress_jitoSOL,
                                }
                            },
                        },
                        // TODO: not implemented yet
                        // {
                        //     supportedTokenMint: tokenMint_INF.address,
                        //     supportedTokenProgram: spl.TOKEN_PROGRAM_ID,
                        //     capacity: new anchor.BN(1_000_000_000).mul(new anchor.BN(1_000_000_000)),
                        //     pricingSource: {
                        //         "infStakePool": {
                        //             address: stakePoolAddress_INF,
                        //         }
                        //     },
                        // },
                    ].map(def =>
                        program.methods
                            .fundManagerAddSupportedToken(
                                def.capacity,
                                def.pricingSource as any,
                            )
                            .accounts({
                                supportedTokenMint: def.supportedTokenMint,
                                supportedTokenProgram: def.supportedTokenProgram,
                            })
                            .remainingAccounts(stakePoolAccounts)
                            .instruction()
                    ),
                ]),
            ),
            [wallet.payer, fundManagerKeypair],
        );

        // configuration of reward by fund manager
        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            new anchor.web3.Transaction().add(
                ...await Promise.all([
                    ...[
                        {
                            name: "fPoint",
                            description: "Point for fToken airdrop from Fragmetric.",
                            type: {
                                point: {
                                    decimals: 4,
                                }
                            },
                            tokenMint: null,
                            tokenProgram: null,
                        },
                        {
                            name: "bSOL",
                            description: "blablabla.",
                            type: {
                                token: {
                                    mint: tokenMintAddress_bSOL,
                                    program: spl.TOKEN_PROGRAM_ID,
                                    decimals: 9,
                                }
                            },
                        },
                    ].map(def => program.methods
                        .fundManagerAddReward(def.name, def.description, def.type)
                        .accounts({
                            rewardTokenMint: def.type.token?.mint ?? program.programId,
                            rewardTokenProgram: def.type.token?.program ?? program.programId,
                        })
                        .instruction()
                    ),
                    program.methods
                        .fundManagerAddRewardPoolHolder('OrcaDEX', 'Hello De-Fi', [])
                        .instruction(),
                    ...[
                        {
                            name: "fragmetric_base",
                            holderId: null,
                            customContributionAccrualRateEnabled: false,
                        },
                        {
                            name: "fragmetric_bonus",
                            holderId: null,
                            customContributionAccrualRateEnabled: true,
                        },
                    ].map(def =>
                        program.methods
                            .fundManagerAddRewardPool(
                                def.name,
                                def.holderId,
                                def.customContributionAccrualRateEnabled,
                            )
                            .accounts({
                                receiptTokenMint: fragSOLTokenMintKeypair.publicKey,
                            })
                            .instruction()
                    ),
                ])
            ),
            [wallet.payer, fundManagerKeypair],
        );

        // check fund initialized correctly
        const tokensInitialized = (await program.account.fundAccount.fetch(fragSOLFundAddress)).supportedTokens;

        // expect(tokensInitialized[0].mint.toString()).to.eq(tokenMint1.toString());
        expect(tokensInitialized[0].mint.toString()).to.eq(tokenMint_bSOL.address.toString());
        expect(tokensInitialized[0].operationReservedAmount.toNumber()).to.eq(0);

        // expect(tokensInitialized[1].mint.toString()).to.eq(tokenMint2.toString());
        expect(tokensInitialized[1].mint.toString()).to.eq(tokenMint_mSOL.address.toString());
        expect(tokensInitialized[1].operationReservedAmount.toNumber()).to.eq(0);

        const rewardInitialized = await program.account.rewardAccount.fetch(fragSOLRewardAddress);
        expect(rewardInitialized.dataVersion).to.equal(1);

        const receiptTokenMintAccount = await spl.getMint(program.provider.connection, fragSOLTokenMintKeypair.publicKey, undefined, TOKEN_2022_PROGRAM_ID);
        expect(receiptTokenMintAccount.mintAuthority.toString()).to.equal(fragSOLTokenMintAuthorityAddress.toString());
    });
});
