import * as anchor from '@coral-xyz/anchor';
import {getLogger} from './logger';
import {Keychain} from './keychain';
import {WORKSPACE_PROGRAM_NAME} from "./types";
import {AnchorError} from "@coral-xyz/anchor";
import { bs58 } from '@coral-xyz/anchor/dist/cjs/utils/bytes/index';

const {logger, LOG_PAD_SMALL, LOG_PAD_LARGE } = getLogger('anchor');

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

        logger.info(`loaded program ${this.programName}:`.padEnd(LOG_PAD_LARGE), programAddress);
    }

    public async run(args: {
        instructions: (Promise<anchor.web3.TransactionInstruction> | anchor.web3.TransactionInstruction)[],
        signers?: anchor.web3.Signer[],
        signerNames?: KEYS[],
    }) {
        let txSig: string | null = null;
        try {
            // prepare instructions
            let {instructions, signers = [], signerNames = []} = args;
            const tx = new anchor.web3.Transaction()
                .add(
                    ...await Promise.all(instructions)
                );

            // set recent block hash
            const { blockhash, lastValidBlockHeight } = await this.connection.getLatestBlockhash();

            // sign with wallet to pay fee
            tx.recentBlockhash = blockhash;
            tx.feePayer = this.keychain.wallet.publicKey;
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
            tx.partialSign(...signers);

            // send transaction
            txSig = bs58.encode(tx.signature);
            await anchor.web3.sendAndConfirmRawTransaction(
                this.provider.connection,
                tx.serialize(),
                {
                    abortSignal: undefined,
                    lastValidBlockHeight,
                    blockhash,
                    signature: txSig,
                },
                {
                    skipPreflight: false,
                    commitment: 'confirmed',
                }
            );

            // get result and parse events and errors
            const txResult = await this.connection.getParsedTransaction(txSig, 'confirmed');
            logger.info(`transaction confirmed:`.padEnd(LOG_PAD_LARGE), txSig);
            return {
                event: this.eventParser.parseLogs(txResult.meta.logMessages, true),
                error: AnchorError.parse(txResult.meta.logMessages) ?? null,
            };
        } catch (err) {
            logger.error(`transaction failed`.padEnd(LOG_PAD_LARGE), txSig);
            throw err;
        }
    }

    public get wallet() {
        return this.keychain.wallet;
    }

    public get walletAddress() {
        return this.wallet.publicKey.toString();
    }

    public get connection() {
        return this.provider.connection;
    }

    public get programId(): anchor.web3.PublicKey {
        return this.program.programId;
    }

    public get methods(): anchor.MethodsNamespace<IDL> {
        return this.program.methods;
    }

    public get account(): anchor.AccountNamespace<IDL> {
        return this.program.account;
    }

    public async tryAirdrop(account: anchor.web3.PublicKey, sol = 1) {
        let [txSig, { blockhash, lastValidBlockHeight }] = await Promise.all([
            this.connection.requestAirdrop(
                account,
                sol * anchor.web3.LAMPORTS_PER_SOL
            ),
            this.connection.getLatestBlockhash(),
        ]);
        await this.connection.confirmTransaction({
            abortSignal: undefined,
            lastValidBlockHeight,
            blockhash,
            signature: txSig,
        });
        const balance = await this.connection.getBalance(account);
        logger.debug(`${account.toString()} SOL balance:`.padEnd(LOG_PAD_LARGE), balance.toLocaleString());
    }

    public get isMaybeLocalnet(): boolean {
        const endpoint = this.connection.rpcEndpoint;
        return endpoint.startsWith('http://') && endpoint.endsWith('8899');
    }

    public get isMaybeDevnet(): boolean {
        return this.connection.rpcEndpoint == anchor.web3.clusterApiUrl('devnet');
    }

    public get isMaybeMainnetBeta(): boolean {
        return this.connection.rpcEndpoint == anchor.web3.clusterApiUrl("mainnet-beta");
    }

    public getConstant(name: ExtractConstantNames<IDL>): string {
        return this.program.idl.constants.find(a => a.name == name).value;
    }

    public getConstantAsPublicKey(name: ExtractConstantNames<IDL>): anchor.web3.PublicKey {
        return new anchor.web3.PublicKey(this.getConstant(name));
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
}

type ExtractAccountNames<T extends anchor.Idl> = T['accounts'] extends Array<infer U>
    ? U extends { name: infer N }
        ? N extends string
            ? N
            : never
        : never
    : never;

type ExtractConstantNames<T extends anchor.Idl> = T['constants'] extends Array<infer U>
    ? U extends { name: infer N }
        ? N extends string
            ? N
            : never
        : never
    : never;
