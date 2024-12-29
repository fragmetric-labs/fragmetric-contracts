import * as web3 from '@solana/web3.js';
import BN from "bn.js";

import { RestakingFund, RestakingProgram } from '@fragmetric-labs/sdk/src';

const wallet = web3.Keypair.fromSecretKey(Uint8Array.from([18,99,108,102,2,206,6,7,168,174,190,163,59,172,204,141,105,14,181,146,108,161,134,128,169,57,152,205,238,237,220,216,150,75,239,172,33,80,166,64,55,49,182,185,30,49,104,33,14,163,68,64,59,209,64,244,34,15,83,110,17,139,78,4]));

async function example() {
    const program = new RestakingProgram({
        cluster: 'devnet',
        connection: undefined, // default RPC
        idl: undefined, // default IDL
        receiptTokenMint: RestakingProgram.receiptTokenMint.fragSOL,
    });

    const result = await program
        .operator
        .donateSOLToFund({ operator: wallet, amount: new BN(1000), offsetReceivable: false })
        .then(msg => msg.signAndSend());

    console.log(result);
}

example();
