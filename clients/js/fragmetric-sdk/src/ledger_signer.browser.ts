import {ILedgerSignerConnector} from "./ledger_signer";

export const LedgerSigner: ILedgerSignerConnector = {
    connect() {
        throw "ledger signer is not supported in browser bundle";
    }
}
