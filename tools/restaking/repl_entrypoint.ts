import {getKeychain, KEYCHAIN_ENV} from "./keychain";
import {askOnce, startREPL} from "../lib/repl";
import {RestakingPlayground} from "./playground";
import * as web3 from "@solana/web3.js";
import * as anchor from '@coral-xyz/anchor';
// @ts-ignore
import chalk from 'chalk';

if (process.argv.length > 2) {
    run(process.argv[process.argv.length - 1] as KEYCHAIN_ENV);
} else {
    askOnce<KEYCHAIN_ENV>(`[?] select target environment (local/devnet/mainnet): `).then(env => {
        run(env);
    });
}

function run(env: KEYCHAIN_ENV) {
    anchor.BN.prototype[Symbol.for("nodejs.util.inspect.custom")] = function() {
        return chalk.yellow(this.toString().replace(/\B(?=(\d{3})+(?!\d))/g, "_"));
    }
    web3.PublicKey.prototype[Symbol.for("nodejs.util.inspect.custom")] = anchor.web3.PublicKey.prototype[Symbol.for("nodejs.util.inspect.custom")] = function () {
        return chalk.blue(this.toString());
    }

    RestakingPlayground.create(env)
        .then(restaking => {
            const cluster = restaking.isMainnet ? chalk.bgRed.white('mainnet') : (restaking.isDevnet ? chalk.bgYellow.black('devnet') : chalk.bgWhite.black('local'));
            const endpoint = chalk.dim(`${restaking.connection.rpcEndpoint.length > 35 ? restaking.connection.rpcEndpoint.substring(0, 33) + '..' : restaking.connection.rpcEndpoint}`);

            console.log(`[!] Type 'restaking.' or 'r.' and press TAB to start...\n[!] Can use _ to refer the previous expression.`);
            startREPL({
                prompt: `${cluster} ${endpoint} > `,
                context: {
                    web3,
                    BN: anchor.BN,
                    restaking,
                    r: restaking,
                },
            });
        })
        .catch(err => {
            console.error(err);
            process.exit(1);
        });
}
