import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import { Restaking } from "../../target/types/restaking";
import { before } from "mocha";
import * as fs from "fs";
import * as utils from "../utils/utils";

export const fund_initialize = describe("fund_initialize", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.Restaking as Program<Restaking>;
    console.log(`programId: ${program.programId}`);

    const admin = (program.provider as anchor.AnchorProvider).wallet as anchor.Wallet;
    const payer = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user1.json")));
    console.log(`Payer key: ${payer.publicKey}`);

    let receiptTokenMint: anchor.web3.Keypair;
    let tokenMint1: anchor.web3.PublicKey;
    let tokenMint2: anchor.web3.PublicKey;

    let bSOLMintPublicKey: anchor.web3.PublicKey;
    let mSOLMintPublicKey: anchor.web3.PublicKey;
    let jitoSOLMintPublicKey: anchor.web3.PublicKey;
    let infMintPublicKey: anchor.web3.PublicKey;

    let bSOLMint: spl.Mint;
    let mSOLMint: spl.Mint;
    let jitoSOLMint: spl.Mint;
    let infMint: spl.Mint;

    let fund_pda: anchor.web3.PublicKey;
    let fund_token_authority_pda: anchor.web3.PublicKey;
        
    // generate keypair to use as address for the transfer-hook enabled mint account
    const mintOwner = admin; // same as admin
    const decimals = 9;

    before("Sol airdrop", async () => {
        await utils.requestAirdrop(program.provider, payer, 10);
    });

    before("Prepare accounts", async () => {
        receiptTokenMint = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("./fragsolMint.json")));
        [fund_token_authority_pda, ] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("fund_token_authority"), receiptTokenMint.publicKey.toBuffer()],
            program.programId
        );
        [fund_pda, ] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("fund"), receiptTokenMint.publicKey.toBuffer()],
            program.programId
        );

        // NEED TO CHECK: receiptTokenMint == createMint result account
        console.log(`mintOwner: ${mintOwner.publicKey}, receiptTokenMint: ${receiptTokenMint.publicKey}, fund_pda: ${fund_pda}, fund_token_authority_pda: ${fund_token_authority_pda}`);
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
    });

    it("Create receipt token mint with Transfer Hook extension", async () => {
        const extensions = [spl.ExtensionType.TransferHook];
        const mintLen = spl.getMintLen(extensions);
        console.log(`mintLen: ${mintLen}`);
        const lamports = await program.provider.connection.getMinimumBalanceForRentExemption(mintLen);
        console.log(`lamports: ${lamports}`);

        const mintTx = new anchor.web3.Transaction().add(
            anchor.web3.SystemProgram.createAccount({ // already in use at devnet
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
    });

    it.skip("Create test token mint accounts for initialize", async () => { // for localnet
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

        fs.writeFileSync("./tests/restaking/tokenMint1", JSON.stringify(tokenMint1));
        fs.writeFileSync("./tests/restaking/tokenMint2", JSON.stringify(tokenMint2));

        const receiptTokenMintAccount = (await spl.getMint(program.provider.connection, receiptTokenMint.publicKey, undefined, TOKEN_2022_PROGRAM_ID));
        console.log("Fund =", fund_pda);
        console.log("Fund Token Authority =", fund_token_authority_pda);
        console.log("Receipt Token Mint =", receiptTokenMintAccount.address);
        console.log("It's authority =", receiptTokenMintAccount.mintAuthority);
        console.log("It's freeze authority = ", receiptTokenMintAccount.freezeAuthority);
    });

    it("Set mainnet token mint accounts for initialize", async () => { // for localnet
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
        console.log(`bSOL mintAuthority: ${bSOLMint.mintAuthority}`);
        console.log(`mSOL mintAuthority: ${mSOLMint.mintAuthority}`);
        console.log(`JitoSOL mintAuthority: ${jitoSOLMint.mintAuthority}`);
        console.log(`INF mintAuthority: ${infMint.mintAuthority}`);
    });

    it("Is initialized!", async () => {
        const solWithdrawalFeeRate = 10;
        const tokenCap1 = new anchor.BN(1_000_000_000 * 1000);
        const tokenCap2 = new anchor.BN(1_000_000_000 * 2000);
        const tokenCap3 = new anchor.BN(1_000_000_000 * 3000);
        const tokenCap4 = new anchor.BN(1_000_000_000 * 4000);

        const whitelistedTokens = [
            {
                // address: tokenMint1,
                address: bSOLMint.address,
                tokenCap: tokenCap1,
                tokenAmountIn: new anchor.BN(0),
            },
            {
                // address: tokenMint2,
                address: mSOLMint.address,
                tokenCap: tokenCap2,
                tokenAmountIn: new anchor.BN(0),
            },
            {
                address: jitoSOLMint.address,
                tokenCap: tokenCap3,
                tokenAmountIn: new anchor.BN(0),
            },
            {
                address: infMint.address,
                tokenCap: tokenCap4,
                tokenAmountIn: new anchor.BN(0),
            },
        ];
        // const whitelistedTokens = [];

        // const [receipt_token_lock_account_pda, ] = anchor.web3.PublicKey.findProgramAddressSync(
        //     [Buffer.from("receipt_lock"), receipt_token_mint_pda.toBuffer()],
        //     program.programId
        // );

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
                .fundInitializeWhitelistedTokens(whitelistedTokens)
                .accounts({})
                .signers([])
                .instruction(),
        );
        const txSig = await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            txs,
            [admin.payer],
        );

        // check fund initialized correctly
        const tokensInitialized = (await program.account.fund.fetch(fund_pda)).whitelistedTokens;

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
