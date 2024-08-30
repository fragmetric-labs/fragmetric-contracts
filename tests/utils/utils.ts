import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import * as fs from "fs";
import { Buffer } from "buffer";
import * as bs58 from "bs58";

const requestAirdrop = async (provider: anchor.Provider, user: anchor.web3.Keypair, amount: number) => {
    let airdropSignature = await provider.connection.requestAirdrop(
       user.publicKey,
       amount * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSignature);
    const userBal = await provider.connection.getBalance(user.publicKey);
    console.log(`user ${user.publicKey} SOL balance: ${userBal}`);
}

const changeMintAuthority = (mintAuthority: string, mintFilePath: string) => {
    // Load the mint file and parse JSON
    let mint = fs.readFileSync(mintFilePath, "utf8");
    mint = JSON.parse(mint);
    // Decode the base64 data
    let data = Uint8Array.from(Buffer.from(mint["account"]["data"][0], "base64"));
    // Decode the mint authority from base58
    let mintAuthorityBytes = bs58.default.decode(mintAuthority);
    // Replace part of the data with the new mint authority bytes
    for (let i = 0; i < mintAuthorityBytes.length; i++) {
        data[4 + i] = mintAuthorityBytes[i];
    }
    // Encode the data back to base64
    let encodedData = Buffer.from(data).toString("base64");
    // Update the JSON structure
    mint["account"]["data"][0] = encodedData;
    // Write the updated JSON back to the file
    // fs.writeFileSync(mintFilePath, JSON.stringify(mint, null, 2));
    fs.writeFileSync(mintFilePath, JSON.stringify(mint));
}

function isLocalnet(conn: anchor.web3.Connection): boolean {
    return conn.rpcEndpoint == "http://0.0.0.0:8899";
}

function isDevnet(conn: anchor.web3.Connection): boolean {
    return conn.rpcEndpoint == anchor.web3.clusterApiUrl("devnet");
}

function isMainnetBeta(conn: anchor.web3.Connection): boolean {
    return conn.rpcEndpoint == anchor.web3.clusterApiUrl("mainnet-beta");
}

export {
    requestAirdrop,
    changeMintAuthority,
    isLocalnet,
    isDevnet,
    isMainnetBeta as isMainnet,
};
