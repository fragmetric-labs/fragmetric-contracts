import { getKeychain } from './keychain';

getKeychain('local')
    .catch(err => {
        console.error(err);
        process.exit(1);
    })
    .finally(() => {
        process.exit(0);
    });