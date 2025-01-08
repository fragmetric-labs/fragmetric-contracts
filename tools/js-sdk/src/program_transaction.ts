import * as web3 from '@solana/web3.js';
import * as anchor from '@coral-xyz/anchor';
import { Buffer } from 'buffer';

if (typeof globalThis !== 'undefined') { // for browser bundle
    globalThis.Buffer = Buffer;
}

import { Program, ProgramEvent } from './program';

export type ProgramTransactionOnBeforeSend<IDL extends anchor.Idl> = (txMessage: ProgramTransactionMessage<IDL, any, keyof ProgramEvent<IDL>>, program: Program<IDL>) => Promise<void>;
export type ProgramTransactionSignature = web3.SignaturePubkeyPair | web3.Signer | Buffer | Uint8Array | web3.VersionedTransaction | null;
export type ProgramTransactionOnSign<SIGNER extends string> = (tx: web3.VersionedTransaction, publicKey: web3.PublicKey, name: 'payer' | SIGNER) => Promise<ProgramTransactionSignature> | ProgramTransactionSignature;
export type ProgramTransactionOnBeforeConfirm<IDL extends anchor.Idl> = (tx: web3.VersionedTransaction, confirmStrategy: web3.TransactionConfirmationStrategy, commitment: web3.Finality, program: Program<IDL>) => Promise<void>;
export type ProgramTransactionOnConfirm<IDL extends anchor.Idl> = (result: Awaited<ReturnType<typeof ProgramTransactionMessage.prototype.send>>, program: Program<IDL>) => Promise<void>;
export type ProgramTransactionHandler<IDL extends anchor.Idl> = {
    onSign: ProgramTransactionOnSign<'payer'|string>,
    onBeforeSend: ProgramTransactionOnBeforeSend<IDL>,
    onBeforeConfirm: ProgramTransactionOnBeforeConfirm<IDL>,
    onConfirm: ProgramTransactionOnConfirm<IDL>,
};

export class ProgramTransactionMessage<IDL extends anchor.Idl, SIGNER extends string, EVENT extends keyof ProgramEvent<IDL>> extends web3.TransactionMessage {
    public readonly descriptions: any[] | null;
    public readonly expectedEvents: EVENT[];
    public readonly signers: { [k in 'payer' | SIGNER]: web3.PublicKey | web3.Signer };
    public readonly addressLookupTables: web3.AddressLookupTableAccount[] = [];
    private readonly program: Program<IDL>;

    constructor({ descriptions = [], instructions, events = [], signers, recentBlockhash = null, addressLookupTables = [], program }: {
        descriptions?: any[] | null,
        events?: EVENT[],
        instructions: web3.TransactionInstruction[],
        signers: {[k in 'payer' | SIGNER]: web3.PublicKey | web3.Signer},
        recentBlockhash?: web3.Blockhash | null,
        addressLookupTables?: web3.AddressLookupTableAccount[],
        program: Program<IDL>,
    }) {
        super({
            payerKey: (signers.payer as web3.Signer).secretKey ? (signers.payer as web3.Signer).publicKey : signers.payer as web3.PublicKey,
            recentBlockhash: recentBlockhash ?? '',
            instructions,
        });
        this.descriptions = descriptions;
        this.expectedEvents = events;
        this.signers = signers;
        this.addressLookupTables = addressLookupTables;
        this.program = program;
    }

    public compileToV0Message(addressLookupTableAccounts?: web3.AddressLookupTableAccount[]) {
        return super.compileToV0Message([...this.addressLookupTables, ...(addressLookupTableAccounts ?? [])]);
    }

    public compileToLegacyMessage() {
        if (this.addressLookupTables.length > 0) {
            console.warn(`Warning: transaction might fail due to size limits; compile as a v0 message to utilize the preset address lookup tables`);
        }
        return super.compileToLegacyMessage();
    }

    public async send({ onSign = this.program.transactionHandler?.onSign ?? null, onBeforeConfirm = this.program.transactionHandler?.onBeforeConfirm ?? null, sendOptions, confirmStrategy, commitment = 'confirmed' } : {
        onSign?: ProgramTransactionOnSign<SIGNER> | null,
        onBeforeConfirm?: ProgramTransactionOnBeforeConfirm<IDL> | null,
        sendOptions?: web3.SendOptions,
        confirmStrategy?: web3.TransactionConfirmationStrategy,
        commitment?: web3.Finality,
    } = {}) {
        if (this.program.transactionHandler?.onBeforeSend) {
            await this.program.transactionHandler.onBeforeSend(this as any, this.program);
        }

        let minContextSlot: number | undefined = undefined;
        if (!this.recentBlockhash) {
            const { context, value: { blockhash, lastValidBlockHeight } } = await this.program.connection.getLatestBlockhashAndContext('finalized');
            minContextSlot = context.slot;
            this.recentBlockhash = blockhash;
            if (!confirmStrategy) {
                confirmStrategy = {
                    abortSignal: undefined,
                    ...(confirmStrategy ?? {}),
                    blockhash,
                    lastValidBlockHeight,
                    signature: '',
                }
            }
        }

        if (!confirmStrategy) {
            throw new Error(`confirmStrategy with empty signature should be provided when recentBlockhash is manually set`);
        }

        if (!sendOptions) {
            sendOptions = {};
        }
        if (!sendOptions.minContextSlot) {
            sendOptions.minContextSlot = minContextSlot;
        }

        const msg = this.compileToV0Message();
        let tx = new web3.VersionedTransaction(msg);
        const signersWithSecretKey: web3.Signer[] = [];
        for (const [name, publicKeyOrSigner] of Object.entries(this.signers)) {
            if ((publicKeyOrSigner as web3.Signer).secretKey) {
                signersWithSecretKey.push(publicKeyOrSigner as web3.Signer);
            } else {
                const publicKey = publicKeyOrSigner as web3.PublicKey;
                const sig = onSign ? await onSign(tx, publicKey, name as SIGNER) : null;
                if (sig) {
                    if (typeof (sig as web3.VersionedTransaction).version !== 'undefined') { // some wallets mutates the original tx message.
                        tx = sig as web3.VersionedTransaction;
                    } else if (typeof (sig as web3.Signer).secretKey !== 'undefined') {
                        signersWithSecretKey.push(sig as web3.Signer);
                    } else if (typeof (sig as web3.SignaturePubkeyPair).signature !== 'undefined') {
                        tx.addSignature((sig as web3.SignaturePubkeyPair).publicKey, (sig as web3.SignaturePubkeyPair).signature!);
                    } else {
                        tx.addSignature(publicKey, sig as Buffer | Uint8Array);
                    }
                } else {
                    throw new Error(`unhandled signer key: ${publicKey} (${name}) ... ${JSON.stringify(sig)}`);
                }
            }
        }
        tx.sign(signersWithSecretKey);

        const signature: web3.TransactionSignature = await this.program.connection.sendTransaction(tx, sendOptions);
        (confirmStrategy as any).signature = signature;
        if (onBeforeConfirm) {
            await onBeforeConfirm(tx, confirmStrategy, commitment, this.program);
        }

        const res = await this.program.connection.confirmTransaction(confirmStrategy, commitment);
        if (res.value.err) {
            throw res.value.err;
        }

        const txResult = await this.program.connection.getParsedTransaction(signature, {
            commitment,
            maxSupportedTransactionVersion: 0,
        });

        if (!txResult) {
            throw new Error('no transaction result found');
        }

        if (txResult.meta?.err) {
            throw (this.program.idl.parseError(txResult) ?? txResult?.meta?.err);
        }

        const result = {
            signature,
            descriptions: this.descriptions,
            events: this.program.idl.parseEvents<EVENT>(txResult),
            logs: txResult.meta?.logMessages ?? [],
            error: this.program.idl.parseError(txResult),
        };

        if (!result.error) {
            for (const event of Object.keys(result.events)) {
                if (!this.expectedEvents.includes(event as EVENT)) {
                    console.warn(`Warning: unexpected event: ${event}`);
                }
            }

            for (const expectedEvent of this.expectedEvents) {
                if (!result.events[expectedEvent]?.length) {
                    console.warn(`Warning: missing expected event: ${expectedEvent.toString()}`);
                }
            }
        }

        if (this.program.transactionHandler?.onConfirm) {
            await this.program.transactionHandler.onConfirm(result as any, this.program);
        }

        return result;
    }
}
