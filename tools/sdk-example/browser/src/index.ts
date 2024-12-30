import { RestakingProgram } from '@fragmetric-labs/sdk';
// export * as anchor from '@coral-xyz/anchor';
// export * as web3 from '@solana/web3.js';

document.addEventListener('DOMContentLoaded', async () => {
    const solana = (window as any).solana;
    const fragSOL = new RestakingProgram({
        cluster: 'devnet',
        connection: undefined, // default RPC
        idl: undefined, // default IDL
        receiptTokenMint: RestakingProgram.receiptTokenMint.fragSOL,
    });

    // const tx = await fragSOL.operator.donateSOLToFund({amount: 100, offsetReceivable: false});

    function render() {
        if (solana.isConnected) {
            document.body.innerHTML = `
                <button onClick="solana.connectWallet()">Connect Wallet</button>
            `;
        }
    }
    render();
});
