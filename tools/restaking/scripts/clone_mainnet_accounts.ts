import fs from "fs";
import toml from "toml";
import { exec } from "child_process";

/// HOW TO USE
/// if `wantToResetAccountIndexes` array at line 42 is empty, it'll try to clone all of the accounts at Anchor.toml from mainnet
/// if you want to clone only few, then fill out the array with the account indexes wrote at Anchor.toml

/// File running command: npx tsx tools/restaking/scripts/clone_mainnet_accounts.ts

const delay = (ms: number) => {
    return new Promise(resolve => setTimeout(resolve, ms));
}

const cloneMainnetAccounts = async (accounts: Array<number>) => {
    // parse Anchor.toml
    let anchorToml = fs.readFileSync("./Anchor.toml", { encoding: "utf-8" });
    let parsed = toml.parse(anchorToml);

    let tomlAccounts = parsed["test"]["validator"]["account"];
    if (accounts.length > 0) {
        tomlAccounts = accounts.map((element) => tomlAccounts[element]);
    }

    // run cli command
    for (const account of tomlAccounts) {
        console.log(account);
        if (account["filename"].startsWith("./tests/mocks/mainnet/jito_")) continue;
        const command = `solana account -u m ${account["address"]} --output-file ${account["filename"]} --output json`;
        exec(command, (error, stdout, stderr) => {
            if (error) {
                console.error("error:", error);
            }
            if (stderr) {
                console.error("stderr:", stderr);
            }
        });
        await delay(100);
    }
}

const wantToResetAccountIndexes = [];
cloneMainnetAccounts(wantToResetAccountIndexes);
