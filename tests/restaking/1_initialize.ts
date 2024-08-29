import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
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
            [Buffer.from("fund_seed"), receiptTokenMint.publicKey.toBuffer()],
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

    // Localnet
    it("Create receipt token mint with Transfer Hook extension on localnet", async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        // generate keypair to use as address for the transfer-hook enabled mint account
        const decimals = 9;
    
        const extensions = [spl.ExtensionType.TransferHook];
        const mintLen = spl.getMintLen(extensions);
        const lamports = await program.provider.connection.getMinimumBalanceForRentExemption(mintLen);

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
                receipt_token_mint_authority_pda,
                program.programId,
                TOKEN_2022_PROGRAM_ID,
            ),
            spl.createInitializeMintInstruction(
                receiptTokenMint.publicKey,
                decimals,
                receipt_token_mint_authority_pda,
                mintOwner.publicKey,
                TOKEN_2022_PROGRAM_ID,
            ),
        );
        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            mintTx,
            [mintOwner.payer, receiptTokenMint],
        );

        const receiptTokenMintAccount = await spl.getMint(program.provider.connection, receiptTokenMint.publicKey, undefined, TOKEN_2022_PROGRAM_ID);
        expect(receiptTokenMintAccount.address.toString()).to.equal(receiptTokenMint.publicKey.toString());
        expect(receiptTokenMintAccount.mintAuthority.toString()).to.equal(receipt_token_mint_authority_pda.toString());
        expect(receiptTokenMintAccount.freezeAuthority.toString()).to.equal(mintOwner.publicKey.toString());
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
    it("Initialize", async function () {
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

        const batchProcessingThresholdAmount = new anchor.BN(0);
        const batchProcessingThresholdDuration = new anchor.BN(0);

        const txs = new anchor.web3.Transaction().add(
            // initializes
            await program.account.rewardAccount.createInstruction(
                rewardAccount,
                1_048_576,
            ),
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
                .fundUpdateWithdrawalEnabledFlag(true)
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
                .fundInitializeToken()
                .accounts({
                    supportedTokenMint: bSOLMint.address,
                    tokenProgram: spl.TOKEN_PROGRAM_ID,
                })
                .signers([])
                .instruction(),
            await program.methods
                .fundInitializeToken()
                .accounts({
                    supportedTokenMint: mSOLMint.address,
                    tokenProgram: spl.TOKEN_PROGRAM_ID,
                })
                .signers([])
                .instruction(),
            await program.methods
                .fundInitializeToken()
                .accounts({
                    supportedTokenMint: jitoSOLMint.address,
                    tokenProgram: spl.TOKEN_PROGRAM_ID,
                })
                .signers([])
                .instruction(),
            // await program.methods
            //     .fundInitializeToken()
            //     .accounts({
            //         supportedTokenMint: infMint.address,
            //         tokenProgram: spl.TOKEN_PROGRAM_ID,
            //     })
            //     .signers([])
            //     .instruction(),
            await program.methods
                .fundAddSupportedToken(
                    tokenCap1,
                    tokenPricingSource1,
                )
                .accounts({
                    supportedTokenMint: bSOLMint.address,
                    tokenProgram: spl.TOKEN_PROGRAM_ID,
                })
                .signers([])
                .instruction(),
            await program.methods
                .fundAddSupportedToken(tokenCap2, tokenPricingSource2)
                .accounts({
                    supportedTokenMint: mSOLMint.address,
                    tokenProgram: spl.TOKEN_PROGRAM_ID,
                })
                .signers([])
                .instruction(),
            // initialize extra account meta list
            await program.methods
                .tokenInitializeExtraAccountMetaList()
                .accounts({
                    payer: mintOwner.publicKey,
                })
                .signers([mintOwner.payer])
                .instruction(),
            await program.methods
                .rewardInitialize()
                .instruction(),
        );
        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            txs,
            [admin.payer, rewardAccount],
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
