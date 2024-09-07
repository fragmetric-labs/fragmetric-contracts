import { getKeychain, KEYCHAIN_ENV } from './keychain';
import {askOnce} from "../lib/repl";

if (process.argv.length > 2) {
    syncKeypairs(process.argv[process.argv.length-1] as KEYCHAIN_ENV);
} else {
    askOnce<KEYCHAIN_ENV>(`[?] select target environment (local/devnet/mainnet): `).then(env => {
        syncKeypairs(env);
    });
}

function syncKeypairs(env: KEYCHAIN_ENV) {
    getKeychain(env)
        .catch(err => {
            console.error(err);
            process.exit(1);
        })
        .finally(() => {
            process.exit(0);
        });
}