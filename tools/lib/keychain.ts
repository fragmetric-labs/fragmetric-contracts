import fs from "fs";
import path from "path";
import readline from 'readline';
import {exec} from "node:child_process";
import * as web3 from "@solana/web3.js";
import {KeychainLedgerAdapter} from "./keychain_ledger_adapter";
import {WORKSPACE_PROGRAM_NAME} from "./types";
import {getLogger} from "./logger";
import {Buffer} from "buffer";
import {Key} from "node:readline";
import {askOnce} from "./repl";

const {logger, LOG_PAD_SMALL, LOG_PAD_LARGE} = getLogger('keychain');

type AskYesNo = (question: string) => Promise<boolean>;

function defaultAskYesNo(question: string) {
    return askOnce(`${question} (y/n)`)
        .then(answer => answer == 'y' || answer == 'yes');
}

const PROGRAM_KEYPAIR_NAME = 'PROGRAM';
const LEDGER_PATH_PREFIX = 'ledger://';

export type KeychainConfig<KEYS extends string> = {
    program: WORKSPACE_PROGRAM_NAME,
    // default is 'anchor build -p target-program'
    buildCommand?: string;
    // give existing local keypair path or instance, or give null to generate new one.
    wallet: web3.Keypair | string | null,
    // give existing local keypair file 'file://secret-file-path' or 'ledger://BIP32-path' to use Ledger, any undefined keypair will be generated newly.
    // reserved 'PROGRAM' keypair is used to build and deploy program.
    keypairs: { [name in KEYS]: 'ledger://BIP32-path' | string | null };
    newKeypairDir: string;
    askYesNo?: AskYesNo;
};

type KeypairMap = {
    local: Map<string, web3.Keypair>;
    ledger: Map<string, { bip32Path: string, publicKey: web3.PublicKey }>;
};

export class Keychain<KEYS extends string> {
    public get programKeypair(): web3.Keypair {
        return this.keypairs.local.get(PROGRAM_KEYPAIR_NAME) ?? null;
    }

    public getKeypair(name: KEYS): web3.Keypair | null {
        return this.keypairs.local.get(name);
    }

    public getPublicKey(name: KEYS): web3.PublicKey | null {
        return this.keypairs.local.get(name)?.publicKey ??
            this.keypairs.ledger.get(name)?.publicKey ??
            null;
    }

    public async signTransaction(name: KEYS, tx: web3.Transaction): Promise<{ local?: web3.Keypair, ledger?: web3.SignaturePubkeyPair }> {
        try {
            if (this.keypairs.local.has(name)) {
                const keypair = this.keypairs.local.get(name);
                return {
                    local: keypair,
                };

            } else if (this.keypairs.ledger.has(name)) {
                const keypair = this.keypairs.ledger.get(name);
                const txBuffer = tx.serializeMessage();

                return {
                    ledger: {
                        publicKey: keypair.publicKey,
                        signature: await Keychain.ledgerAdapter.getSignature(
                            keypair.bip32Path,
                            txBuffer,
                        ),
                    },
                };
            } else {
                throw new Error(`keypair not found in keychain`);
            }
        } catch (err) {
            logger.error(name.padEnd(LOG_PAD_LARGE), `signing failed`);
            throw err;
        }
    }

    public static readKeypairSecretFile(path: string): web3.Keypair {
        try {
            return web3.Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(path).toString())));
        } catch (e) {
            logger.error(`keypair file not exists:`.padEnd(LOG_PAD_SMALL), path);
            throw e;
        }
    }

    public static writeKeypairSecretFile(path: string, keyPair: web3.Keypair) {
        logger.notice(`writing ${path}`);
        fs.writeFileSync(path, JSON.stringify(Buffer.from(keyPair.secretKey.buffer).toJSON().data));
    }

    public static async create<KEYS extends string>(args: KeychainConfig<KEYS>): Promise<Keychain<KEYS>> {
        let {wallet: walletArg, newKeypairDir = './', program} = args;
        let wallet: web3.Keypair;
        if (walletArg === undefined) {
            wallet = web3.Keypair.generate();
            logger.info(`generated local wallet`);
            const saveFilePath = path.join(newKeypairDir, `wallet_${wallet.publicKey.toString()}.json`);
            Keychain.writeKeypairSecretFile(saveFilePath, wallet);
        } else if (typeof walletArg == 'string') {
            wallet = Keychain.readKeypairSecretFile(walletArg);
            logger.debug(`loaded local wallet`);
        } else {
            wallet = walletArg;
            logger.debug(`using given wallet instance`);
        }
        logger.info(`WALLET`.padEnd(LOG_PAD_SMALL), wallet.publicKey.toString());

        const keypairs = await Keychain.initializeKeypairs(args);
        return new Keychain(args.program, wallet, keypairs);
    }

    static ledgerAdapter: KeychainLedgerAdapter | null = null;

    private constructor(
        public readonly programName: WORKSPACE_PROGRAM_NAME,
        public readonly wallet: web3.Keypair,
        private readonly keypairs: KeypairMap,
    ) {
    }

    private static async initializeKeypairs<KEYS extends string>(args: KeychainConfig<KEYS>): Promise<KeypairMap> {
        let {program, keypairs, newKeypairDir = './', buildCommand, askYesNo = defaultAskYesNo} = args;
        if (!buildCommand) {
            buildCommand = `anchor build -p ${program}`;
        }

        const newLocalKeypairKEYS = new Set<string>();
        const keypairMap: KeypairMap = {
            local: new Map(),
            ledger: new Map(),
        };

        if (!Keychain.ledgerAdapter && Object.values(keypairs).some(k => (k as string | null)?.startsWith(LEDGER_PATH_PREFIX))) {
            Keychain.ledgerAdapter = await KeychainLedgerAdapter.create();
        }

        logger.notice(`loading ${program} program keypairs`);
        for (let [k, v] of Object.entries(keypairs)) {
            let keypairName = k.toUpperCase().trim().replace(' ', '_');
            let keypairSecretPath = v as string | null;
            if (keypairSecretPath) {
                if (keypairSecretPath.startsWith(LEDGER_PATH_PREFIX)) {
                    const bip32Path = keypairSecretPath.substring(LEDGER_PATH_PREFIX.length);
                    keypairMap.ledger.set(keypairName, {
                        bip32Path,
                        publicKey: await Keychain.ledgerAdapter.getPublicKey(bip32Path),
                    });
                } else {
                    keypairMap.local.set(keypairName, Keychain.readKeypairSecretFile(keypairSecretPath));
                }
            } else {
                keypairMap.local.set(keypairName, web3.Keypair.generate());
                newLocalKeypairKEYS.add(keypairName);
            }
        }
        if (!keypairMap.local.has(PROGRAM_KEYPAIR_NAME)) {
            logger.error(`'${PROGRAM_KEYPAIR_NAME}' keypair must be loaded from local file, give null to generate new one.`)
            throw new Error('local program keypair not found');
        }

        logger.debug(`ledger keypairs (${keypairMap.ledger.size}):`.padEnd(LOG_PAD_SMALL), Array.from(keypairMap.ledger.keys()).join(', '));
        logger.debug(`local keypairs (${keypairMap.local.size}):`.padEnd(LOG_PAD_SMALL), Array.from(keypairMap.local.keys()).join(', '));
        if (newLocalKeypairKEYS.size) {
            logger.info(`generated local keypairs (${newLocalKeypairKEYS.size}):`, Array.from(newLocalKeypairKEYS.values()).join(', '));
        }

        logger.notice(`applying keypairs to ${program} program workspace:`);
        await Keychain.applyKeypairsToWorkspace(program, keypairMap, newLocalKeypairKEYS.size > 0, buildCommand, askYesNo);

        logger.notice(`loaded ${program} program keypairs' pubkey:`)
        for (const [name, keypair] of keypairMap.local.entries()) {
            logger.info(`${name}`.padEnd(LOG_PAD_SMALL), keypair.publicKey.toString());
            if (newLocalKeypairKEYS.has(name)) {
                const saveFilePath = path.join(newKeypairDir, `local_${name.toLowerCase()}_${keypair.publicKey.toString()}.json`);
                Keychain.writeKeypairSecretFile(saveFilePath, keypair);
            }
        }
        for (const [name, keypair] of keypairMap.ledger.entries()) {
            logger.info(`${name}`.padEnd(LOG_PAD_SMALL), `${keypair.publicKey.toString()} (ledger: ${keypair.bip32Path})`);
        }

        return keypairMap;
    }

    private static async applyKeypairsToWorkspace(program: WORKSPACE_PROGRAM_NAME, keypairMap: KeypairMap, hasNewLocalKeypair: boolean, buildCommand: string, askYesNo: AskYesNo | false) {
        if (askYesNo && hasNewLocalKeypair) {
            if (!await askYesNo(`[?] Applying new keypairs will update workspace code and file, continue?`)) {
                logger.debug(`exit without workspace updates...`);
                throw new Error('workspace update rejected');
            }
        }

        // update program public key in source code
        const keypairKEYS = [...keypairMap.local.keys(), ...keypairMap.ledger.keys()];
        const programSrcDir = path.join(__dirname, '../../programs', program, 'src');
        let fileUpdated: string[] = [];
        for (const fileName of ['constants/local.rs']) {
            const filePath = path.join(programSrcDir, fileName);
            let fileSource = fs.readFileSync(filePath).toString();
            logger.debug(`checking ${filePath}`);

            let updated = 0;
            for (const keypairName of keypairKEYS) {
                const matches = fileSource.match(new RegExp(`\/\\*local:${keypairName}\\*\/"([^"]+)"\/\\*\\*\/`, 'mg'));

                if (matches) {
                    const publicKey = keypairMap.local.get(keypairName)?.publicKey || keypairMap.ledger.get(keypairName).publicKey;
                    const target = `/*local:${keypairName}*/"${publicKey.toString()}"/**/`;
                    for (const match of matches) {
                        if (match != target) {
                            updated++;
                            fileSource = fileSource.replace(match, target);
                            logger.info(`replaced a line starting with`.padEnd(LOG_PAD_SMALL), `/*local:${keypairName}*/...`);
                        }
                    }
                }
            }
            if (updated > 0) {
                fs.writeFileSync(filePath, fileSource);
                fileUpdated.push(filePath);
            }
        }

        // update program build keypair
        const targetDeployDir = path.join(__dirname, '../../target/deploy');
        fs.mkdirSync(targetDeployDir, {recursive: true});

        const programKeypairName = `${program}-keypair.json`;
        const programKeyPairPath = path.join(targetDeployDir, programKeypairName);
        const programKeyPair = keypairMap.local.get(PROGRAM_KEYPAIR_NAME);
        logger.debug(`checking ${programKeyPairPath}`);

        if (!fs.existsSync(programKeyPairPath) || Keychain.readKeypairSecretFile(programKeyPairPath)?.secretKey.toString() != programKeyPair.secretKey.toString()) {
            Keychain.writeKeypairSecretFile(programKeyPairPath, programKeyPair);
            fileUpdated.push(programKeyPairPath);
            logger.info(`replaced ${programKeypairName}`);
        }

        if (fileUpdated.length > 0) {
            logger.notice(`rebuild ${program} program workspace:`);
            try {
                if (!askYesNo || await askYesNo(`[?] running "${buildCommand}" to reflect workspace changes, continue?`)) {
                    await new Promise<void>((resolve, reject) => {
                        const buildProcess = exec(buildCommand);
                        buildProcess.stdout.pipe(process.stdout);
                        buildProcess.stderr.pipe(process.stderr);
                        buildProcess.on('exit', (exitCode) => {
                            if (exitCode == 0) {
                                resolve();
                            } else {
                                reject(new Error('failed to run build command'));
                            }
                        });
                    })
                } else {
                    throw new Error('workspace updated but rebuild rejected');
                }
            } catch (err) {
                fs.unlinkSync(programKeyPairPath); // remove build keypair to re-trigger build later
                throw err;
            }
        }
    }
}
