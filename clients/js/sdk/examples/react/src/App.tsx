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
    const [selectTokenMint, setSelectTokenMint] = useState<string>('SOL');
    const [fragSOLState, setFragSOLState] = useState<{
        loading: boolean;
        data: {
            supportedAssets: fragmetricSDK.RestakingFundSupportedAsset[],
            normalizedToken: fragmetricSDK.RestakingFundNormalizedToken | null,
            receiptToken: fragmetricSDK.RestakingFundReceiptToken,
        } | null;
    }>({ loading: true, data: null });

    const fragSOL = useMemo(() => {
        return new fragmetricSDK.RestakingProgram({
            cluster: 'devnet',
            connection, // use wallet adapter connection
            fundReceiptTokenMint: 'fragSOL',
        });
    }, [connection]);

    useEffect(() => {
        Promise.all([
            fragSOL.state.supportedAssets(),
            fragSOL.state.normalizedToken(),
            fragSOL.state.receiptToken(),
        ]).then(([supportedAssets, normalizedToken, receiptToken]) => {
            setFragSOLState({
                loading: false,
                data: { supportedAssets, normalizedToken, receiptToken },
            });
        });
    }, [fragSOL]);

    // example1: just using web3.js and wallet adapter hooks
    const deposit1 = useCallback(async () => {
        try {
            setTransactionStatus('sending tx...');
            const msg = await fragSOL.operator
                .donateSOLToFund({ operator: walletAddress!, amount: new fragmetricSDK.BN(100), offsetReceivable: false });

            const { context: { slot: minContextSlot }, value: { blockhash, lastValidBlockHeight } } = await connection.getLatestBlockhashAndContext();
            msg.recentBlockhash = blockhash;
            const tx = new web3.VersionedTransaction(msg.compileToV0Message());
            const signature = await sendTransaction(tx, connection, { minContextSlot });
            console.log('tx sent using wallet', { tx, signature });
            setTransactionStatus(`tx sent: ${signature}`);

            const res = await connection.confirmTransaction({ signature, blockhash, lastValidBlockHeight }, 'confirmed');
            if (res.value.err) {
                throw res.value.err;
            }
            console.log('tx confirmed using wallet', signature);
            setTransactionStatus(`tx confirmed: ${signature}`);
        } catch (err) {
            setTransactionStatus(`tx confirmation failed: ${err}`);
            console.error(err);
        }
    }, [fragSOL, walletAddress, selectTokenMint]);

    // example2: simply using SDK builtin `send` method and wallet adapter's `signTransaction`
    const deposit2 = useCallback(async () => {
        try {
            setTransactionStatus('sending tx...');
            const res = await fragSOL.operator
                .donateSOLToFund({ operator: walletAddress!, amount: new fragmetricSDK.BN(100), offsetReceivable: false })
                .then(msg => msg.send({
                    commitment: 'confirmed',
                    onSign: async (tx, publicKey, name) => {
                        if (publicKey.equals(walletAddress!)) {
                            return signTransaction!(tx);
                        }
                        return null;
                    },
                    onBeforeConfirm: async (tx, confirmStrategy, commitment) => {
                        console.log('tx sent using sdk method', { tx, confirmStrategy, commitment });
                        setTransactionStatus(`tx sent: ${confirmStrategy.signature}`);
                    },
                }));
            if (res.error) {
                throw res.error;
            }
            console.log('tx confirmed using sdk method', res);
            setTransactionStatus(`tx confirmed: ${res.signature}`);
        } catch (err) {
            setTransactionStatus(`tx confirmation failed: ${err}`);
            console.error(err);
        }
    }, [fragSOL, walletAddress, selectTokenMint]);

    return (
        <div style={{textAlign: 'center', marginTop: '50px'}}>
            <h1>Fragmetric SDK Example React</h1>
            {walletAddress ? (
                <div>
                    <center><WalletDisconnectButton/></center>
                    <p>Connected Wallet Address: {walletAddress.toString()}</p>
                    <p>Select an Asset to Deposit</p>
                    <select style={{marginBottom: 10, width: 350}} size={5}>
                        {(fragSOLState.data?.supportedAssets ?? []).filter(a => a.depositable).map((a) => {
                            const mint = a.isNativeSOL ? 'SOL' : a.mint?.toString();
                           return (<option selected={selectTokenMint == mint}>
                               {mint}
                           </option>)
                        })}
                    </select>
                    <div style={{marginBottom: 10}}>
                        <button onClick={deposit1}>Deposit 100 lamports/token (1: using wallet)</button>
                    </div>
                    <div>
                        <button onClick={deposit2}>Deposit 100 lamports/token (2: using sdk method)</button>
                    </div>
                </div>
            ) : (
                <center><WalletMultiButton/></center>
            )}
            {transactionStatus && <p>{transactionStatus}</p>}
            <pre style={{textAlign: "left", margin: "10px 50px", padding: "10px", background: "#eee"}}>
                {JSON.stringify(fragSOLState, null, 2)}
            </pre>
        </div>
    )
}
