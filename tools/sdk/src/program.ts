import * as anchor from '@coral-xyz/anchor';
import * as web3 from '@solana/web3.js';

export class Program<IDL extends anchor.Idl> {
    public programID: web3.PublicKey;
    public readonly connection: web3.Connection;
    private readonly anchorProgram: anchor.Program<IDL>;
    private readonly anchorEventParser: anchor.EventParser;
    private readonly anchorErrorMap: Map<number, string>;

    public readonly transactionHandlers: ProgramTransactionHandlers | null = null;

    public static readonly defaultClusterURL = {
        mainnet: 'https://api.mainnet-beta.solana.com',
        devnet: 'https://api.devnet.solana.com',
        testnet: 'https://api.testnet.solana.com',
        local: 'http://0.0.0.0:8899',
    };

    public readonly cluster: keyof typeof Program['defaultClusterURL'];

    constructor({cluster, programID, idl, connection, transactionHandlers = null }: {
        cluster: keyof typeof Program['defaultClusterURL'],
        programID: web3.PublicKey,
        idl: IDL,
        connection?: web3.Connection,
        transactionHandlers?: ProgramTransactionHandlers | null,
    }) {
        this.cluster = cluster;
        this.programID = programID;

        // create a default connection
        this.connection = connection ?? new web3.Connection(Program.defaultClusterURL[cluster] ?? Program.defaultClusterURL.local, {
            commitment: 'confirmed',
            disableRetryOnRateLimit: true,
        });

        // check invalid rpc configuration
        const usingDefaultClusterURL = Object.entries(Program.defaultClusterURL).find(([_, url]) => url == this.connection.rpcEndpoint);
        if (usingDefaultClusterURL && usingDefaultClusterURL[0] != cluster) {
            throw new Error(`provided connection URL does not match the specified cluster: ${cluster} != ${usingDefaultClusterURL[0]} (${usingDefaultClusterURL[1]})`);
        }

        this.transactionHandlers = transactionHandlers;

        // this instance never uses the given wallet, it is just a placeholder.
        const anchorProvider = new anchor.AnchorProvider(this.connection, new anchor.Wallet(new anchor.web3.Keypair()));
        this.anchorProgram = new anchor.Program<IDL>({ ...idl, address: programID.toString() }, anchorProvider);
        this.anchorEventParser = new anchor.EventParser(this.programID, this.anchorProgram.coder);
        this.anchorErrorMap = anchor.parseIdlErrors(this.anchorProgram.idl);

        const x= this.idl.parseEvents<'FundManagerUpdatedFund'>({} as any);
        const y = x.operatorUpdatedFundPrices[0];
    }

    public readonly idl = {
        getConstant: (name: ProgramConstantName<IDL>): string => {
            return this.anchorProgram.idl.constants!.find(a => a.name == name)!.value;
        },
        getConstantAsPublicKey: (name: ProgramConstantName<IDL>): web3.PublicKey => {
            return new web3.PublicKey(this.idl.getConstant(name));
        },
        parseEvents: <EVENT extends ProgramEventName<IDL>>(tx: web3.ParsedTransactionWithMeta, expectedEventNames: EVENT[] = []) => {
            const events: {[k in EVENT]: (ProgramEvent<IDL>[EVENT])[]} = {} as any;

            // parse event data from emit!
            const eventsFromEmitMacro = this.anchorEventParser.parseLogs(tx.meta?.logMessages ?? [], false);
            while (true) {
                const event = eventsFromEmitMacro.next().value;
                if (!event) break;
                const name = event.name as EVENT;
                (events[name] = events[name] ?? []).push(event as any);
            }

            // parse event data from emit_cpi!
            for (const ixs of tx.meta?.innerInstructions ?? []) {
                for (const ix of ixs.instructions) {
                    if (ix.programId.equals(this.programID)) {
                        const data = (ix as any).data as string|null ?? null;
                        if (data) {
                            const ixData = anchor.utils.bytes.bs58.decode(data);
                            const eventData = anchor.utils.bytes.base64.encode(ixData.subarray(8)); // remove ix discriminant
                            const event = this.anchorProgram.coder.events.decode(eventData);
                            if (event) {
                                const name = event.name as EVENT;
                                (events[name] = events[name] ?? []).push(event as any);
                            }
                        }
                    }
                }
            }

            return events;
        },
        parseError: (tx: web3.ParsedTransactionWithMeta): Error|null => {
            let err = tx.meta?.err ? anchor.translateError({ logs: tx.meta?.logMessages ?? [] }, this.anchorErrorMap) : null;
            if (!err) err = anchor.AnchorError.parse(tx.meta?.logMessages ?? []);
            return err ?? null;
        },
    }

    public get programMethods() {
        return this.anchorProgram.methods;
    }

    public get programAccounts(): anchor.AccountNamespace<IDL> {
        return this.anchorProgram.account;
    }

    public createUnsignedTransactionMessage<SIGNER extends string, EVENT extends ProgramEventName<IDL>>(args: Omit<ConstructorParameters<typeof ProgramTransactionMessage<IDL, SIGNER, EVENT>>[0], 'program'>) {
        return new ProgramTransactionMessage<IDL, SIGNER, EVENT>({
            ...args,
            program: this,
        });
    }
}

export type ProgramAccount<IDL extends anchor.Idl> = anchor.IdlAccounts<IDL>;

export type ProgramEvent<IDL extends anchor.Idl> = anchor.IdlEvents<IDL>;

export type ProgramType<IDL extends anchor.Idl> = anchor.IdlTypes<IDL>;

type ProgramEventName<IDL extends anchor.Idl> = IDL['events'] extends Array<infer U>
    ? U extends { name: infer N }
        ? N extends string
            ? N
            : never
        : never
    : never;

type ProgramAccountName<IDL extends anchor.Idl> = IDL['accounts'] extends Array<infer U>
    ? U extends { name: infer N }
        ? N extends string
            ? N
            : never
        : never
    : never;

type ProgramTypeName<IDL extends anchor.Idl> = IDL['types'] extends Array<infer U>
    ? U extends { name: infer N }
        ? N extends string
            ? N
            : never
        : never
    : never;

type ProgramConstantName<IDL extends anchor.Idl> = IDL['constants'] extends Array<infer U>
    ? U extends { name: infer N }
        ? N extends string
            ? N
            : never
        : never
    : never;

export type ProgramTransactionSigner = (name: string, publicKey: web3.PublicKey, message: Buffer) => Promise<web3.SignaturePubkeyPair | web3.Signer> | web3.SignaturePubkeyPair | web3.Signer;
export type ProgramTransactionBeforeHook = (tx: ProgramTransactionMessage<any, any, never>) => Promise<void>;
export type UnsignedTransactionAfterHook = (result: Awaited<ReturnType<typeof ProgramTransactionMessage.prototype.signAndSend>>) => Promise<void>;
export type ProgramTransactionHandlers = {
    signer: ProgramTransactionSigner,
    before: ProgramTransactionBeforeHook,
    after: UnsignedTransactionAfterHook,
};

export class ProgramTransactionMessage<IDL extends anchor.Idl, SIGNER extends string, EVENT extends ProgramEventName<IDL>> extends web3.TransactionMessage {
    public readonly descriptions: any[] | null;
    public readonly signers: { [k in "payer" | SIGNER]: web3.PublicKey | web3.Signer };
    public readonly addressLookupTables: web3.AddressLookupTableAccount[] = [];
    private readonly program: Program<any>;
    private readonly expectedEventNames: EVENT[];

    constructor({ descriptions = [], instructions, expectedEventNames = [], signers, recentBlockhash = null, addressLookupTables = [], program }: {
        descriptions?: any[] | null,
        expectedEventNames?: EVENT[],
        instructions: web3.TransactionInstruction[],
        signers: {[k in SIGNER | 'payer']: web3.PublicKey | web3.Signer},
        recentBlockhash?: web3.Blockhash | null,
        addressLookupTables?: web3.AddressLookupTableAccount[],
        program: Program<any>,
    }) {
        super({
            payerKey: (signers.payer as web3.Signer).secretKey ? (signers.payer as web3.Signer).publicKey : signers.payer as web3.PublicKey,
            recentBlockhash: recentBlockhash ?? '',
            instructions,
        });
        this.descriptions = descriptions;
        this.expectedEventNames = expectedEventNames;
        this.signers = signers;
        this.addressLookupTables = addressLookupTables;
        this.program = program;
    }

    public compileToV0Message(addressLookupTableAccounts?: web3.AddressLookupTableAccount[]) {
        return super.compileToV0Message([...this.addressLookupTables, ...(addressLookupTableAccounts ?? [])]);
    }

    public compileToLegacyMessage() {
        if (this.addressLookupTables.length > 0) {
            console.warn(`transaction might fail due to size limits; compile as a v0 message to utilize the preset address lookup tables`);
        }
        return super.compileToLegacyMessage();
    }

    public async signAndSend({ signer = this.program.transactionHandlers?.signer ?? null, sendOptions, confirmOptions, commitment = 'confirmed' } : {
        signer?: ProgramTransactionSigner | null,
        sendOptions?: web3.SendOptions,
        confirmOptions?: web3.TransactionConfirmationStrategy,
        commitment?: web3.Finality,
    } = {}) {
        if (this.program.transactionHandlers?.before) {
            await this.program.transactionHandlers.before(this);
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
                const sigPair = signer ? await signer(name, publicKey, message) : null;
                if (!sigPair) {
                    throw new Error(`unhandled signer key: ${name} (${key})`);
                } else if (!sigPair.publicKey.equals(publicKey)) {
                    console.warn(`signed key does not match with the requested signer key: ${publicKey} (${name}) != ${sigPair.publicKey}`);
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
            logs: txResult.meta?.logMessages ?? [],
            error: this.program.idl.parseError(txResult),
            events: this.program.idl.parseEvents<EVENT>(txResult),
        };

        if (this.program.transactionHandlers?.after) {
            await this.program.transactionHandlers.after(result);
        }

        return result;
    }
}

export class AddressBook<KEYS extends string> {
    public readonly entries = new Map<KEYS, web3.PublicKey>();
    private readonly program: Program<any>;

    constructor({ program }: { program: Program<any> }) {
        this.program = program;
    }

    public add(name: KEYS, address: web3.PublicKey) {
        this.entries.set(name, address);
    }

    public addAll(addresses: Partial<{[name in KEYS]: web3.PublicKey}>) {
        for (let [name, address] of Object.entries(addresses)) {
            this.entries.set(name as KEYS, address as web3.PublicKey);
        }
    }

    public get(name: KEYS): web3.PublicKey|null {
        return this.entries.get(name) ?? null;
    }

    public async syncWithLookupTable({ lookupTableAddress = null, payer, authority, signer = this.program.transactionHandlers?.signer ?? null }: {
        lookupTableAddress: web3.PublicKey|null,
        payer: web3.PublicKey,
        authority: web3.PublicKey,
        signer?: ProgramTransactionSigner | null,
    }) {
        const exists = lookupTableAddress ? await this.program.connection.getAccountInfo(lookupTableAddress).then(() => true).catch(() => false) : false;
        if (!exists) {
            const recentSlot = await this.program.connection.getSlot({commitment: 'recent'});
            const [createIx, newLookupTableAddress] = web3.AddressLookupTableProgram.createLookupTable({
                authority,
                payer,
                recentSlot,
            });
            await this.program.createUnsignedTransactionMessage({
                descriptions: [`create a new address lookup table`],
                instructions: [createIx],
                signers: { payer, authority },
            })
                .signAndSend({ signer });
            lookupTableAddress = newLookupTableAddress;
        }

        let lookupTable = await this.program.connection
            .getAddressLookupTable(lookupTableAddress!, { commitment: 'confirmed' })
            .then(res => res.value);

        const existingAddresses = new Set(lookupTable?.state.addresses.map(a => a.toString()) ?? []);
        const newAddresses = Array.from(this.entries.values()).filter(address => !existingAddresses.has(address.toString()));
        const listOfNewAddresses = newAddresses.reduce((listOfNewAddresses, address) =>  {
            if (!listOfNewAddresses[0] || listOfNewAddresses[0].length == 27) { // 27 (addresses) + 5 (admin/authority, payer, alt_program, alt, system_program)
                listOfNewAddresses.unshift([address]);
            } else {
                listOfNewAddresses[0].push(address);
            }
            return listOfNewAddresses;
        }, [] as web3.PublicKey[][]);

        if (listOfNewAddresses.length > 0) {
            let size = existingAddresses.size;
            for (let addresses of listOfNewAddresses) {
                await this.program.createUnsignedTransactionMessage({
                    descriptions: [`extend the address lookup table (${size} + ${addresses.length})`],
                    instructions: [
                        web3.AddressLookupTableProgram.extendLookupTable({
                            lookupTable: lookupTableAddress!,
                            authority,
                            payer,
                            addresses,
                        }),
                    ],
                    signers: { payer, authority },
                })
                    .signAndSend({ signer });
                size += addresses.length;
            }

            lookupTable = await this.program.connection
                .getAddressLookupTable(lookupTableAddress!, { commitment: 'confirmed' })
                .then(res => res.value!);
        }

        return lookupTable!;
    }
}
