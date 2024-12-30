import * as anchor from '@coral-xyz/anchor';
import * as web3 from '@solana/web3.js';
import {ProgramTransactionHandler, ProgramTransactionMessage, ProgramTransactionSigner} from "./program_transaction";

export class Program<IDL extends anchor.Idl> {
    public programID: web3.PublicKey;
    public readonly connection: web3.Connection;
    private readonly anchorProgram: anchor.Program<IDL>;
    private readonly anchorEventParser: anchor.EventParser;
    private readonly anchorErrorMap: Map<number, string>;

    public readonly transactionHandler: ProgramTransactionHandler<IDL> | null = null;

    public static readonly defaultClusterURL = {
        mainnet: 'https://api.mainnet-beta.solana.com',
        devnet: 'https://api.devnet.solana.com',
        testnet: 'https://api.testnet.solana.com',
        local: 'http://0.0.0.0:8899',
    };

    public readonly cluster: keyof typeof Program['defaultClusterURL'];

    constructor({cluster, programID, idl, connection, transactionHandler = null }: {
        cluster: keyof typeof Program['defaultClusterURL'],
        programID: web3.PublicKey,
        idl: IDL,
        connection?: web3.Connection,
        transactionHandler?: ProgramTransactionHandler<IDL> | null,
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

        this.transactionHandler = transactionHandler
            ? Object.fromEntries(Object.entries(transactionHandler).map(([k, v]) => [k, v.bind(this)])) as ProgramTransactionHandler<IDL>
            : null;

        this.anchorProgram = new anchor.Program<IDL>({ ...idl, address: programID.toString() }, new anchor.AnchorProvider(this.connection, null as any));
        this.anchorEventParser = new anchor.EventParser(this.programID, this.anchorProgram.coder);
        this.anchorErrorMap = anchor.parseIdlErrors(this.anchorProgram.idl);
    }

    // TODO: universal cache, localstorage or memory
    // public readonly cache: Map<string, any> = new Map();

    public readonly idl = {
        getConstant: (name: ProgramConstantName<IDL>): string => {
            return this.anchorProgram.idl.constants!.find(a => a.name == name)!.value;
        },
        getConstantAsPublicKey: (name: ProgramConstantName<IDL>): web3.PublicKey => {
            return new web3.PublicKey(this.idl.getConstant(name));
        },
        parseEvents: <EVENT extends keyof ProgramEvent<IDL>>(tx: web3.ParsedTransactionWithMeta) => {
            const events: {[k in EVENT]: ProgramEvent<IDL>[k][]} = {} as any;

            // parse event data from emit! macro
            const eventsFromEmitMacro = this.anchorEventParser.parseLogs(tx.meta?.logMessages ?? [], false);
            while (true) {
                const event = eventsFromEmitMacro.next().value;
                if (!event) break;
                const name = event.name as EVENT;
                (events[name] = events[name] ?? []).push(event as any);
            }

            // parse event data from emit_cpi! macro
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
    };

    public get programMethods() {
        return this.anchorProgram.methods;
    }

    public get programAccounts(): anchor.AccountNamespace<IDL> {
        return this.anchorProgram.account;
    }

    public createTransactionMessage<SIGNER extends string, EVENT extends keyof ProgramEvent<IDL>>(args: Omit<ConstructorParameters<typeof ProgramTransactionMessage<IDL, SIGNER, EVENT>>[0], 'program'>) {
        return new ProgramTransactionMessage<IDL, SIGNER, EVENT>({
            ...args,
            program: this,
        });
    }
}

export type ProgramEvent<IDL extends anchor.Idl> = anchor.IdlEvents<IDL>;
export type ProgramAccount<IDL extends anchor.Idl> = anchor.IdlAccounts<IDL>;
export type ProgramType<IDL extends anchor.Idl> = anchor.IdlTypes<IDL>;
export type ProgramConstantName<IDL extends anchor.Idl> = IDL['constants'] extends Array<infer U>
    ? U extends { name: infer N }
        ? N extends string
            ? N
            : never
        : never
    : never;
