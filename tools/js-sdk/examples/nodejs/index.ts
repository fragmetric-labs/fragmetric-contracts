import * as web3 from '@solana/web3.js';
import { RestakingProgram, LedgerSigner } from '@fragmetric-labs/sdk';


(async function example1() {
    const wallet = web3.Keypair.fromSecretKey(Uint8Array.from([18,99,108,102,2,206,6,7,168,174,190,163,59,172,204,141,105,14,181,146,108,161,134,128,169,57,152,205,238,237,220,216,150,75,239,172,33,80,166,64,55,49,182,185,30,49,104,33,14,163,68,64,59,209,64,244,34,15,83,110,17,139,78,4]));

    const ledger = await LedgerSigner.connect({
        handler: {
            onBeforeConnect: (bip32Path) => console.log(`[ledger] connecting: ${bip32Path}`),
            onConnect: (publicKey, firmwareVersion) => console.log(`[ledger] connected: ${publicKey} (app version: ${firmwareVersion})`),
            onError: console.error,
        },
    });

    const program = new RestakingProgram({
        cluster: 'devnet',
        connection: undefined, // default RPC
        idl: undefined, // default IDL
        receiptTokenMint: RestakingProgram.receiptTokenMint.fragSOL,
        transactionHandler: {
            onSign: async (tx, publicKey, name) => {
                if (publicKey == wallet.publicKey) {
                    return wallet;
                } else if (publicKey == ledger.publicKey) {
                    console.log(`[ledger] signing: ${publicKey}`);
                    return ledger.signTransaction(tx);
                }
                return null;
            },
            onBeforeSend: async (txMessage) =>{
                console.log(`[sending] description: ${txMessage.descriptions.join(', ')}`);
            },
            onBeforeConfirm: async (tx, confirmStrategy, commitment) => {
                console.log(`[confirming] commitment: ${commitment}`, confirmStrategy);
            },
            onConfirm: async (result) => {
                console.log(`[confirmed] signature: ${result.signature}`, result.events);
            },
        },
    });

    const result = await program
        .operator
        .updateFundPrices({ operator: ledger.publicKey })
        .then(txMessage => txMessage.send());
    console.log(result);
})();
