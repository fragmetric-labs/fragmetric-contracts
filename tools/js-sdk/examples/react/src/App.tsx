import * as fragmetricSDK from '@fragmetric-labs/sdk';
import * as web3 from '@solana/web3.js';

import { ConnectionProvider, WalletProvider, useWallet, useConnection } from '@solana/wallet-adapter-react';
import { WalletAdapterNetwork } from '@solana/wallet-adapter-base';
import { UnsafeBurnerWalletAdapter } from '@solana/wallet-adapter-wallets';
import { WalletModalProvider, WalletDisconnectButton, WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import '@solana/wallet-adapter-react-ui/styles.css';
import {useCallback, useMemo, useState, useEffect} from 'react';

export default function App() {
    return (
        <ConnectionProvider endpoint={web3.clusterApiUrl(WalletAdapterNetwork.Devnet)}>
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
    const { publicKey: walletAddress, signTransaction, sendTransaction } = useWallet();
    const { connection } = useConnection();

    const fragSOLProgram = useMemo(() => {
        return new fragmetricSDK.RestakingProgram({
            cluster: 'devnet',
            connection, // use wallet adappter connection
            receiptTokenMint: fragmetricSDK.RestakingProgram.receiptTokenMint.fragSOL,
        });
    }, [connection]);

    // useEffect(() => {
    //     Promise.all([
    //         fragSOLProgram.state.fund(),
    //         fragSOLProgram.state.fund(),
    //         fragSOLProgram.state.fund(),
    //         fragSOLProgram.state.fund(),
    //         fragSOLProgram.state.addressLookupTables(),
    //         fragSOLProgram.state.addressLookupTables(),
    //         fragSOLProgram.state.addressLookupTables(),
    //     ]).then(console.log);
    // }, [fragSOLProgram]);

    const donateSOL1 = useCallback(async () => {
        try {
            setTransactionStatus('Sending donation...');
            const msg = await fragSOLProgram.operator.donateSOLToFund({ operator: walletAddress!, amount: new fragmetricSDK.BN(100), offsetReceivable: false });
            const { context: { slot: minContextSlot }, value: { blockhash, lastValidBlockHeight } } = await connection.getLatestBlockhashAndContext();
            msg.recentBlockhash = blockhash;
            const tx = new web3.VersionedTransaction(msg.compileToV0Message());
            const signature = await sendTransaction(tx, connection, { minContextSlot });
            console.log('tx sent using wallet', { tx, signature });
            setTransactionStatus(`Donation sent: ${signature}`);

            await connection.confirmTransaction({ signature, blockhash, lastValidBlockHeight }, 'confirmed');
            console.log('tx confirmed using wallet', signature);
            setTransactionStatus(`Donation confirmed: ${signature}`);
        } catch (err) {
            setTransactionStatus(`Donation failed: ${err}`);
            console.error(err);
        }
    }, [fragSOLProgram, walletAddress]);

    const donateSOL2 = useCallback(async () => {
        try {
            setTransactionStatus('Sending donation...');
            const msg = await fragSOLProgram.operator.donateSOLToFund({ operator: walletAddress!, amount: new fragmetricSDK.BN(100), offsetReceivable: false });
            const res = await msg.send({
                commitment: 'confirmed',
                onSign: async (tx, publicKey, _name) => {
                    if (publicKey.equals(walletAddress!)) {
                        return signTransaction!(tx);
                    }
                    return null;
                },
                onBeforeConfirm: async (tx, confirmStrategy, commitment) => {
                    console.log('tx sent using sdk method', { tx, confirmStrategy, commitment });
                    setTransactionStatus(`Donation sent: ${confirmStrategy.signature}`);
                },
            });
            console.log('tx confirmed using sdk method', res);
            setTransactionStatus(`Donation confirmed: ${res.signature}`);
        } catch (err) {
            setTransactionStatus(`Donation failed: ${err}`);
            console.error(err);
        }
    }, [fragSOLProgram, walletAddress]);

    return (
        <div style={{textAlign: 'center', marginTop: '50px'}}>
            <h1>Fragmetric SDK Example React</h1>
            {walletAddress ? (
                <div>
                    <center><WalletDisconnectButton/></center>
                    <p>Connected Wallet Address: {walletAddress.toString()}</p>
                    <div style={{marginBottom: 10}}>
                        <button onClick={donateSOL1}>Donate 100 SOL (1: using wallet)</button>
                    </div>
                    <div>
                        <button onClick={donateSOL2}>Donate 100 SOL (2: using sdk method)</button>
                    </div>
                </div>
            ) : (
                <center><WalletMultiButton/></center>
            )}
            {transactionStatus && <p>{transactionStatus}</p>}
        </div>
    )
}
