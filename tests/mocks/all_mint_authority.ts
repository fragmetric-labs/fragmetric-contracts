import fs from "fs";
import {Buffer} from "buffer";
import * as anchor from "@coral-xyz/anchor";

if (process.argv.length > 3) {
    const mintFilePath = process.argv[process.argv.length - 1];
    console.log(`Changing mint authority of token mint at ${mintFilePath}`);
    changeMintAuthority(mintFilePath)
} else {
    throw Error("not enough arguments");
}

function changeMintAuthority(mintFilePath: string) {
    // Load the mint file and parse JSON
    let mintRaw = fs.readFileSync(mintFilePath, "utf8");
    let mint = JSON.parse(mintRaw);
    let mintData = Uint8Array.from(Buffer.from(mint["account"]["data"][0], "base64"));

    // all_mint_authority.json
    let mintAuthority = new anchor.web3.PublicKey('24z2hejEqmQGpPKU3q2xZe1ZuAzPsNeEU55KT3k629e6');
    let mintAuthorityBytes = mintAuthority.toBytes();

    // Replace part of the data with the new mint authority bytes
    mintData[0] = 1;
    for (let i = 0; i < mintAuthorityBytes.length; i++) {
        mintData[4 + i] = mintAuthorityBytes[i];
    }

    // Encode the data back to base64
    mint["account"]["data"][0] = Buffer.from(mintData).toString("base64");

    // Write the updated JSON back to the file
    // console.log(mintFilePath, mint)
    fs.writeFileSync(mintFilePath, JSON.stringify(mint, null, 2));
}
