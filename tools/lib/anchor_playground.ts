import util from "node:util";
import * as anchor from '@coral-xyz/anchor';
import * as web3 from '@solana/web3.js';
import {getLogger} from './logger';
import {Keychain} from './keychain';
import {WORKSPACE_PROGRAM_NAME} from "./types";
import {IdlTypes} from "@coral-xyz/anchor/dist/cjs/program/namespace/types";
// @ts-ignore
import chalk from "chalk";
import {PartiallyDecodedInstruction} from "@solana/web3.js";

const {logger, LOG_PAD_SMALL, LOG_PAD_LARGE } = getLogger('anchor');

anchor.BN.prototype.toJSON = function() {
    return this.toString(); //.replace(/\B(?=(\d{3})+(?!\d))/g, "_");
}
anchor.BN.prototype[Symbol.for("nodejs.util.inspect.custom")] = function() {
    return chalk.yellow(this.toString().replace(/\B(?=(\d{3})+(?!\d))/g, "_"));
}
web3.PublicKey.prototype[Symbol.for("nodejs.util.inspect.custom")] = anchor.web3.PublicKey.prototype[Symbol.for("nodejs.util.inspect.custom")] = function () {
    return chalk.blue(this.toString());
}

const opt = util.inspect.defaultOptions;
opt.depth = 10;
opt.colors = true;
opt.customInspect = true;
opt.maxArrayLength = 10;

export type AnchorPlaygroundConfig<IDL extends anchor.Idl, KEYS extends string> = {
    provider: anchor.Provider,
    idl: IDL,
    keychain: Keychain<KEYS>,
};

export class AnchorPlayground<IDL extends anchor.Idl, KEYS extends string> {
    public readonly programName: WORKSPACE_PROGRAM_NAME;
    public readonly keychain: Keychain<KEYS>;
    protected readonly provider: anchor.Provider;
    protected readonly program: anchor.Program<IDL>;
    protected readonly eventParser: anchor.EventParser;
    protected readonly idlErrorsMap: Map<number, string>;

    constructor(args: AnchorPlaygroundConfig<IDL, KEYS>) {
        let {idl, keychain, provider} = args;
        this.programName = keychain.programName;
        this.provider = provider;
        this.keychain = keychain;

        logger.notice(`initializing ${this.programName} playground`);

        logger.info(`connected to:`.padEnd(LOG_PAD_LARGE), provider.connection.rpcEndpoint);

        if (idl?.metadata?.name != this.programName) {
            throw new Error('program name and idl not matched');
        }

        const programAddress = keychain.programKeypair.publicKey.toString();
        if (!programAddress) {
            throw new Error(`program keypair not initialized for ${this.programName}`);
        }
        if (idl.address != programAddress) {
            logger.debug('updating idl address due to mismatched program keypair');
            idl.address = programAddress;
        }

        this.program = new anchor.Program<IDL>(idl, this.provider);
        this.eventParser = new anchor.EventParser(this.program.programId, this.program.coder);
        this.idlErrorsMap = anchor.parseIdlErrors(this.program.idl);

        logger.info(`loaded program ${this.programName}:`.padEnd(LOG_PAD_LARGE), programAddress);
    }

    private addressLookupTables: web3.AddressLookupTableAccount[] = [];
    protected async setAddressLookupTableAddresses(addresses: web3.PublicKey[]) {
        this.addressLookupTables = await Promise.all(
            addresses.map(address => this.connection.getAddressLookupTable(address, { commitment: 'confirmed' }).then(res => res.value)),
        );
        await this.sleep(1);
        logger.notice('set address lookup tables for future transactions:'.padEnd(LOG_PAD_LARGE), addresses);
    }

    public async run<EVENTS extends ExtractEventNames<IDL>>(args: {
        instructions: (Promise<web3.TransactionInstruction> | web3.TransactionInstruction)[],
        signers?: web3.Signer[],
        signerNames?: KEYS[],
        events?: EVENTS[],
        skipPreflight?: boolean,
        requestHeapFrameBytes?: number,
        setComputeUnitLimitUnits?: number,
        setComputeUnitPriceMicroLamports?: number,
    }) {
        let txSig: string | null = null;
        try {
            // prepare instructions
            let {instructions, signers = [], signerNames = [], skipPreflight = false, requestHeapFrameBytes, setComputeUnitLimitUnits, setComputeUnitPriceMicroLamports} = args;

            // get recent block hash
            const { blockhash, lastValidBlockHeight } = await this.connection.getLatestBlockhash();
            const tx = new web3.VersionedTransaction(
                new web3.TransactionMessage({
                    payerKey: this.keychain.wallet.publicKey,
                    recentBlockhash: blockhash,
                    instructions: [
                        ...(typeof requestHeapFrameBytes != 'undefined' ? [
                            web3.ComputeBudgetProgram.requestHeapFrame({
                                bytes: requestHeapFrameBytes, // should be multiple of 1024
                            }),
                        ] : []),
                        ...(typeof setComputeUnitPriceMicroLamports != 'undefined' ? [
                            web3.ComputeBudgetProgram.setComputeUnitPrice({
                                microLamports: setComputeUnitPriceMicroLamports,
                            }),
                        ] : []),
                        ...(typeof setComputeUnitLimitUnits != 'undefined' ? [
                            web3.ComputeBudgetProgram.setComputeUnitLimit({
                                units: setComputeUnitLimitUnits,
                            })
                        ] : []),
                        ...await Promise.all(instructions),
                    ]
                }).compileToV0Message(this.addressLookupTables)
            );

            // sign with wallet to pay fee
            signers.push(this.keychain.wallet);

            // sign from keypair loader
            for (const keypairName of signerNames) {
                const { local, ledger } = await this.keychain.signTransaction(keypairName, tx);
                if (local) {
                    signers.push(local);
                    logger.debug(`${keypairName} (signer)`.padEnd(LOG_PAD_LARGE), `${local.publicKey.toString()}`)
                } else if (ledger) {
                    tx.addSignature(ledger.publicKey, ledger.signature);
                    logger.debug(`${keypairName} (signer)`.padEnd(LOG_PAD_LARGE), `${ledger.publicKey.toString()}`)
                }
            }

            // sign with given signers
            tx.sign(signers);

            // send transaction
            txSig = await this.connection.sendTransaction(tx, {
                skipPreflight,
            });
            await this.connection.confirmTransaction({
                abortSignal: undefined,
                lastValidBlockHeight,
                blockhash,
                signature: txSig,
            }, 'confirmed');

            // get result and parse events and errors
            const txResult = await this.connection.getParsedTransaction(txSig, {
                commitment: 'confirmed',
                maxSupportedTransactionVersion: 0,
            });

            if (txResult.meta.err) {
                throw anchor.translateError({ logs: txResult.meta.logMessages }, this.idlErrorsMap);
            }

            logger.info(`transaction confirmed (${tx.serialize().length}/1232 byte)`.padEnd(LOG_PAD_LARGE), txSig.substring(0, 40) + ' ...');
            return {
                txSig,
                error: anchor.AnchorError.parse(txResult.meta.logMessages),
                event: this.parseEvents<EVENTS>(txResult.meta, args.events),
            };

        } catch (err) {
            logger.error(`transaction failed`.padEnd(LOG_PAD_LARGE), txSig ? (txSig.substring(0, 40) + ' ...') : null);
            throw err;
        }
    }

    public get wallet() {
        return this.keychain.wallet;
    }

    public get walletAddress() {
        return this.wallet.publicKey.toString();
    }

    public get connection(): web3.Connection {
        return this.provider.connection as unknown as web3.Connection;
    }

    public get programId(): web3.PublicKey {
        return this.program.programId;
    }

    public get methods(): anchor.MethodsNamespace<IDL> {
        return this.program.methods;
    }

    public get account(): anchor.AccountNamespace<IDL> {
        return this.program.account;
    }

    public async tryAirdrop(account: web3.PublicKey, lamports: anchor.BN = new anchor.BN(100 * web3.LAMPORTS_PER_SOL)) {
        let [txSig, { blockhash, lastValidBlockHeight }] = await Promise.all([
            this.connection.requestAirdrop(
                account,
                lamports.toNumber()
            ),
            this.connection.getLatestBlockhash(),
        ]);
        await this.connection.confirmTransaction({
            abortSignal: undefined,
            lastValidBlockHeight,
            blockhash,
            signature: txSig,
        });
        const balance = new anchor.BN((await this.connection.getBalance(account)).toString());
        logger.debug(`SOL airdropped (+${this.lamportsToSOL(lamports)}): ${this.lamportsToSOL(balance)}`.padEnd(LOG_PAD_LARGE), account.toString());
    }

    public getConstant(name: ExtractConstantNames<IDL>): string {
        return this.program.idl.constants.find(a => a.name == name).value;
    }

    public getConstantAsPublicKey(name: ExtractConstantNames<IDL>): web3.PublicKey {
        return new web3.PublicKey(this.getConstant(name));
    }

    public asType<K extends ExtractTypeNames<IDL>>(value: IdlTypes<IDL>[K]) {
        return value;
    }

    private parseEvents<K extends ExtractEventNames<IDL>>(meta: web3.ParsedTransactionMeta, eventNames: K[] = []) {
        const events: {[k in K]: anchor.IdlEvents<IDL>[k]} = {} as any;
        const required = new Set(eventNames);
        const found = new Set();
        const ignored = new Set();

        // parse event data from emit! -> logs
        const it = this.eventParser.parseLogs(meta.logMessages, false) as unknown as Generator<anchor.Event<IDL['events'][number]>>;
        while (true) {
            const event = it.next();
            const name = event?.value?.name;
            if (!name) break;
            if (required.has(name)) {
                events[name] = event.value.data;
                found.add(name);
            } else {
                ignored.add(name);
            }
        }

        // to parse event data from emit_cpi! -> self-CPI ix data
        for (const ixs of meta.innerInstructions) {
            for (const ix of ixs.instructions) {
                if (ix.programId.equals(this.programId)) {
                    const data = (ix as any).data as string ?? null;
                    if (data) {
                        const ixData = anchor.utils.bytes.bs58.decode(data);
                        const eventData = anchor.utils.bytes.base64.encode(ixData.subarray(8)); // remove ix discriminant
                        const event = this.program.coder.events.decode(eventData);
                        if (event) {
                            const name = event.name as any;
                            if (required.has(name)) {
                                events[name] = event.data;
                                found.add(name);
                            } else {
                                ignored.add(name);
                            }
                        }
                    }
                }
            }
        }

        // ...

        if (required.size != found.size) {
            const notFound = new Set();
            required.forEach(elem => notFound.add(elem));
            found.forEach(elem => notFound.delete(elem));
            logger.warn(`event not found: ${Array.from(notFound.values()).join(', ')}`);
        }
        if (ignored.size > 0) {
            logger.fatal(`event ignored: ${Array.from(ignored.values()).join(', ')}`)
        }
        return events;
    }

    public static binToString(buf: Uint8Array | number[]) {
        const codes = [];
        for (let v of buf) {
            if (v == 0) break;
            codes.push(v);
        }
        return String.fromCharCode.apply(null, codes)
    }
    public readonly binToString = AnchorPlayground.binToString;

    public static binIsEmpty(buf: Uint8Array | number[]) {
        return buf.every(v => v == 0);
    }
    public readonly binIsEmpty = AnchorPlayground.binIsEmpty;

    public lamportsToSOL(lamports: anchor.BN): string {
        return this.lamportsToX(lamports, 9, 'SOL');
    }

    public lamportsToX(lamports: anchor.BN, decimals: number, symbol: string): string {
        const unit = lamports.div(new anchor.BN(10 ** decimals));
        const remainder = lamports.mod(new anchor.BN(10 ** decimals));
        return `${unit.toString()}.${remainder.toString().padStart(decimals, '0')} ${symbol}`;
    }

    // it returns over-slept number of slots, zero means it slept as much as requested duration exactly.
    public async sleep(slotDuration: number): Promise<number> {
        const startedSlot = (await this.connection.getSlot('confirmed'));
        const targetSlot = startedSlot + slotDuration;
        return this.sleepUntil(targetSlot, startedSlot)
    }

    public async sleepUntil(targetSlot: number, startedSlot: number|null = null): Promise<number> {
        startedSlot = startedSlot !== null ? startedSlot : (await this.connection.getSlot('confirmed'));

        return new Promise(resolve => {
            let intervalID = setInterval(async () => {
                const endedSlot = await this.connection.getSlot('confirmed');
                if (targetSlot <= endedSlot) {
                    clearInterval(intervalID);
                    resolve(endedSlot - targetSlot);
                    logger.debug(`slept for ${endedSlot - startedSlot} slots, started=${startedSlot}, ended=${endedSlot}, requested=${targetSlot}`);
                }
            }, 200);
        });
    }
}

export type ExtractEventNames<T extends anchor.Idl> = T['events'] extends Array<infer U>
    ? U extends { name: infer N }
        ? N extends string
            ? N
            : never
        : never
    : never;

export type ExtractTypeNames<T extends anchor.Idl> = T['types'] extends Array<infer U>
    ? U extends { name: infer N }
        ? N extends string
            ? N
            : never
        : never
    : never;

export type ExtractConstantNames<T extends anchor.Idl> = T['constants'] extends Array<infer U>
    ? U extends { name: infer N }
        ? N extends string
            ? N
            : never
        : never
    : never;
