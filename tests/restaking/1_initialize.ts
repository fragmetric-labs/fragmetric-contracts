import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import type { TokenMetadata } from "@solana/spl-token-metadata";
import { createInitializeInstruction, createUpdateFieldInstruction, pack } from "@solana/spl-token-metadata";
import { expect } from "chai";
import { Restaking } from "../../target/types/restaking";
import { before } from "mocha";
import * as fs from "fs";
import * as utils from "../utils/utils";

export let receiptTokenMint: anchor.web3.Keypair;
export let rewardAccount: anchor.web3.Keypair;
// program accounts
export let fund_pda: anchor.web3.PublicKey;
export let receipt_token_lock_authority_pda: anchor.web3.PublicKey;
export let receipt_token_mint_authority_pda: anchor.web3.PublicKey;
export let bSOL_token_authority_pda: anchor.web3.PublicKey;
export let mSOL_token_authority_pda: anchor.web3.PublicKey;
export let jitoSOL_token_authority_pda: anchor.web3.PublicKey;
export let inf_token_authority_pda: anchor.web3.PublicKey;
export let receipt_token_lock_account_pda: anchor.web3.PublicKey;

// for devnet
export let tokenMint1: anchor.web3.PublicKey;
export let tokenMint2: anchor.web3.PublicKey;
// for localnet
export let bSOLMintPublicKey: anchor.web3.PublicKey;
export let mSOLMintPublicKey: anchor.web3.PublicKey;
export let jitoSOLMintPublicKey: anchor.web3.PublicKey;
export let infMintPublicKey: anchor.web3.PublicKey;
export let bSOLMint: spl.Mint;
export let mSOLMint: spl.Mint;
export let jitoSOLMint: spl.Mint;
export let infMint: spl.Mint;
export let bSOLStakePoolPublicKey: anchor.web3.PublicKey;
export let mSOLStakePoolPublicKey: anchor.web3.PublicKey;
export let jitoSOLStakePoolPublicKey: anchor.web3.PublicKey;

export const initialize = describe("initialize everything", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.Restaking as Program<Restaking>;
    console.log(`programId:     ${program.programId}`);
    console.log(`rpcEndpoint:   ${program.provider.connection.rpcEndpoint}`)

    const admin = (program.provider as anchor.AnchorProvider).wallet as anchor.Wallet;
    const mintOwner = admin;
    const payer = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user1.json")));
    console.log(`Payer(user1.json) key: ${payer.publicKey}`);

    let receiptTokenMintMetadata: TokenMetadata;

    // Localnet
    before("Sol airdrop to payer", async () => {
        if (utils.isLocalnet(program.provider.connection)) {
            await utils.requestAirdrop(program.provider, payer, 10);
            console.log("======= Sol airdrop to payer =======");
        }
    });

    before("Prepare mainnet token mint/stake pool accounts for localnet", async () => {
        // utils.changeMintAuthority(payer.publicKey.toString(), "./tests/restaking/clones/mainnet/bSOL_mint.json");
        // utils.changeMintAuthority(payer.publicKey.toString(), "./tests/restaking/clones/mainnet/mSOL_mint.json");
        // utils.changeMintAuthority(payer.publicKey.toString(), "./tests/restaking/clones/mainnet/JitoSOL_mint.json");
        // utils.changeMintAuthority(payer.publicKey.toString(), "./tests/restaking/clones/mainnet/INF_mint.json");
        bSOLMintPublicKey = new anchor.web3.PublicKey(fs.readFileSync("./tests/restaking/lsts/mainnet/addresses/bSOL_mint", {encoding: "utf8"}).replace(/"/g, ''));
        bSOLStakePoolPublicKey = new anchor.web3.PublicKey(fs.readFileSync("./tests/restaking/lsts/devnet/addresses/bSOL_stake_pool", {encoding: "utf8"}).replace(/"/g, ''));
        mSOLMintPublicKey = new anchor.web3.PublicKey(fs.readFileSync("./tests/restaking/lsts/mainnet/addresses/mSOL_mint", {encoding: "utf8"}).replace(/"/g, ''));
        mSOLStakePoolPublicKey = new anchor.web3.PublicKey(fs.readFileSync("./tests/restaking/lsts/mainnet/addresses/mSOL_stake_pool", {encoding: "utf8"}).replace(/"/g, ''));
        jitoSOLMintPublicKey = new anchor.web3.PublicKey(fs.readFileSync("./tests/restaking/lsts/mainnet/addresses/JitoSOL_mint", {encoding: "utf8"}).replace(/"/g, ''));
        jitoSOLStakePoolPublicKey = new anchor.web3.PublicKey(fs.readFileSync("./tests/restaking/lsts/mainnet/addresses/JitoSOL_stake_pool", {encoding: "utf8"}).replace(/"/g, ''));
        infMintPublicKey = new anchor.web3.PublicKey(fs.readFileSync("./tests/restaking/lsts/mainnet/addresses/INF_mint", {encoding: "utf8"}).replace(/"/g, ''));
        console.log("======= Prepare mainnet token mint accounts for localnet =======");
    });

    before("Prepare program accounts", async () => {
        receiptTokenMint = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("./fragsolMint.json")));
        rewardAccount = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("./rewardAccount.json")));
        receiptTokenMintMetadata = {
            mint: receiptTokenMint.publicKey,
            name: "fragSOL",
            symbol: "FRAGSOL",
            uri: "https://quicknode.quicknode-ipfs.com/ipfs/QmdTSAPXn8dT2mnS2rxMouedauhjzJeYA74Hmez5P3ZEgE",
            additionalMetadata: [["description", "Fragmetric Liquid Restaking Token"]],
            updateAuthority: mintOwner.publicKey,
        };
        [receipt_token_lock_authority_pda] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("receipt_token_lock_authority"), receiptTokenMint.publicKey.toBuffer()],
            program.programId
        );
        [receipt_token_mint_authority_pda] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("receipt_token_mint_authority"), receiptTokenMint.publicKey.toBuffer()],
            program.programId
        );
        [bSOL_token_authority_pda] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("supported_token_authority"), receiptTokenMint.publicKey.toBuffer(), bSOLMintPublicKey.toBuffer()],
            program.programId
        );
        [mSOL_token_authority_pda] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("supported_token_authority"), receiptTokenMint.publicKey.toBuffer(), mSOLMintPublicKey.toBuffer()],
            program.programId
        );
        [jitoSOL_token_authority_pda] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("supported_token_authority"), receiptTokenMint.publicKey.toBuffer(), jitoSOLMintPublicKey.toBuffer()],
            program.programId
        );
        [inf_token_authority_pda] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("supported_token_authority"), receiptTokenMint.publicKey.toBuffer(), infMintPublicKey.toBuffer()],
            program.programId
        );
        [fund_pda] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("fund"), receiptTokenMint.publicKey.toBuffer()],
            program.programId
        );
        [receipt_token_lock_account_pda] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("receipt_token_lock_account"), receiptTokenMint.publicKey.toBuffer()],
            program.programId
        );

        // NEED TO CHECK: receiptTokenMint == createMint result account
        console.log(`admin                              = ${admin.publicKey}`);
        console.log(`receiptTokenMint                   = ${receiptTokenMint.publicKey}`);
        console.log(`reward account                     = ${rewardAccount.publicKey}`);
        console.log(`fund_pda                           = ${fund_pda}`);
        console.log(`receipt_token_lock_authority_pda   = ${receipt_token_lock_authority_pda}`);
        console.log(`receipt_token_mint_authority_pda   = ${receipt_token_mint_authority_pda}`);
        console.log(`bSOL_token_authority_pda           = ${bSOL_token_authority_pda}`);
        console.log(`mSOL_token_authority_pda           = ${mSOL_token_authority_pda}`);
        console.log(`jitoSOL_token_authority_pda        = ${jitoSOL_token_authority_pda}`);
        console.log(`inf_token_authority_pda            = ${inf_token_authority_pda}`);
        console.log(`receipt_token_lock_account_pda     = ${receipt_token_lock_account_pda}`);
        console.log("======= Prepare program accounts =======");
    });

    // Devnet only
    it.skip("Create test token mint accounts for initialize", async function () {
        if (!utils.isDevnet(program.provider.connection)) {
            this.skip();
        }

        tokenMint1 = await spl.createMint(
            program.provider.connection,
            payer,
            payer.publicKey,
            payer.publicKey,
            9,
            undefined,
            undefined,
            TOKEN_2022_PROGRAM_ID
        );
        tokenMint2 = await spl.createMint(
            program.provider.connection,
            payer,
            payer.publicKey,
            payer.publicKey,
            9,
            undefined,
            undefined,
            TOKEN_2022_PROGRAM_ID
        );
    });

    // NEED ONLY ONCE AT OFF-CHAIN
    it.skip("Make fragSOL token metadata file - needed only once", async () => {
        const receiptTokenMintMetadataJson = {
            name: receiptTokenMintMetadata.name,
            symbol: receiptTokenMintMetadata.symbol,
            description: receiptTokenMintMetadata.additionalMetadata[0][1],
            image: "https://quicknode.quicknode-ipfs.com/ipfs/QmayYcry2mJGHmcYMn1mqiqxR9kkQXtE3uBEzR9y84vQVL",
            // attributes: [],
        };
        fs.writeFileSync("./tests/restaking/fragSOLMetadata.json", JSON.stringify(receiptTokenMintMetadataJson, null, 0));
    });

    // Localnet only: Already created in devnet
    it("Create receipt token mint with Transfer Hook extension", async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        // generate keypair to use as address for the transfer-hook enabled mint account
        const decimals = 9;

        const metadataSize = pack(receiptTokenMintMetadata).length;

        const extensions = [spl.ExtensionType.TransferHook, spl.ExtensionType.MetadataPointer];
        const mintLen = spl.getMintLen(extensions);
        const metadataExtensionLen = spl.TYPE_SIZE + spl.LENGTH_SIZE;
        const lamports = await program.provider.connection.getMinimumBalanceForRentExemption(mintLen + metadataExtensionLen + metadataSize);

        const mintTx = new anchor.web3.Transaction().add(
            anchor.web3.SystemProgram.createAccount({
                fromPubkey: mintOwner.publicKey,
                newAccountPubkey: receiptTokenMint.publicKey,
                lamports: lamports,
                space: mintLen,
                programId: TOKEN_2022_PROGRAM_ID,
            }),
            spl.createInitializeTransferHookInstruction(
                receiptTokenMint.publicKey,
                mintOwner.publicKey,
                program.programId,
                TOKEN_2022_PROGRAM_ID,
            ),
            spl.createInitializeMetadataPointerInstruction(
                receiptTokenMint.publicKey,
                mintOwner.publicKey,
                receiptTokenMint.publicKey,
                spl.TOKEN_2022_PROGRAM_ID,
            ),
            spl.createInitializeMintInstruction(
                receiptTokenMint.publicKey,
                decimals,
                mintOwner.publicKey,
                null, // TODO: freeze authority to be null
                spl.TOKEN_2022_PROGRAM_ID,
            ),
            createInitializeInstruction({
                programId: spl.TOKEN_2022_PROGRAM_ID,
                mint: receiptTokenMint.publicKey,
                metadata: receiptTokenMint.publicKey,
                name: receiptTokenMintMetadata.name,
                symbol: receiptTokenMintMetadata.symbol,
                uri: receiptTokenMintMetadata.uri,
                mintAuthority: mintOwner.publicKey, // receipt_token_mint_authority_pda -> need signature
                updateAuthority: receiptTokenMintMetadata.updateAuthority,
            }),
            createUpdateFieldInstruction({
                programId: spl.TOKEN_2022_PROGRAM_ID,
                metadata: receiptTokenMint.publicKey,
                updateAuthority: receiptTokenMintMetadata.updateAuthority,
                field: receiptTokenMintMetadata.additionalMetadata[0][0],
                value: receiptTokenMintMetadata.additionalMetadata[0][1],
            }),
        );
        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            mintTx,
            [mintOwner.payer, receiptTokenMint],
        );

        const receiptTokenMintAccount = await spl.getMint(program.provider.connection, receiptTokenMint.publicKey, undefined, TOKEN_2022_PROGRAM_ID);
        expect(receiptTokenMintAccount.address.toString()).to.equal(receiptTokenMint.publicKey.toString());
        // expect(receiptTokenMintAccount.freezeAuthority.toString()).to.equal(mintOwner.publicKey.toString());
    });

    // Localnet only
    it("Set mainnet token mint accounts for initialize", async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        bSOLMint = await spl.getMint(
            program.provider.connection,
            bSOLMintPublicKey,
        );
        mSOLMint = await spl.getMint(
            program.provider.connection,
            mSOLMintPublicKey,
        );
        jitoSOLMint = await spl.getMint(
            program.provider.connection,
            jitoSOLMintPublicKey,
        );
        infMint = await spl.getMint(
            program.provider.connection,
            infMintPublicKey,
        );

        expect(bSOLMint.mintAuthority.toString()).to.equal(payer.publicKey.toString());
        expect(mSOLMint.mintAuthority.toString()).to.equal(payer.publicKey.toString());
        expect(jitoSOLMint.mintAuthority.toString()).to.equal(payer.publicKey.toString());
        expect(infMint.mintAuthority.toString()).to.equal(payer.publicKey.toString());
    });

    // Localnet only
    it("Initialize fund, extraAccountMetaList and mint authority", async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        const solWithdrawalFeeRate = 10;
        const solCap = new anchor.BN(1_000_000_000 * 10000);
        const tokenCap1 = new anchor.BN(1_000_000_000 * 1000);
        const tokenPricingSource1 = {
            "splStakePool": {
                address: bSOLStakePoolPublicKey,
            }
        };
        const tokenCap2 = new anchor.BN(1_000_000_000 * 2000);
        const tokenPricingSource2 = {
            "marinadeStakePool": {
                address: mSOLStakePoolPublicKey,
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
        const createRewardAccountIx = await program.account
            .rewardAccount
            .createInstruction(
                rewardAccount,
                2097152,
            );
        const initializeRewardIx = await program.methods
            .rewardInitialize()
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
                payer: admin.publicKey,
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
            await initializeSupportedTokenIx(bSOLMint.address, spl.TOKEN_PROGRAM_ID),
            await initializeSupportedTokenIx(mSOLMint.address, spl.TOKEN_PROGRAM_ID),
            await initializeSupportedTokenIx(jitoSOLMint.address, spl.TOKEN_PROGRAM_ID),
            await initializeSupportedTokenIx(infMint.address, spl.TOKEN_PROGRAM_ID),
            await addSupportedTokenIx(bSOLMint.address, spl.TOKEN_PROGRAM_ID, tokenCap1, tokenPricingSource1),
            await addSupportedTokenIx(mSOLMint.address, spl.TOKEN_PROGRAM_ID, tokenCap2, tokenPricingSource2),
        );
        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            tx1,
            [admin.payer],
        );

        const tx2 = new anchor.web3.Transaction().add(
            createRewardAccountIx,
            initializeRewardIx,
            await addRewardIx(rewardName, rewardDescription, rewardType),
            await addRewardPoolIx("fragmetricBase", false, receiptTokenMint.publicKey),
            await addRewardPoolIx("fragmetricBonus", true, receiptTokenMint.publicKey),
        )
        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            tx2,
            [admin.payer, rewardAccount],
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
            [admin.payer],
        );

        // check fund initialized correctly
        const tokensInitialized = (await program.account.fund.fetch(fund_pda)).supportedTokens;

        // expect(tokensInitialized[0].mint.toString()).to.eq(tokenMint1.toString());
        expect(tokensInitialized[0].mint.toString()).to.eq(bSOLMint.address.toString());
        expect(tokensInitialized[0].capacityAmount.toNumber()).to.eq(tokenCap1.toNumber());
        expect(tokensInitialized[0].operationReservedAmount.toNumber()).to.eq(0);

        // expect(tokensInitialized[1].mint.toString()).to.eq(tokenMint2.toString());
        expect(tokensInitialized[1].mint.toString()).to.eq(mSOLMint.address.toString());
        expect(tokensInitialized[1].capacityAmount.toNumber()).to.equal(tokenCap2.toNumber());
        expect(tokensInitialized[1].operationReservedAmount.toNumber()).to.eq(0);

        const rewardInitialized = await program.account.rewardAccount.fetch(rewardAccount.publicKey);
        
        expect(rewardInitialized.dataVersion).to.equal(1);
        const receiptTokenMintAccount = await spl.getMint(program.provider.connection, receiptTokenMint.publicKey, undefined, TOKEN_2022_PROGRAM_ID);
        expect(receiptTokenMintAccount.mintAuthority.toString()).to.equal(receipt_token_mint_authority_pda.toString());
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
                address: bSOLStakePoolPublicKey,
            }
        };
        const tokenCap2 = new anchor.BN(1_000_000_000).mul(new anchor.BN(1_000_000_000));
        const tokenPricingSource2 = {
            "marinadeStakePool": {
                address: mSOLStakePoolPublicKey,
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
                    supportedTokenMint: bSOLMintPublicKey,
                    tokenProgram: spl.TOKEN_PROGRAM_ID,
                })
                .signers([])
                .instruction(),
            await program.methods
                .fundAddSupportedToken(tokenCap2, tokenPricingSource2)
                .accounts({
                    supportedTokenMint: mSOLMintPublicKey,
                    tokenProgram: spl.TOKEN_PROGRAM_ID,
                })
                .signers([])
                .instruction(),
        );
        const txSig = await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            tx,
            [admin.payer],
            { commitment: "confirmed" },
        );
        console.log(`initialize txSig: ${txSig}`);
    });
});
