import fs from "fs";
import path from "path";
import toml from "toml";
import {execSync} from "child_process";

/// HOW TO USE
/// if `targetAccountAddresses` array at line 42 is empty, it'll try to clone all of the accounts at Anchor.toml from mainnet
/// if you want to clone only few, then fill out the array with the account indexes wrote at Anchor.toml

/// File running command: npx tsx tests/mock_mainnet_accounts.ts

const delay = (ms: number) => {
    return new Promise(resolve => setTimeout(resolve, ms));
}

const cloneMainnetAccounts = async (addresses: Array<string>) => {
    // parse Anchor.toml
    let anchorToml = fs.readFileSync(path.join(__dirname, "../Anchor.toml"), {encoding: "utf-8"});
    let parsed = toml.parse(anchorToml);

    let tomlAccounts = parsed["test"]["validator"]["account"].concat(parsed["test"]["genesis"]);
    if (addresses.length > 0) {
        let addressesSet = new Set(addresses);
        tomlAccounts = tomlAccounts.filter((elem) => {
            return addressesSet.has(elem.address);
        });
    }
    /* tomlAccounts is an array like below:
        [
            {
                address: '5eosrve6LktMZgVNszYzebgmmC7BjLK8NoWyRQtcmGTF',
                filename: './tests/mocks/mainnet/jito_program_fee_wallet_vrt_account.json'
            },
            {
                address: 'J6AS6PFJip13cStdiuvRrLz2hDZiZvxdLhmsopN7YTDM',
                filename: './tests/mocks/local/fragsol_jito_vrt_mint.json'
            },
            {
                address: 'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
                program: './tools/mocks/mainnet/metaplex.so'
            },
            ...,
        ]
     */

    // run cli command; copy mainnet accounts
    for (const account of tomlAccounts) {
        const targetPath = account["filename"] ?? account["program"] ?? "";
        const isProgram = !!account["program"];
        if (!targetPath.includes("/tests/mocks/mainnet/")) continue;

        const command = isProgram ? `solana program dump -u m ${account["address"]} ${account["program"]}` : `solana account -u m ${account["address"]} --output-file ${account["filename"]} --output json`;
        try {
            console.log(command);
            execSync(command);
        } catch (err) {
            console.error(err);
        }
        await delay(100);
    }
}

const targetAccountAddresses = [];
cloneMainnetAccounts(targetAccountAddresses)
    .then((accounts) => {
        process.exit(0);
    })
    .catch((err) => {
        console.error(err);
        process.exit(1);
    })
