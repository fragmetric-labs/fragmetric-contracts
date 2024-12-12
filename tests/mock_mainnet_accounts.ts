import fs from "fs";
import path from "path";
import toml from "toml";
import { execSync } from "child_process";

/// HOW TO USE
/// if `wantToResetAccountIndexes` array at line 42 is empty, it'll try to clone all of the accounts at Anchor.toml from mainnet
/// if you want to clone only few, then fill out the array with the account indexes wrote at Anchor.toml

/// File running command: npx tsx tests/mock_mainnet_accounts.ts

const delay = (ms: number) => {
    return new Promise(resolve => setTimeout(resolve, ms));
}

const cloneMainnetAccounts = async (accounts: Array<number>) => {
    // parse Anchor.toml
    let anchorToml = fs.readFileSync(path.join(__dirname, "../Anchor.toml"), { encoding: "utf-8" });
    let parsed = toml.parse(anchorToml);

    let tomlAccounts = parsed["test"]["validator"]["account"];
    if (accounts.length > 0) {
        tomlAccounts = accounts.map((element) => tomlAccounts[element]);
    }

    // run cli command; copy mainnet accounts
    for (const account of tomlAccounts) {
        if (!account["filename"].includes("/tests/mocks/mainnet/")) continue;
        const command = `solana account -u m ${account["address"]} --output-file ${account["filename"]} --output json`;
        try {
            console.log(command);
            execSync(command);
        } catch (err) {
            console.error(err);
        }
        await delay(100);
    }
}

const wantToResetAccountIndexes = [];
cloneMainnetAccounts(wantToResetAccountIndexes)
    .then((accounts) => {
        process.exit(0);
    })
    .catch((err) => {
        console.error(err);
        process.exit(1);
    })
