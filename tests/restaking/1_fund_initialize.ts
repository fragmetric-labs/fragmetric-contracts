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
// program accounts
export let fund_pda: anchor.web3.PublicKey;
export let fund_token_authority_pda: anchor.web3.PublicKey;

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


export const fund_initialize = describe("fund_initialize", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.Restaking as Program<Restaking>;
    console.log(`programId:     ${program.programId}`);
    console.log(`rpcEndpoint:   ${program.provider.connection.rpcEndpoint}`)

    const admin = (program.provider as anchor.AnchorProvider).wallet as anchor.Wallet;
    const payer = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user1.json")));
    console.log(`Payer(user1.json) key: ${payer.publicKey}`);

    // Localnet
    before("Sol airdrop to payer", async () => {
        if (utils.isLocalnet(program.provider.connection)) {
            await utils.requestAirdrop(program.provider, payer, 10);
            console.log("======= Sol airdrop to payer =======");
        }
    });

    before("Prepare program accounts", async () => {
        receiptTokenMint = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("./fragsolMint.json")));
        [fund_token_authority_pda] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("fund_token_authority"), receiptTokenMint.publicKey.toBuffer()],
            program.programId
        );
        [fund_pda] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("fund"), receiptTokenMint.publicKey.toBuffer()],
            program.programId
        );

        // NEED TO CHECK: receiptTokenMint == createMint result account
        console.log(`admin                      = ${admin.publicKey}`);
        console.log(`receiptTokenMint           = ${receiptTokenMint.publicKey}`);
        console.log(`fund_pda                   = ${fund_pda}`);
        console.log(`fund_token_authority_pda   = ${fund_token_authority_pda}`);
        console.log("======= Prepare program accounts =======");
    });

    before("Prepare mainnet token mint accounts for localnet", async () => {
        // utils.changeMintAuthority(payer.publicKey.toString(), "./tests/restaking/clones/mainnet/bSOL_mint.json");
        // utils.changeMintAuthority(payer.publicKey.toString(), "./tests/restaking/clones/mainnet/mSOL_mint.json");
        // utils.changeMintAuthority(payer.publicKey.toString(), "./tests/restaking/clones/mainnet/JitoSOL_mint.json");
        // utils.changeMintAuthority(payer.publicKey.toString(), "./tests/restaking/clones/mainnet/INF_mint.json");
        bSOLMintPublicKey = new anchor.web3.PublicKey(fs.readFileSync("./tests/restaking/lsts/mainnet/addresses/bSOL_mint", {encoding: "utf8"}).replace(/"/g, ''));
        mSOLMintPublicKey = new anchor.web3.PublicKey(fs.readFileSync("./tests/restaking/lsts/mainnet/addresses/mSOL_mint", {encoding: "utf8"}).replace(/"/g, ''));
        jitoSOLMintPublicKey = new anchor.web3.PublicKey(fs.readFileSync("./tests/restaking/lsts/mainnet/addresses/JitoSOL_mint", {encoding: "utf8"}).replace(/"/g, ''));
        infMintPublicKey = new anchor.web3.PublicKey(fs.readFileSync("./tests/restaking/lsts/mainnet/addresses/INF_mint", {encoding: "utf8"}).replace(/"/g, ''));
        console.log("======= Prepare mainnet token mint accounts for localnet =======");
    });

    // Localnet only: Already created in devnet
    it("Create receipt token mint with Transfer Hook extension", async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        // generate keypair to use as address for the transfer-hook enabled mint account
        const mintOwner = admin; // same as admin
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
                fund_token_authority_pda,
                program.programId,
                TOKEN_2022_PROGRAM_ID,
            ),
            spl.createInitializeMintInstruction(
                receiptTokenMint.publicKey,
                decimals,
                fund_token_authority_pda,
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
        expect(receiptTokenMintAccount.mintAuthority.toString()).to.equal(fund_token_authority_pda.toString());
        expect(receiptTokenMintAccount.freezeAuthority.toString()).to.equal(mintOwner.publicKey.toString());
    });

    // Devnet only
    it("Create test token mint accounts for initialize", async function () {
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
    it("Initialize fund and fundTokenAuthority", async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        const solWithdrawalFeeRate = 10;
        const tokenCap1 = new anchor.BN(1_000_000_000 * 1000);
        const tokenCap2 = new anchor.BN(1_000_000_000 * 2000);
        const tokenCap3 = new anchor.BN(1_000_000_000 * 3000);
        const tokenCap4 = new anchor.BN(1_000_000_000 * 4000);
 
        const txs = new anchor.web3.Transaction().add(
            await program.methods
                .fundInitialize()
                .accounts({})
                .signers([])
                .instruction(),
            await program.methods
                .fundInitializeSolWithdrawalFeeRate(solWithdrawalFeeRate)
                .accounts({})
                .signers([])
                .instruction(),
            await program.methods
                .fundAddSupportedToken(tokenCap1)
                .accounts({
                    tokenMint: bSOLMint.address,
                    tokenProgram: spl.TOKEN_PROGRAM_ID,
                })
                .signers([])
                .instruction(),
            await program.methods
                .fundAddSupportedToken(tokenCap2)
                .accounts({
                    tokenMint: mSOLMint.address,
                    tokenProgram: spl.TOKEN_PROGRAM_ID,
                })
                .signers([])
                .instruction(),
            await program.methods
                .fundAddSupportedToken(tokenCap3)
                .accounts({
                    tokenMint: jitoSOLMint.address,
                    tokenProgram: spl.TOKEN_PROGRAM_ID,
                })
                .signers([])
                .instruction(),
            await program.methods
                .fundAddSupportedToken(tokenCap4)
                .accounts({
                    tokenMint: infMint.address,
                    tokenProgram: spl.TOKEN_PROGRAM_ID,
                })
                .signers([])
                .instruction(),
        );
        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            txs,
            [admin.payer],
        );

        // check fund initialized correctly
        const tokensInitialized = (await program.account.fund.fetch(fund_pda)).supportedTokens;

        // expect(tokensInitialized[0].address.toString()).to.eq(tokenMint1.toString());
        expect(tokensInitialized[0].address.toString()).to.eq(bSOLMint.address.toString());
        expect(tokensInitialized[0].tokenCap.toNumber()).to.eq(tokenCap1.toNumber());
        expect(tokensInitialized[0].tokenAmountIn.toNumber()).to.eq(0);

        // expect(tokensInitialized[1].address.toString()).to.eq(tokenMint2.toString());
        expect(tokensInitialized[1].address.toString()).to.eq(mSOLMint.address.toString());
        expect(tokensInitialized[1].tokenCap.toNumber()).to.equal(tokenCap2.toNumber());
        expect(tokensInitialized[1].tokenAmountIn.toNumber()).to.eq(0);

        expect(tokensInitialized[2].address.toString()).to.eq(jitoSOLMint.address.toString());
        expect(tokensInitialized[2].tokenCap.toNumber()).to.equal(tokenCap3.toNumber());
        expect(tokensInitialized[2].tokenAmountIn.toNumber()).to.eq(0);

        expect(tokensInitialized[3].address.toString()).to.eq(infMint.address.toString());
        expect(tokensInitialized[3].tokenCap.toNumber()).to.equal(tokenCap4.toNumber());
        expect(tokensInitialized[3].tokenAmountIn.toNumber()).to.eq(0);
    });
});
