import * as anchor from '@coral-xyz/anchor';
import {getLogger, LOG_PAD_LARGE, LOG_PAD_SMALL} from './logger';
import {KeypairLoader} from './keypair_loader';
import {Restaking} from '../../target/types/restaking';
import {WORKSPACE_PROGRAM_NAME} from "./types";
import {AnchorError} from "@coral-xyz/anchor";

const logger = getLogger('anchor');

export type AnchorPlaygroundConfig<IDL extends anchor.Idl, KEYS extends string> = {
    provider: anchor.Provider,
    idl: IDL,
    keypairs: KeypairLoader<KEYS>,
};

export class AnchorPlayground<IDL extends anchor.Idl, KEYS extends string> {
    public readonly programName: WORKSPACE_PROGRAM_NAME;
    public readonly keypairs: KeypairLoader<KEYS>;
    protected readonly provider: anchor.Provider;
    protected readonly program: anchor.Program<IDL>;
    protected readonly eventParser: anchor.EventParser;


    constructor(args: AnchorPlaygroundConfig<IDL, KEYS>) {
        let {idl, keypairs, provider} = args;
        this.programName = keypairs.programName;
        this.provider = provider;
        this.keypairs = keypairs;

        logger.notice(`initializing ${this.programName} playground`);

        logger.info(`connected to:`.padEnd(LOG_PAD_LARGE), provider.connection.rpcEndpoint);

        if (idl?.metadata?.name != this.programName) {
            throw new Error('program name and idl not matched');
        }

        const programAddress = keypairs.programKeypair.publicKey.toString();
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
        instructions: Promise<anchor.web3.TransactionInstruction>[],
        signers: anchor.web3.Signer[],
    }) {
        let txSig: string | null = null;
        try {
            let {instructions, signers} = args;
            txSig = await anchor.web3.sendAndConfirmTransaction(
                this.provider.connection,
                new anchor.web3.Transaction().add(
                    ...await Promise.all(instructions)
                ),
                signers,
            );

            const tx = await this.connection.getParsedTransaction(txSig, 'confirmed');
            logger.info(`run succeeded:`.padEnd(LOG_PAD_LARGE), txSig);
            return {
                event: this.eventParser.parseLogs(tx.meta.logMessages, true),
                error: AnchorError.parse(tx.meta.logMessages),
            };
        } catch (err) {
            logger.warn(`run failed`.padEnd(LOG_PAD_LARGE), txSig);
            return {
                event: null,
                error: err,
            };
        }
    }

    public get wallet() {
        return this.keypairs.wallet;
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
}

KeypairLoader.create({
    program: 'restaking',
    newKeypairDir: './keypairs/restaking',
    wallet: './keypairs/wallet.json',
    keypairs: {
        'PROGRAM': './keypairs/restaking/devnet_program_frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ.json',
        'FRAGSOL_MINT': './keypairs/restaking/fragsol_mint_FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo.json',
        'ADMIN': './keypairs/restaking/devnet_admin_fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP.json',
        'FUND_MANAGER': './keypairs/restaking/devnet_fund_manager_fragHx7xwt9tXZEHv2bNo3hGTtcHP9geWkqc2Ka6FeX.json',
        // 'FUND_MANAGER': `ledger://44'/501'/0'`,
    },
})
    .then(async (keypairs) => {
        const playground = new AnchorPlayground({
            provider: new anchor.AnchorProvider(
                new anchor.web3.Connection('http://127.0.0.1:8899'),
                new anchor.Wallet(keypairs.wallet),
            ),
            idl: require('../../target/idl/restaking.json') as Restaking,
            keypairs,
        });
        const result = await playground.run({
            instructions: [
                playground.methods
                    .adminInitializeFundAccounts()
                    .accounts({payer: playground.walletAddress})
                    .instruction(),
            ],
            signers: [playground.wallet, playground.keypairs.keypair('ADMIN')],
        });
        console.log(result);
    })
    .catch(err => {
        console.error(err);
    })
    .finally(() => {
        process.exit(0);
    });