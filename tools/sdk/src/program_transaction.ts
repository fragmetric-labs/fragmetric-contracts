import * as web3 from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import {Program, ProgramEvent} from "./program";

export type ProgramTransactionSigner<SIGNER extends string> = (name: 'payer' | SIGNER, publicKey: web3.PublicKey, buffer: Buffer) => Promise<web3.SignaturePubkeyPair | web3.Signer> | web3.SignaturePubkeyPair | web3.Signer;
export type ProgramTransactionBeforeHook<IDL extends anchor.Idl> = (tx: ProgramTransactionMessage<IDL, any, keyof ProgramEvent<IDL>>, program: Program<IDL>) => Promise<void>;
export type ProgramTransactionAfterHook<IDL extends anchor.Idl> = (result: Awaited<ReturnType<typeof ProgramTransactionMessage.prototype.send>>, program: Program<IDL>) => Promise<void>;
export type ProgramTransactionHandler<IDL extends anchor.Idl> = {
    signer: ProgramTransactionSigner<'payer'|string>,
    before: ProgramTransactionBeforeHook<IDL>,
    after: ProgramTransactionAfterHook<IDL>,
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

    public async send({ signer = this.program.transactionHandler?.signer ?? null, sendOptions, confirmOptions, commitment = 'confirmed' } : {
        signer?: ProgramTransactionSigner<SIGNER> | null,
        sendOptions?: web3.SendOptions,
        confirmOptions?: web3.TransactionConfirmationStrategy,
        commitment?: web3.Finality,
    } = {}) {
        if (this.program.transactionHandler?.before) {
            await this.program.transactionHandler.before(this as any, this.program);
        }

        if (!this.recentBlockhash) {
            const { blockhash, lastValidBlockHeight } = await this.program.connection.getLatestBlockhash(commitment);
            this.recentBlockhash = blockhash;
            confirmOptions = {
                abortSignal: undefined,
                ...(confirmOptions ?? {}),
                blockhash,
                lastValidBlockHeight,
                signature: '',
            }
        }

        const tx = new web3.VersionedTransaction(this.compileToV0Message());
        let message: Buffer | null = null;
        const signersWithSecretKey: web3.Signer[] = [];
        for (const [name, key] of Object.entries(this.signers)) {
            if ((key as web3.Signer).secretKey) {
                signersWithSecretKey.push(key as web3.Signer);
            } else {
                const publicKey = key as web3.PublicKey;
                if (!message) {
                    message = Buffer.from(tx.message.serialize());
                }
                const sigPair = signer ? await signer(name as SIGNER, publicKey, message) : null;
                if (!sigPair) {
                    throw new Error(`unhandled signer key: ${key} (${name})`);
                } else if (!sigPair.publicKey.equals(publicKey)) {
                    throw new Error(`signed key does not match with the requested signer key: ${publicKey} (${name}) != ${sigPair.publicKey}`);
                }
                if ((sigPair as web3.Signer).secretKey) {
                    signersWithSecretKey.push(sigPair as web3.Signer)
                } else {
                    tx.addSignature(sigPair.publicKey, (sigPair as web3.SignaturePubkeyPair).signature!);
                }
            }
        }
        tx.sign(signersWithSecretKey);

        const signature: web3.TransactionSignature = await this.program.connection.sendTransaction(tx, sendOptions);
        const res = await this.program.connection.confirmTransaction({
            ...(confirmOptions as web3.TransactionConfirmationStrategy),
            signature,
        }, commitment);
        if (res.value.err) {
            throw res.value.err;
        }

        const txResult = await this.program.connection.getParsedTransaction(signature, {
            commitment,
            maxSupportedTransactionVersion: 0,
        });

        if (!txResult) {
            throw new Error("no transaction result found");
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
                    console.warn(`Warning: missing expected event: ${expectedEvent}`);
                }
            }
        }

        if (this.program.transactionHandler?.after) {
            await this.program.transactionHandler.after(result as any, this.program);
        }

        return result;
    }
}
