import {getKeychain, KEYCHAIN_ENV} from './keychain';
import {askOnce} from "../lib/repl";

if (process.argv.length > 2) {
    run(process.argv[process.argv.length - 1] as KEYCHAIN_ENV);
} else {
    askOnce<KEYCHAIN_ENV>(`[?] select target environment (local/devnet/mainnet): `).then(env => {
        run(env);
    });
}

function run(env: KEYCHAIN_ENV) {
    getKeychain(env)
        .catch(err => {
            console.error(err);
            process.exit(1);
        })
        .finally(() => {
            process.exit(0);
        });
}