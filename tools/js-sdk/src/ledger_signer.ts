import * as web3 from "@solana/web3.js";
import {ProgramTransactionSignature} from "./program_transaction";

export type LedgerSignerConnectionHandler = {
    onBeforeConnect: (bip32Path: string) => void;
    onConnect: (publicKey: web3.PublicKey, solanaAppVersion: string) => void;
    // return true to retry
    onError: (err: Error) => boolean;
};

export interface ILedgerSigner {
    readonly publicKey: web3.PublicKey;
    readonly bip32Path: string;
    signTransaction(tx: web3.VersionedTransaction | web3.Transaction): Promise<ProgramTransactionSignature> | ProgramTransactionSignature;
}

export interface ILedgerSignerConnector {
    connect(params?: {
        handler?: LedgerSignerConnectionHandler;
        bip32Path?: string;
        retryDelaySeconds?: number;
    }): Promise<ILedgerSigner>;
}

// will be aliased to './ledger_signer_impl.browser' in browser bundle
export { LedgerSigner } from './ledger_signer_impl';
