import * as web3 from '@solana/web3.js';
import { RestakingClient, LedgerSigner, BN } from '@fragmetric-labs/sdk';


(async function example1() {
    const walletKeyPair = web3.Keypair.fromSecretKey(Uint8Array.from([18,99,108,102,2,206,6,7,168,174,190,163,59,172,204,141,105,14,181,146,108,161,134,128,169,57,152,205,238,237,220,216,150,75,239,172,33,80,166,64,55,49,182,185,30,49,104,33,14,163,68,64,59,209,64,244,34,15,83,110,17,139,78,4]));
    const ledgerPublicKey = new web3.PublicKey("79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84");

    const fragSOL = new RestakingClient({
        receiptTokenMint: 'fragSOL',
        cluster: 'devnet',
        connection: undefined, // default RPC
        transactionHandler: {
            onBeforeSign: async (txMessage) => {
                console.log(`[signing] description:`, ...txMessage.descriptions ?? []);
            },
            onSign: async (tx, publicKey, name) => {
                if (publicKey.equals(walletKeyPair.publicKey)) {

                    console.log(`[signed] by keypair:`, publicKey);
                    return walletKeyPair;
                } else if (publicKey.equals(ledgerPublicKey)) {
                    const ledger = await LedgerSigner.connect({
                        handler: {
                            onBeforeConnect: (bip32Path) => console.log(`[ledger] connecting: ${bip32Path}`),
                            onConnect: (publicKey, solanaAppVersion) => console.log(`[ledger] connected: ${publicKey} (solana app version: ${solanaAppVersion})`),
                            onError: (err) => {
                                console.error(err);
                                return true; // always retry
                            },
                        },
                    });

                    console.log(`[ledger] signing:`, publicKey);
                    const res = await ledger.signTransaction(tx);

                    console.log(`[signed] by ledger:`, publicKey);
                    return res;
                }
                return null;
            },
            onBeforeConfirm: async (tx, confirmStrategy, commitment) => {
                console.log(`[confirming] commitment: ${commitment}`, confirmStrategy);
            },
            onConfirm: async (result) => {
                console.log(`[confirmed] signature: ${result.signature}`, result.events);
            },
        },
    });

    const res1 = await fragSOL
        .user
        .deposit({
            user: walletKeyPair.publicKey,
            supportedTokenMint: null,
            amount: new BN(100),
        })
        .then(txMessage => txMessage.send());
    console.log(`[result]`, res1);

    const res2 = await fragSOL
        .operator
        .updateFundPrices({ operator: walletKeyPair.publicKey })
        .then(txMessage => txMessage.send());
    console.log(`[result]`, res2);

    const res3 = await fragSOL
        .operator
        .updateNormalizedTokenPoolPrices({ operator: ledgerPublicKey })
        .then(txMessage => txMessage.send());
    console.log(`[result]`, res3);
})();
