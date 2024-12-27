import * as anchor from '@coral-xyz/anchor';
import * as web3 from '@solana/web3.js';

export class Program<IDL extends anchor.Idl> {
    public programID: web3.PublicKey;
    public readonly connection: web3.Connection;
    private readonly anchorProgram: anchor.Program<IDL>;
    private readonly anchorEventParser: anchor.EventParser;
    private readonly anchorErrorMap: Map<number, string>;

    constructor({programID, idl, connection}: {programID: web3.PublicKey, idl: IDL, connection: web3.Connection}) {
        this.programID = programID;
        this.connection = connection;

        // this instance never uses the given wallet, it is just a placeholder.
        const anchorProvider = new anchor.AnchorProvider(connection, new anchor.Wallet(new anchor.web3.Keypair()));
        this.anchorProgram = new anchor.Program<IDL>({ ...idl, address: programID.toString() }, anchorProvider);
        this.anchorEventParser = new anchor.EventParser(this.programID, this.anchorProgram.coder);
        this.anchorErrorMap = anchor.parseIdlErrors(this.anchorProgram.idl);
    }

    public async run<EVENTS extends ProgramEventName<IDL>>(args: {
        logs?: any[],
        instructions: (Promise<web3.TransactionInstruction> | web3.TransactionInstruction)[],
        signers?: web3.Signer[],
        events?: EVENTS[],
        skipPreflight?: boolean,
        requestHeapFrameBytes?: number,
        setComputeUnitLimitUnits?: number,
        setComputeUnitPriceMicroLamports?: number,
    }) {

        // TODO: ... here some interceptor needed.. for logging and more..
        // or just.. provide create instructions & remaining accounts for UserContext usage?
        // for ledger Signer interface as Promise? signers: PromiseLike<web3.Signer[]> ?
        return {
            signature: new web3.PublicKey('dd'),
            error: this.parseError(null as any),
            event: this.parseEvents<EVENTS>(null as any),
        };
    }

    /// helper methods for typing arbitrary data into IDL types
    public asEvent<K extends ProgramEventName<IDL>>(value: anchor.IdlEvents<IDL>[K]) {
        return value;
    }

    public asAccount<K extends ProgramAccountName<IDL>>(value: anchor.IdlAccounts<IDL>[K]) {
        return value;
    }

    public asType<K extends ProgramTypeName<IDL>>(value: anchor.IdlTypes<IDL>[K]) {
        return value;
    }

    protected get methods() {
        return this.anchorProgram.methods;
    }

    protected get accounts(): anchor.AccountNamespace<IDL> {
        return this.anchorProgram.account;
    }

    protected parseEvents<K extends ProgramEventName<IDL>>(tx: web3.ParsedTransactionWithMeta) {
        const events: {[k in K]: anchor.IdlEvents<IDL>[k][]} = {} as any;

        // parse event data from emit!
        const eventsFromEmitMacro = this.anchorEventParser.parseLogs(tx.meta?.logMessages ?? [], false);
        while (true) {
            const event = eventsFromEmitMacro.next().value;
            if (!event) break;
            const name = event.name as K;
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
                            const name = event.name as K;
                            (events[name] = events[name] ?? []).push(event as any);
                        }
                    }
                }
            }
        }

        return events;
    }

    protected parseError(tx: web3.ParsedTransactionWithMeta): Error|null {
        let err = anchor.translateError({ logs: tx.meta?.logMessages ?? [] }, this.anchorErrorMap);
        if (!err) {
            err = anchor.AnchorError.parse(tx.meta?.logMessages ?? [])
        }
        return err ?? null;
    }

    protected getConstant(name: ProgramConstantName<IDL>): string {
        return this.anchorProgram.idl.constants!.find(a => a.name == name)!.value;
    }

    protected getConstantAsPublicKey(name: ProgramConstantName<IDL>): web3.PublicKey {
        return new web3.PublicKey(this.getConstant(name));
    }

    protected createAddressBook<KEYS extends string>() {
        return new AddressBook<KEYS>({ program: this });
    }
}

export type ProgramEventName<IDL extends anchor.Idl> = IDL['events'] extends Array<infer U>
    ? U extends { name: infer N }
        ? N extends string
            ? N
            : never
        : never
    : never;

export type ProgramAccountName<IDL extends anchor.Idl> = IDL['accounts'] extends Array<infer U>
    ? U extends { name: infer N }
        ? N extends string
            ? N
            : never
        : never
    : never;

export type ProgramTypeName<IDL extends anchor.Idl> = IDL['types'] extends Array<infer U>
    ? U extends { name: infer N }
        ? N extends string
            ? N
            : never
        : never
    : never;

export type ProgramConstantName<IDL extends anchor.Idl> = IDL['constants'] extends Array<infer U>
    ? U extends { name: infer N }
        ? N extends string
            ? N
            : never
        : never
    : never;

export class AddressBook<KEYS extends string> {
    public readonly addresses = new Map<KEYS, web3.PublicKey>();
    private program: Program<any>;

    constructor({ program }: { program: Program<any> }) {
        this.program = program;
    }

    public add(name: KEYS, address: web3.PublicKey) {
        this.addresses.set(name, address);
    }

    public addAll(addresses: Partial<{[name in KEYS]: web3.PublicKey}>) {
        for (let [name, address] of Object.entries(addresses)) {
            this.addresses.set(name as KEYS, address as web3.PublicKey);
        }
    }

    public get(name: KEYS): web3.PublicKey|null {
        return this.addresses.get(name) ?? null;
    }

    public async syncLookupTable({ lookupTableAddress = null, payer, authority }: {
        lookupTableAddress: web3.PublicKey|null,
        payer: web3.Signer,
        authority: web3.Signer,
    }) {
        const exists = lookupTableAddress ? await this.program.connection.getAccountInfo(lookupTableAddress).then(() => true).catch(() => false) : false;
        if (!exists) {
            const recentSlot = await this.program.connection.getSlot({commitment: 'recent'});
            const [createIx, newLookupTableAddress] = web3.AddressLookupTableProgram.createLookupTable({
                authority: authority.publicKey,
                payer: payer.publicKey,
                recentSlot,
            });
            await this.program.run({
                logs: [`create a new address lookup table`],
                instructions: [createIx],
                signers: [payer, authority],
            });
            lookupTableAddress = newLookupTableAddress;
        }

        const lookupTable = await this.program.connection
            .getAddressLookupTable(lookupTableAddress!, { commitment: 'confirmed' })
            .then(res => res.value);

        const existingAddresses = new Set(lookupTable?.state.addresses.map(a => a.toString()) ?? []);
        const newAddresses = Array.from(this.addresses.values()).filter(address => !existingAddresses.has(address.toString()));
        const listOfNewAddresses = newAddresses.reduce((listOfNewAddresses, address) =>  {
            if (!listOfNewAddresses[0] || listOfNewAddresses[0].length == 27) { // 27 (addresses) + 5 (admin/authority, payer, alt_program, alt, system_program)
                listOfNewAddresses.unshift([address]);
            } else {
                listOfNewAddresses[0].push(address);
            }
            return listOfNewAddresses;
        }, [] as web3.PublicKey[][]);

        let size = existingAddresses.size;
        for (let addresses of listOfNewAddresses) {
            await this.program.run({
                logs: [`extend the address lookup table (${size} + ${addresses.length})`],
                instructions: [
                    web3.AddressLookupTableProgram.extendLookupTable({
                        lookupTable: lookupTableAddress!,
                        authority: authority.publicKey,
                        payer: payer.publicKey,
                        addresses,
                    }),
                ],
                signers: [payer, authority],
            });
            size += addresses.length;
        }

        return await this.program.connection
            .getAddressLookupTable(lookupTableAddress!, { commitment: 'confirmed' })
            .then(res => res.value!);
    }
}
