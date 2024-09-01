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

// for devnet // TODO: remove...
export let tokenMintAddress1: anchor.web3.PublicKey;
export let tokenMintAddress2: anchor.web3.PublicKey;

// for localnet
export let tokenMintAddress_bSOL: anchor.web3.PublicKey;
export let tokenMintAddress_mSOL: anchor.web3.PublicKey;
export let tokenMintAddress_jitoSOL: anchor.web3.PublicKey;
export let tokenMintAddress_inf: anchor.web3.PublicKey;
export let tokenMint_bSOL: spl.Mint;
export let tokenMint_mSOL: spl.Mint;
export let tokenMint_jitoSOL: spl.Mint;
export let tokenMint_inf: spl.Mint;
export let tokenMintAuthorityKeypair_all: anchor.web3.Keypair;
export let stakePoolAddress_bSOL: anchor.web3.PublicKey;
export let stakePoolAddress_mSOL: anchor.web3.PublicKey;
export let stakePoolAddress_jitoSOL: anchor.web3.PublicKey;

export const initialize = describe("Initialize program accounts", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.Restaking as Program<Restaking>;
    wallet = (program.provider as anchor.AnchorProvider).wallet as anchor.Wallet;
    adminKeypair = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("./keypairs/devnet_admin_fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP.json")));
    fundManagerKeypair = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("./keypairs/devnet_fund_manager_fragHx7xwt9tXZEHv2bNo3hGTtcHP9geWkqc2Ka6FeX.json")));

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
    stakePoolAddress_bSOL = new anchor.web3.PublicKey(require("../mocks/mainnet/bSOL_stake_pool_address.json"));
    tokenMintAddress_mSOL = new anchor.web3.PublicKey(require("../mocks/mainnet/mSOL_mint_address.json"));
    stakePoolAddress_mSOL = new anchor.web3.PublicKey(require("../mocks/mainnet/mSOL_stake_pool_address.json"));
    tokenMintAddress_jitoSOL = new anchor.web3.PublicKey(require("../mocks/mainnet/jitoSOL_mint_address.json"));
    stakePoolAddress_jitoSOL = new anchor.web3.PublicKey(require("../mocks/mainnet/jitoSOL_stake_pool_adress.json"));
    tokenMintAddress_inf = new anchor.web3.PublicKey(require("../mocks/mainnet/INF_mint_address.json"));
    tokenMintAuthorityKeypair_all = wallet.payer;

    // should reset validator once changed
    const tokenMintAuthorityPublicKey_all = tokenMintAuthorityKeypair_all.publicKey;
    require("../mocks").changeMintAuthority(tokenMintAuthorityPublicKey_all, "./tests/mocks/mainnet/bSOL_mint.json");
    require("../mocks").changeMintAuthority(tokenMintAuthorityPublicKey_all, "./tests/mocks/mainnet/jitoSOL_mint.json");
    require("../mocks").changeMintAuthority(tokenMintAuthorityPublicKey_all, "./tests/mocks/mainnet/mSOL_mint.json");
    require("../mocks").changeMintAuthority(tokenMintAuthorityPublicKey_all, "./tests/mocks/mainnet/INF_mint.json");

    before("Prepare program accounts initialization", async () => {
        fragSOLTokenMintKeypair = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("./keypairs/mint_fragsol_FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo.json")));
        fragSOLTokenMintMetadata = {
            mint: fragSOLTokenMintKeypair.publicKey,
            name: "fragSOL",
            symbol: "FragSOL",
            uri: "https://quicknode.quicknode-ipfs.com/ipfs/QmdTSAPXn8dT2mnS2rxMouedauhjzJeYA74Hmez5P3ZEgE",
            additionalMetadata: [["description", "Fragmetric Liquid Restaking Token"]],
            updateAuthority: adminKeypair.publicKey,
        };
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
            [Buffer.from("supported_token_authority"), fragSOLTokenMintKeypair.publicKey.toBuffer(), tokenMintAddress_inf.toBuffer()],
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
        const metadataSize = splTokenMetadata.pack(fragSOLTokenMintMetadata).length;
        const extensions = [spl.ExtensionType.TransferHook, spl.ExtensionType.MetadataPointer];
        const mintLen = spl.getMintLen(extensions);
        const metadataExtensionLen = spl.TYPE_SIZE + spl.LENGTH_SIZE;
        const lamports = await program.provider.connection.getMinimumBalanceForRentExemption(mintLen + metadataExtensionLen + metadataSize);

        const mintTx = new anchor.web3.Transaction().add(
            anchor.web3.SystemProgram.createAccount({
                fromPubkey: adminKeypair.publicKey,
                newAccountPubkey: fragSOLTokenMintKeypair.publicKey,
                lamports: lamports,
                space: mintLen,
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
            [adminKeypair, fragSOLTokenMintKeypair],
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
        tokenMint_inf = await spl.getMint(
            program.provider.connection,
            tokenMintAddress_inf,
        );

        expect(tokenMint_bSOL.mintAuthority.toString()).to.equal(tokenMintAuthorityKeypair_all.publicKey.toString());
        expect(tokenMint_mSOL.mintAuthority.toString()).to.equal(tokenMintAuthorityKeypair_all.publicKey.toString());
        expect(tokenMint_jitoSOL.mintAuthority.toString()).to.equal(tokenMintAuthorityKeypair_all.publicKey.toString());
        expect(tokenMint_inf.mintAuthority.toString()).to.equal(tokenMintAuthorityKeypair_all.publicKey.toString());
    });

    it("Initialize fund, reward accounts and configure token mint", async function () {
        const solWithdrawalFeeRate = 10;
        const solCap = new anchor.BN(1_000_000_000 * 10000);
        const tokenCap1 = new anchor.BN(1_000_000_000 * 1000);
        const tokenPricingSource1 = {
            "splStakePool": {
                address: stakePoolAddress_bSOL,
            }
        };
        const tokenCap2 = new anchor.BN(1_000_000_000 * 2000);
        const tokenPricingSource2 = {
            "marinadeStakePool": {
                address: stakePoolAddress_mSOL,
            }
        };

        const rewardName = "FPoint";
        const rewardDescription = "FPoint is good";
        const rewardType = {
            "point": {
                decimals: 4,
            }
        };

        const batchProcessingThresholdAmount = new anchor.BN(0);
        const batchProcessingThresholdDuration = new anchor.BN(0);
        
        // initializes
        const initializeFundIx = await program.methods
            .fundInitialize()
            .instruction();
        const updateSolCapacityAmountIx = await program.methods
            .fundUpdateSolCapacityAmount(solCap)
            .instruction();
        const updateSolWithdrawalFeeRateIx = await program.methods
            .fundUpdateSolWithdrawalFeeRate(solWithdrawalFeeRate)
            .instruction();
        const updateWithdrawalEnabledFlagIx = await program.methods
            .fundUpdateWithdrawalEnabledFlag(true)
            .instruction();
        const updateBatchProcessingThresholdIx = await program.methods
            .fundUpdateBatchProcessingThreshold(
                batchProcessingThresholdAmount,
                batchProcessingThresholdDuration,
            )
            .instruction();
        const initializeSupportedTokenIx = async (
            supportedTokenMint: anchor.web3.PublicKey,
            tokenProgram: anchor.web3.PublicKey
        ) => {
            return await program.methods
                .fundInitializeSupportedToken()
                .accounts({
                    supportedTokenMint,
                    tokenProgram,
                })
                .instruction();
        };
        const addSupportedTokenIx = async(
            supportedTokenMint: anchor.web3.PublicKey,
            tokenProgram: anchor.web3.PublicKey,
            capacityAmount: anchor.BN,
            pricingSource,
        ) => {
            return await program.methods
                .fundAddSupportedToken(
                    capacityAmount,
                    pricingSource,
                )
                .accounts({
                    supportedTokenMint,
                    tokenProgram,
                })
                .instruction();
        };
        const initializeRewardIx = await program.methods
            .rewardInitialize()
            .accounts({ rewardAccount: fragSOLRewardAddress })
            .instruction();
        const reallocIfNeededRewardIx = await program.methods
            .rewardReallocIfNeeded(null, false)
            .accounts({ rewardAccount: fragSOLRewardAddress })
            .instruction();
        const reallocIfNeededRewardWithAssertionIx = await program.methods
            .rewardReallocIfNeeded(null, true)
            .accounts({ rewardAccount: fragSOLRewardAddress })
            .instruction();

        const addRewardIx = async (
            rewardName: string,
            rewardDescription: string,
            rewardType,
            rewardTokenMint?: anchor.web3.PublicKey,
        ) => {
            if (rewardTokenMint == null) {
                rewardTokenMint = program.programId;
            }
            return await program.methods
                .rewardAddReward(
                    rewardName,
                    rewardDescription,
                    rewardType,
                )
                .accounts({
                    rewardTokenMint,
                })
                .instruction();
        };
        const addRewardPoolIx = async (
            name: string,
            customContributionAccrualRateEnabled: boolean,
            receiptTokenMint: anchor.web3.PublicKey,
        ) => {
            return await program.methods
                .rewardAddRewardPool(
                    name,
                    null,
                    customContributionAccrualRateEnabled,
                )
                .accounts({
                    receiptTokenMint: receiptTokenMint,
                })
                .instruction();
        };
        const initializeExtraAccountMetaListIx = await program.methods
            .tokenInitializeExtraAccountMetaList()
            .accounts({
                payer: adminKeypair.publicKey,
            })
            .instruction();
        const initializePayerAccountIx = await program.methods
            .tokenInitializePayerAccount()
            .instruction();
        const addPayerAccountLamportsIx = await program.methods
            .tokenAddPayerAccountLamports(new anchor.BN(10_000_000_000))
            .instruction();

        const tx1 = new anchor.web3.Transaction().add(
            initializeFundIx,
            updateSolCapacityAmountIx,
            updateSolWithdrawalFeeRateIx,
            updateWithdrawalEnabledFlagIx,
            updateBatchProcessingThresholdIx,
            await initializeSupportedTokenIx(tokenMint_bSOL.address, spl.TOKEN_PROGRAM_ID),
            await initializeSupportedTokenIx(tokenMint_mSOL.address, spl.TOKEN_PROGRAM_ID),
            await initializeSupportedTokenIx(tokenMint_jitoSOL.address, spl.TOKEN_PROGRAM_ID),
            await initializeSupportedTokenIx(tokenMint_inf.address, spl.TOKEN_PROGRAM_ID),
            await addSupportedTokenIx(tokenMint_bSOL.address, spl.TOKEN_PROGRAM_ID, tokenCap1, tokenPricingSource1),
            await addSupportedTokenIx(tokenMint_mSOL.address, spl.TOKEN_PROGRAM_ID, tokenCap2, tokenPricingSource2),
        );
        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            tx1,
            [wallet.payer],
        );

        const tx2 = new anchor.web3.Transaction().add(
            initializeRewardIx,
            reallocIfNeededRewardIx,
            reallocIfNeededRewardIx,
            reallocIfNeededRewardIx,
            reallocIfNeededRewardIx,
            reallocIfNeededRewardWithAssertionIx,
            await addRewardIx(rewardName, rewardDescription, rewardType),
            await addRewardPoolIx("fragmetricBase", false, fragSOLTokenMintKeypair.publicKey),
            await addRewardPoolIx("fragmetricBonus", true, fragSOLTokenMintKeypair.publicKey),
        )
        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            tx2,
            [wallet.payer],
        );

        const tx3 = new anchor.web3.Transaction().add(
            initializeExtraAccountMetaListIx,
            initializePayerAccountIx,
            // set receipt token mint authority to pda
            await program.methods
                .tokenSetReceiptTokenMintAuthority()
                .accounts({})
                .signers([])
                .instruction(),
            addPayerAccountLamportsIx,
        );
        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            tx3,
            [adminKeypair],
        );

        // check fund initialized correctly
        const tokensInitialized = (await program.account.fund.fetch(fragSOLFundAddress)).supportedTokens;

        // expect(tokensInitialized[0].mint.toString()).to.eq(tokenMint1.toString());
        expect(tokensInitialized[0].mint.toString()).to.eq(tokenMint_bSOL.address.toString());
        expect(tokensInitialized[0].capacityAmount.toNumber()).to.eq(tokenCap1.toNumber());
        expect(tokensInitialized[0].operationReservedAmount.toNumber()).to.eq(0);

        // expect(tokensInitialized[1].mint.toString()).to.eq(tokenMint2.toString());
        expect(tokensInitialized[1].mint.toString()).to.eq(tokenMint_mSOL.address.toString());
        expect(tokensInitialized[1].capacityAmount.toNumber()).to.equal(tokenCap2.toNumber());
        expect(tokensInitialized[1].operationReservedAmount.toNumber()).to.eq(0);

        const rewardInitialized = await program.account.rewardAccount.fetch(fragSOLRewardAddress);
        
        expect(rewardInitialized.dataVersion).to.equal(1);
        const receiptTokenMintAccount = await spl.getMint(program.provider.connection, fragSOLTokenMintKeypair.publicKey, undefined, TOKEN_2022_PROGRAM_ID);
        expect(receiptTokenMintAccount.mintAuthority.toString()).to.equal(fragSOLTokenMintAuthorityAddress.toString());
    });

    // Devnet only
    it("Initialize for devnet", async function () {
        if (!utils.isDevnet(program.provider.connection)) {
            this.skip();
        }

        const solWithdrawalFeeRate = 10;
        const solCap = new anchor.BN(1_000_000_000).mul(new anchor.BN(1_000_000_000));
        const tokenCap1 = new anchor.BN(1_000_000_000).mul(new anchor.BN(1_000_000_000));
        const tokenPricingSource1 = {
            "splStakePool": {
                address: stakePoolAddress_bSOL,
            }
        };
        const tokenCap2 = new anchor.BN(1_000_000_000).mul(new anchor.BN(1_000_000_000));
        const tokenPricingSource2 = {
            "marinadeStakePool": {
                address: stakePoolAddress_mSOL,
            }
        };
        const tokenCap3 = new anchor.BN(1_000_000_000).mul(new anchor.BN(1_000_000_000));
        // const tokenPricingSource3 = {
        //     "splStakePool": {
        //         address: jitoSOLStakePoolPublicKey,
        //     }
        // };

        const batchProcessingThresholdAmount = new anchor.BN(10000);
        const batchProcessingThresholdDuration = new anchor.BN(600);

        const tx = new anchor.web3.Transaction().add(
            // initializes
            await program.methods
                .fundInitialize()
                .accounts({})
                .signers([])
                .instruction(),
            await program.methods
                .fundUpdateSolCapacityAmount(solCap)
                .accounts({})
                .signers([])
                .instruction(),
            await program.methods
                .fundUpdateSolWithdrawalFeeRate(solWithdrawalFeeRate)
                .accounts({})
                .signers([])
                .instruction(),
            await program.methods
                .fundUpdateBatchProcessingThreshold(
                    batchProcessingThresholdAmount,
                    batchProcessingThresholdDuration
                )
                .accounts({})
                .signers([])
                .instruction(),
            await program.methods
                .fundUpdateWithdrawalEnabledFlag(true)
                .accounts({})
                .signers([])
                .instruction(),
            // initialize extra account meta list (this will be needed for Mainnet Beta)
            // await program.methods
            //     .tokenInitializeExtraAccountMetaList()
            //     .accounts({
            //         payer: mintOwner.publicKey,
            //     })
            //     .signers([mintOwner.payer])
            //     .instruction(),

            // add supported tokens (bSOL, mSOL)
            await program.methods
                .fundAddSupportedToken(tokenCap1, tokenPricingSource1)
                .accounts({
                    supportedTokenMint: tokenMintAddress_bSOL,
                    tokenProgram: spl.TOKEN_PROGRAM_ID,
                })
                .signers([])
                .instruction(),
            await program.methods
                .fundAddSupportedToken(tokenCap2, tokenPricingSource2)
                .accounts({
                    supportedTokenMint: tokenMintAddress_mSOL,
                    tokenProgram: spl.TOKEN_PROGRAM_ID,
                })
                .signers([])
                .instruction(),
        );
        const txSig = await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            tx,
            [adminKeypair],
            { commitment: "confirmed" },
        );
        console.log(`initialize txSig: ${txSig}`);
    });
});
