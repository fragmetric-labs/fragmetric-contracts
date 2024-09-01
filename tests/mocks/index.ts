import fs from "fs";
import {Buffer} from "buffer";
import * as anchor from "@coral-xyz/anchor";

const changeMintAuthority = (newMintAuthority: anchor.web3.PublicKey, mintFilePath: string) => {
    // Load the mint file and parse JSON
    let mintRaw = fs.readFileSync(mintFilePath, "utf8");
    let mint = JSON.parse(mintRaw);
    let data = Uint8Array.from(Buffer.from(mint["account"]["data"][0], "base64"));
    let newMintAuthorityBytes = newMintAuthority.toBytes();

    // Replace part of the data with the new mint authority bytes
    for (let i = 0; i < newMintAuthorityBytes.length; i++) {
        data[4 + i] = newMintAuthorityBytes[i];
    }

    // Encode the data back to base64
    mint["account"]["data"][0] = Buffer.from(data).toString("base64");

    // Write the updated JSON back to the file
    // console.log(mintFilePath, mint)
    fs.writeFileSync(mintFilePath, JSON.stringify(mint, null, 0));
}

export {
    changeMintAuthority,
}