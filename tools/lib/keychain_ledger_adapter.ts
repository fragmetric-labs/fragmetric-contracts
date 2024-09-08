import * as web3 from "@solana/web3.js";
import {getLogger} from "./logger";

// ref: https://github.com/LedgerHQ/ledger-live/tree/develop/libs/ledgerjs/packages/hw-transport-node-hid-singleton
import { Subscription } from "@ledgerhq/hw-transport";
import Transport from "@ledgerhq/hw-transport-node-hid-singleton";

// ref: https://github.com/LedgerHQ/ledger-live/tree/develop/libs/ledgerjs/packages/hw-app-solana
import SolanaApp from "@ledgerhq/hw-app-solana";

const {logger, LOG_PAD_LARGE, LOG_PAD_SMALL} = getLogger('keychain/ledger');

export class KeychainLedgerAdapter {
    public static async create(): Promise<KeychainLedgerAdapter> {
        logger.notice(`initializing keypair ledger adapter:`);
        const transport = await new Promise<Transport>((resolve, reject) => {
            let found = false;
            let subscription: Subscription | null = null;
            let timeoutId = setTimeout(() => {
                subscription?.unsubscribe();
                reject(new Error('finding ledger timed out'));
            }, 3000);
            subscription = Transport.listen({
                next: event => {
                    found = true;
                    subscription?.unsubscribe();
                    clearTimeout(timeoutId);
                    Transport
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
        const adapter = new KeychainLedgerAdapter(new SolanaApp(transport));
        logger.debug(`finding ledger device...`);
        try {
            const config = await adapter.solanaApp.getAppConfiguration();
            logger.info(`connected to solana app:`, `firmware v${config.version}`);
            if (!config.blindSigningEnabled) {
                logger.fatal(`blind signing is disabled in ledger settings`);
            }
        } catch (err) {
            logger.error(`failed to load solana app configuration: device locked or app not open`);
            throw err;
        }
        return adapter;
    }


    private constructor(
        readonly solanaApp: SolanaApp,
    ) {
    }

    public static readonly defaultBIP32Path = `44'/501'/0'`;

    public async getPublicKey(bip32Path: string): Promise<web3.PublicKey> {
        const r = await this.solanaApp.getAddress(bip32Path);
        return new web3.PublicKey(r.address);
    }

    public async getSignature(bip32Path: string, txBuffer: Buffer): Promise<Buffer> {
        let account = `${(await this.getPublicKey(bip32Path)).toString()} (${bip32Path})`;
        logger.notice(`singing with ledger`.padEnd(LOG_PAD_LARGE), account);
        try {
            const r = await this.solanaApp.signTransaction(bip32Path, txBuffer);
            return r.signature;
        } catch (e) {
            if (e.statusCode == 27265) {
                logger.error(`invalid signer for the transaction`.padEnd(LOG_PAD_LARGE), account);
            }
        }
    }
}
