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
    const { publicKey: walletAddress, signTransaction, sendTransaction } = useWallet();
    const { connection } = useConnection();
    const [transactionStatus, setTransactionStatus] = useState<string>('');

    /** There are three ways to initialize restaking program client.

     First, populating a fund with a specific receipt token mint as like fragSOL, fragJTO and more.
    ```
     const fragSOL = new fragmetricSDK.RestakingClient({
         receiptTokenMint: fragmetricSDK.RestakingClient.receiptTokenMints.fragSOL, // or 'fragSOL' string, or web3.PublicKey
         cluster: 'devnet', // or 'mainnet', or 'local' for SDK development
         connection, // here, it can reuse wallet adapter connection
     });
    ```

     Second, populating a fund with a restaking vault address. This is for the use-case of the restaking protocols.
     Below `restakingVault` address is of the Jito Restaking Vault account for fragSOL on devnet.
     It will throw if there is no fund related to the given restaking vault address.
     ```
     const fragSOL = await fragmetricSDK.RestakingClient.createWithVault({
         restakingVault: new web3.PublicKey('BxhsigZDYjWTzXGgem9W3DsvJgFpEK5pM2RANP22bxBE'),
         cluster: 'devnet',
         connection,
     });
     ```

     Finally, it can populate clients for all the available liquid restaking tokens as like below.
    **/

    // funds state
    const [funds, setFunds] = useState<Array<{
        active: boolean,
        activeSupportedAssetIndex: number,
        client: fragmetricSDK.RestakingClient,
        supportedAssets: fragmetricSDK.RestakingFundSupportedAsset[],
        normalizedToken: fragmetricSDK.RestakingFundNormalizedToken | null,
        receiptToken: fragmetricSDK.RestakingFundReceiptToken,
    }>>([]);

    useEffect(() => {
        (async () => {
            const clients = await fragmetricSDK.RestakingClient.createAll({ cluster: 'devnet', connection });
            const funds = await Promise.all(
                // this kind of state loading is not mandatory, it can be done on demand, here, just for demonstration.
                clients.map(async (client, i) => {
                    const [supportedAssets, normalizedToken, receiptToken] = await Promise.all([
                        client.state.supportedAssets(), // can utilize this to display supported assets for users (SOL or tokens)
                        client.state.normalizedToken(), // can utilize this to get nSOL price
                        client.state.receiptToken(), // can utilize this to estimate minting fragSOL amount
                    ]);
                    return {
                        active: i == clients.length - 1,
                        activeSupportedAssetIndex: supportedAssets.findIndex(supportedAsset => supportedAsset.depositable),
                        client,
                        supportedAssets,
                        normalizedToken,
                        receiptToken,
                    };
                })
            );
            setFunds(funds);
        })();
    }, [connection]);

    const fund = useMemo(() => {
        return funds.find(f => f.active) ?? funds[0] ?? null;
    }, [funds]);

    const deposit1 = useCallback(async () => {
        try {
            console.log(fund.activeSupportedAssetIndex, fund.supportedAssets[fund.activeSupportedAssetIndex]?.mint);
            setTransactionStatus('sending tx...');
            // it creates an instance of extended web3.TransactionMessage class.
            const msg = await fund.client.user.deposit({
                    user: walletAddress!,
                    supportedTokenMint: fund.supportedAssets[fund.activeSupportedAssetIndex]?.mint ?? null,
                    amount: new fragmetricSDK.BN(100),
                });

            // here it just uses general web3.js and wallet adapter hooks to send tx
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
    }, [fund, walletAddress]);

    const deposit2 = useCallback(async () => {
        try {
            setTransactionStatus('sending tx...');
            const res = await fund.client.user.deposit({
                    user: walletAddress!,
                    supportedTokenMint: fund.supportedAssets[fund.activeSupportedAssetIndex]?.mint ?? null,
                    amount: new fragmetricSDK.BN(100),
                })
                // here, it simply uses SDK builtin `send` method and wallet adapter's `signTransaction`
                .then(msg => {
                    return msg.send({
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
                    });
                });
            if (res.error) {
                throw res.error;
            }
            console.log('tx confirmed using sdk method', res);
            setTransactionStatus(`tx confirmed: ${res.signature}`);
        } catch (err) {
            setTransactionStatus(`tx confirmation failed: ${err}`);
            console.error(err);
        }
    }, [fund, walletAddress]);

    return (
        <div style={{textAlign: 'center', marginTop: '50px'}}>
            <h1>Fragmetric SDK Example React</h1>
            {walletAddress ? (
                <div>
                    <div>
                        <p>Connected Wallet Address: {walletAddress.toString()}</p>
                        <center><WalletDisconnectButton/></center>
                    </div>
                    <div>
                        <h3>Select a Token to Mint</h3>
                        <select
                            style={{marginBottom: 10, width: 350}}
                            size={5}
                            value={funds.findIndex(f => f.active).toString()}
                            onChange={(e) => setFunds(funds => funds.map((fund, i) => {
                                fund.active = i.toString() == e.target.value;
                                return fund;
                            }))}
                        >
                            {funds.map((fund, i) => {
                                return (
                                    <option
                                        key={i}
                                        value={i.toString()}
                                    >
                                        {fund.receiptToken.mint.toString()}
                                    </option>
                                )
                            })}
                        </select>
                    </div>
                    {fund ? (
                        <div>
                            <h3>Select an Asset to Deposit</h3>
                            <select
                                style={{marginBottom: 10, width: 350}}
                                size={5}
                                value={fund.activeSupportedAssetIndex.toString()}
                                onChange={(e) => setFunds(funds => funds.map((fund, i) => {
                                    if (fund.active) {
                                        fund.activeSupportedAssetIndex = parseInt(e.target.value);
                                    }
                                    return fund;
                                }))}
                            >
                                {(fund.supportedAssets ?? [])
                                    .map((supportedAsset, i) => {
                                        return (
                                            <option
                                                key={i}
                                                disabled={!supportedAsset.depositable}
                                                value={i.toString()}
                                            >
                                                {`${supportedAsset.mint?.toString() ?? 'native SOL'}${supportedAsset.depositable ? '' : ' (disabled)'}`}
                                            </option>
                                        );
                                    })
                                }
                            </select>
                            <div style={{marginBottom: 10}}>
                                <button onClick={deposit1}>Deposit 100 lamports/token (1: using wallet)</button>
                            </div>
                            <div style={{marginBottom: 10}}>
                                <button onClick={deposit2}>Deposit 100 lamports/token (2: using sdk method)</button>
                            </div>
                            <h3>Transaction Status</h3>
                            <div>
                                <p>{transactionStatus || '...'}</p>
                            </div>
                            <h3>Fund Information</h3>
                            <pre style={{textAlign: "left", margin: "10px 50px", padding: "10px", background: "#eee"}}>
                                {JSON.stringify({
                                    receiptToken: fund.receiptToken,
                                    supportedAssets: fund.supportedAssets,
                                    normalizedToken: fund.normalizedToken,
                                }, null, 2)}
                            </pre>
                        </div>
                    ) : null}
                </div>
            ) : (
                <center><WalletMultiButton/></center>
            )}
        </div>
    )
}
