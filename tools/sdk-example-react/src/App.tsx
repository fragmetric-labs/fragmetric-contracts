import { RestakingProgram, BN } from '@fragmetric-labs/sdk';
import * as web3 from '@solana/web3.js';

import { ConnectionProvider, WalletProvider, useWallet } from '@solana/wallet-adapter-react';
import { UnsafeBurnerWalletAdapter } from '@solana/wallet-adapter-wallets';
import { WalletModalProvider, WalletDisconnectButton, WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import '@solana/wallet-adapter-react-ui/styles.css';

import { useState } from 'react';

const fragSOL = new RestakingProgram({
    cluster: 'devnet',
    connection: undefined, // default RPC
    idl: undefined, // default IDL
    receiptTokenMint: RestakingProgram.receiptTokenMint.fragSOL,
});

export default function App() {
    return (
        <ConnectionProvider endpoint={fragSOL.connection.rpcEndpoint}>
            <WalletProvider wallets={[new UnsafeBurnerWalletAdapter()]} autoConnect>
                <WalletModalProvider>
                    <div>
                        <Main/>
                    </div>
                </WalletModalProvider>
            </WalletProvider>
        </ConnectionProvider>
    );
};

function Main() {
    const [transactionStatus, setTransactionStatus] = useState<string>('');
    const { publicKey: walletAddress } = useWallet();

    const donateSOL1 = async () => {
        try {
            setTransactionStatus('Sending donation...');
            const msg = await fragSOL.operator.donateSOLToFund({ operator: walletAddress!, amount: new BN(100), offsetReceivable: false });
            const { blockhash } = await fragSOL.connection.getLatestBlockhash();
            msg.recentBlockhash = blockhash;
            const tx = new web3.VersionedTransaction(msg.compileToV0Message());
            const res = await window.solana.signAndSendTransaction(tx);
            console.log('using wallet send', res);
            setTransactionStatus(`Donation sent: ${res.signature}`);
        } catch (err) {
            setTransactionStatus(`Donation failed: ${err}`);
            console.error(err);
        }
    };

    const donateSOL2 = async () => {
        try {
            setTransactionStatus('Sending donation...');
            const msg = await fragSOL.operator.donateSOLToFund({ operator: walletAddress!, amount: new BN(100), offsetReceivable: false });
            const res = await msg.send({
                signer: async (name, publicKey, tx) => {
                    if (publicKey.equals(walletAddress!)) {
                        const res = await window.solana.signTransaction(tx);
                        return {
                            signature: res.signatures[0],
                            publicKey,
                        };
                    }
                    throw `unhandled singer: ${name} (${publicKey})`;
                },
                sendOptions: {
                    skipPreflight: true,
                },
                onBeforeConfirm: async (confirmStrategy, commitment) => {
                    console.log(confirmStrategy, commitment);
                },
            });
            console.log('using builtin send', res);
            setTransactionStatus(`Donation sent: ${res.signature}`);
        } catch (err) {
            setTransactionStatus(`Donation failed: ${err}`);
            console.error(err);
        }
    };

    return (
        <div style={{textAlign: 'center', marginTop: '50px'}}>
            <h1>Fragmetric SDK Example</h1>
            {walletAddress ? (
                <div>
                    <p>Connected Wallet Address: {walletAddress.toString()}</p>
                    <div style={{marginBottom: 10}}>
                        <button onClick={donateSOL1}>Donate 100 SOL (method1: builtin send)</button>
                    </div>
                    <div>
                        <button onClick={donateSOL2}>Donate 100 SOL (method2: wallet send)</button>
                    </div>
                    <div>
                        <WalletDisconnectButton/>
                    </div>
                </div>
            ) : (
                <WalletMultiButton/>
            )}
            {transactionStatus && <p>{transactionStatus}</p>}
        </div>
    )
}
