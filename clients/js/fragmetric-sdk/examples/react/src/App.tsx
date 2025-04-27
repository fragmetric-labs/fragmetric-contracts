import { RestakingProgram, createDefaultTransactionExecutionHooks } from '@fragmetric-labs/sdk';
import * as web3 from '@solana/web3.js';

import { ConnectionProvider, WalletProvider, useWallet } from '@solana/wallet-adapter-react';
import { WalletAdapterNetwork } from '@solana/wallet-adapter-base';
import { UnsafeBurnerWalletAdapter } from '@solana/wallet-adapter-wallets';
import { WalletModalProvider, WalletDisconnectButton, WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import '@solana/wallet-adapter-react-ui/styles.css';
import { useCallback, useMemo, useState, useEffect } from 'react';
import { Base64EncodedWireTransaction } from '@solana/kit';

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
    const { publicKey: walletAddress, signTransaction } = useWallet();
    const [loading, setLoading] = useState(true);
    const restaking = useMemo(() => RestakingProgram.devnet(), []);
    useEffect(() => {
        restaking.resolve()
          .then(() => setLoading(false));
    }, [restaking]);
    const deposit = useCallback(async () => {
      const tx = await restaking
        .fragSOL
        .user(walletAddress.toBase58())
        .deposit
        .serializeToBase64({ assetAmount: 10n }, null, true);
      // TODO: use WalletAdapterSigner... instead of using web3 and raw rpc method
      try {
        const versionedTx = web3.VersionedTransaction.deserialize(Buffer.from(tx, 'base64'));
        const signedTx = await signTransaction(versionedTx);
        console.log(await restaking.runtime.rpc.sendTransaction(Buffer.from(signedTx.serialize()).toString('base64') as Base64EncodedWireTransaction, { encoding: 'base64' }).send());
      } catch (e) {
        console.error(e);
      }
    }, [restaking, walletAddress, signTransaction]);

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
                      <center><button onClick={deposit} disabled={loading}>Deposit 10 lamports</button></center>
                        <pre style={{textAlign:'left'}}>{restaking.toContextTreeString({multiline: true, maxLineWidth: 150})}</pre>
                    </div>
                    {/*<div>*/}
                    {/*    <h3>Select a Token to Mint</h3>*/}
                    {/*    <select*/}
                    {/*        style={{marginBottom: 10, width: 350}}*/}
                    {/*        size={5}*/}
                    {/*        value={funds.findIndex(f => f.active).toString()}*/}
                    {/*        onChange={(e) => setFunds(funds => funds.map((fund, i) => {*/}
                    {/*            fund.active = i.toString() == e.target.value;*/}
                    {/*            return fund;*/}
                    {/*        }))}*/}
                    {/*    >*/}
                    {/*        {funds.map((fund, i) => {*/}
                    {/*            return (*/}
                    {/*                <option*/}
                    {/*                    key={i}*/}
                    {/*                    value={i.toString()}*/}
                    {/*                >*/}
                    {/*                    {fund.receiptToken.mint.toString()}*/}
                    {/*                </option>*/}
                    {/*            )*/}
                    {/*        })}*/}
                    {/*    </select>*/}
                    {/*</div>*/}
                    {/*{fund ? (*/}
                    {/*    <div>*/}
                    {/*        <h3>Select an Asset to Deposit</h3>*/}
                    {/*        <select*/}
                    {/*            style={{marginBottom: 10, width: 350}}*/}
                    {/*            size={5}*/}
                    {/*            value={fund.activeSupportedAssetIndex.toString()}*/}
                    {/*            onChange={(e) => setFunds(funds => funds.map((fund, i) => {*/}
                    {/*                if (fund.active) {*/}
                    {/*                    fund.activeSupportedAssetIndex = parseInt(e.target.value);*/}
                    {/*                }*/}
                    {/*                return fund;*/}
                    {/*            }))}*/}
                    {/*        >*/}
                    {/*            {(fund.supportedAssets ?? [])*/}
                    {/*                .map((supportedAsset, i) => {*/}
                    {/*                    return (*/}
                    {/*                        <option*/}
                    {/*                            key={i}*/}
                    {/*                            disabled={!supportedAsset.depositable}*/}
                    {/*                            value={i.toString()}*/}
                    {/*                        >*/}
                    {/*                            {`${supportedAsset.mint?.toString() ?? 'native SOL'}${supportedAsset.depositable ? '' : ' (disabled)'}`}*/}
                    {/*                        </option>*/}
                    {/*                    );*/}
                    {/*                })*/}
                    {/*            }*/}
                    {/*        </select>*/}
                    {/*        <div style={{marginBottom: 10}}>*/}
                    {/*            <button onClick={deposit1}>Deposit 100 lamports/token (1: using wallet)</button>*/}
                    {/*        </div>*/}
                    {/*        <div style={{marginBottom: 10}}>*/}
                    {/*            <button onClick={deposit2}>Deposit 100 lamports/token (2: using sdk method)</button>*/}
                    {/*        </div>*/}
                    {/*        <h3>Transaction Status</h3>*/}
                    {/*        <div>*/}
                    {/*            <p>{transactionStatus || '...'}</p>*/}
                    {/*        </div>*/}
                    {/*        <h3>Fund Information</h3>*/}
                    {/*        <pre style={{textAlign: "left", margin: "10px 50px", padding: "10px", background: "#eee"}}>*/}
                    {/*            {JSON.stringify({*/}
                    {/*                receiptToken: fund.receiptToken,*/}
                    {/*                supportedAssets: fund.supportedAssets,*/}
                    {/*                normalizedToken: fund.normalizedToken,*/}
                    {/*            }, null, 2)}*/}
                    {/*        </pre>*/}
                    {/*    </div>*/}
                    {/*) : null}*/}
                </div>
            ) : (
                <center><WalletMultiButton/></center>
            )}
        </div>
    )
}
