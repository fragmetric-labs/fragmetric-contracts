import * as web3 from "@solana/web3.js";
// ref: https://github.com/LedgerHQ/ledger-live/tree/develop/libs/ledgerjs/packages/hw-transport-node-hid-singleton
import { Subscription } from "@ledgerhq/hw-transport";
import Transport from "@ledgerhq/hw-transport-node-hid-singleton";
// ref: https://github.com/LedgerHQ/ledger-live/tree/develop/libs/ledgerjs/packages/hw-app-solana
import SolanaApp from "@ledgerhq/hw-app-solana";

import {
    ILedgerSigner,
    ILedgerSignerConnector,
} from "./ledger_signer";

export const LedgerSigner: ILedgerSignerConnector = {
    connect: async ({handler, bip32Path = `44'/501'/0'`, retryDelaySeconds = 5} = {}) => {
        try {
            handler?.onBeforeConnect(bip32Path);

            const transport = await new Promise<Transport>((resolve, reject) => {
                let found = false;
                let subscription: Subscription | null = null;
                let timeoutId = setTimeout(() => {
                    subscription?.unsubscribe();
                    reject(new Error('ledger connection timed out'));
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

            const solanaApp = new SolanaApp(transport);
            const publicKey = await solanaApp.getAddress(bip32Path)
                .then(res => new web3.PublicKey(res.address))
                .catch(err => {
                    throw new Error(`failed to get public key of the ledger: ${err?.toString()}`);
                });
            const solanaAppConfig = await solanaApp.getAppConfiguration()
                .catch(err => {
                    throw new Error(`failed to get solana app configuration: device locked or app not open ... ${err?.toString()}`);
                });

            handler?.onConnect(publicKey, solanaAppConfig.version);

            if (!solanaAppConfig.blindSigningEnabled) {
                throw new Error(`blind signing is disabled in ledger settings`);
            }

            return new LedgerSignerImpl(solanaApp, publicKey, bip32Path);
        } catch (err) {
            if (handler?.onError(err as Error) === true) {
                await new Promise(resolve => setTimeout(resolve, Math.max(retryDelaySeconds, 1) * 1000));
                return LedgerSigner.connect({ handler, bip32Path, retryDelaySeconds });
            }
            throw err;
        }
    }
}

class LedgerSignerImpl implements ILedgerSigner {
    constructor(
        solanaApp: SolanaApp,
        publicKey: web3.PublicKey,
        bip32Path: string,
    ) {
        this.solanaApp = solanaApp;
        this.publicKey = publicKey;
        this.bip32Path = bip32Path;
    }

    private readonly solanaApp: SolanaApp;
    public readonly publicKey: web3.PublicKey;
    public readonly bip32Path: string;

    public async signTransaction(tx: web3.Transaction | web3.VersionedTransaction): Promise<Buffer> {
        const buffer = 'version' in tx ? Buffer.from((tx as web3.VersionedTransaction).message.serialize()) : (tx as web3.Transaction).serializeMessage();
        const res = await this.solanaApp.signTransaction(this.bip32Path, buffer);
        return res.signature;
    }
}
