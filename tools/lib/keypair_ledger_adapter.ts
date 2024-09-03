import * as web3 from "@solana/web3.js";
import {getLogger} from "./logger";

// ref: https://www.npmjs.com/package/@ledgerhq/hw-transport-node-hid-noevents
import { Subscription } from "@ledgerhq/hw-transport";
import LedgerTransport from "@ledgerhq/hw-transport-node-hid-noevents";

// ref: https://www.npmjs.com/package/@ledgerhq/hw-app-solana
import LedgerSolanaApp from "@ledgerhq/hw-app-solana";

const logger = getLogger('keypair/ledger');

export class KeypairLedgerAdapter {
    public static async create(): Promise<KeypairLedgerAdapter> {
        logger.notice(`initializing keypair ledger adapter:`);
        const transport = await new Promise<LedgerTransport>((resolve, reject) => {
            let found = false;
            let subscription: Subscription | null = null;
            let timeoutId = setTimeout(() => {
                subscription?.unsubscribe();
                reject(new Error('finding ledger timed out'));
            }, 5000);
            subscription = LedgerTransport.listen({
                next: event => {
                    found = true;
                    subscription?.unsubscribe();
                    clearTimeout(timeoutId);
                    LedgerTransport
                        .open(event.descriptor)
                        .then(resolve, reject);
                },
                error: err => {
                    clearTimeout(timeoutId);
                    reject(err);
                },
                complete: () => {
                    clearTimeout(timeoutId);
                    if (!found) {
                        reject(new Error('ledger not found'));
                    }
                },
            });
        });
        const adapter = new KeypairLedgerAdapter(new LedgerSolanaApp(transport));
        logger.debug(`finding ledger device...`);
        try {
            const config = await adapter.solanaApp.getAppConfiguration();
            logger.info(`connected to solana app:`, config);
        } catch (err) {
            logger.error(`failed to load solana app configuration: device locked or app not open`);
            throw err;
        }
        return adapter;
    }


    private constructor(
        private readonly solanaApp: LedgerSolanaApp,
    ) {
    }

    public static readonly defaultBIP32Path = `44'/501'/0'`;

    public async getPublicKey(bip32Path: string = KeypairLedgerAdapter.defaultBIP32Path): Promise<web3.PublicKey> {
        return this.solanaApp.getAddress(bip32Path).then(r => new web3.PublicKey(r.address));
    }
}