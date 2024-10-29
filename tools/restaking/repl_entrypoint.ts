import {getKeychain, KEYCHAIN_ENV} from "./keychain";
import {askOnce, startREPL} from "../lib/repl";
import {RestakingPlayground} from "./playground";
import * as anchor from "@coral-xyz/anchor";
import {BN} from "@coral-xyz/anchor";
import * as web3 from "@solana/web3.js";

if (process.argv.length > 2) {
    run(process.argv[process.argv.length - 1] as KEYCHAIN_ENV);
} else {
    askOnce<KEYCHAIN_ENV>(`[?] select target environment (local/devnet/mainnet): `).then(env => {
        run(env);
    });
}

function run(env: KEYCHAIN_ENV) {
    RestakingPlayground.create(env)
        .then(restaking => {
            console.log(`[!] Type 'restaking.' and press TAB to start...`);
            startREPL({
                prompt: `${restaking.connection.rpcEndpoint} > `,
                context: {
                    restaking,
                    BN,
                },
            });
        })
        .catch(err => {
            console.error(err);
            process.exit(1);
        });
}