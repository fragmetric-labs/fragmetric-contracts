import fs from "fs";
import path from "path";
import readline from 'readline';
import * as web3 from "@solana/web3.js";
import {KeypairLedgerAdapter} from "./keypair_ledger_adapter";
import {WORKSPACE_PROGRAM_NAME} from "./types";
import {getLogger, LOG_PAD_SMALL} from "./logger";

const logger = getLogger('keypair');

type AskYesNo = (question: string) => Promise<boolean>;

function defaultAskYesNo(question: string) {
    const rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout
    });
    return new Promise<boolean>((resolve) => {
        rl.question(`${question} (y/n): `, (answer: string) => {
            const normalizedAnswer = answer.trim().toLowerCase();
            resolve(normalizedAnswer === 'y' || normalizedAnswer === 'yes');
            rl.close();
        });
    });
}

const PROGRAM_KEYPAIR_NAME = 'PROGRAM';
type PROGRAM_KEYPAIR_NAME_TYPE = 'PROGRAM';
const LEDGER_PATH_PREFIX = 'ledger://';

type KeypairLoaderConfig<KEYS extends string> = {
    program: WORKSPACE_PROGRAM_NAME,
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

export class KeypairLoader<KEYS extends string> {
    public get programKeypair(): web3.Keypair {
        return this.keypairs.local.get(PROGRAM_KEYPAIR_NAME) ?? null;
    }

    public keypair(name: Exclude<KEYS, PROGRAM_KEYPAIR_NAME_TYPE>): web3.Keypair | null {
        return this.keypairs.local.get(name);
    }

    public publicKey(name: Exclude<KEYS, PROGRAM_KEYPAIR_NAME_TYPE>): web3.PublicKey | null {
        return this.keypairs.local.get(name)?.publicKey ??
            this.keypairs.ledger.get(name)?.publicKey ??
            null;
    }

    // public sign(name: string): web3.Keypair | null {
    //     return this.programKeyPairMaps.get(program)?.ledger.get(keypairName);
    // }


    public static readKeypairSecretFile(path: string): web3.Keypair {
        try {
            return web3.Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(path).toString())));
        } catch (e) {
            logger.error(`keypair file not exists:`.padEnd(LOG_PAD_SMALL), path);
            throw e;
        }
    }

    public static writeKeypairSecretFile(path: string, keyPair: web3.Keypair) {
        logger.notice(`>> ${path}`);
        fs.writeFileSync(path, JSON.stringify(Buffer.from(keyPair.secretKey.buffer).toJSON().data));
    }

    public static async create<KEYS extends string>(args: KeypairLoaderConfig<KEYS>): Promise<KeypairLoader<KEYS>> {
        let {wallet: walletArg, newKeypairDir = './', program} = args;
        let wallet: web3.Keypair;
        if (walletArg === undefined) {
            wallet = web3.Keypair.generate();
            logger.info(`generated local wallet`);
            const saveFilePath = path.join(newKeypairDir, `wallet_${wallet.publicKey.toString()}.json`);
            KeypairLoader.writeKeypairSecretFile(saveFilePath, wallet);
        } else if (typeof walletArg == 'string') {
            wallet = KeypairLoader.readKeypairSecretFile(walletArg);
            logger.debug(`loaded local wallet`);
        } else {
            wallet = walletArg;
            logger.debug(`using given wallet instance`);
        }
        logger.info(`WALLET`.padEnd(LOG_PAD_SMALL), wallet.publicKey.toString());

        const keypairs = await KeypairLoader.initializeKeypairs(args);
        return new KeypairLoader(args.program, wallet, keypairs);
    }

    private static ledgerAdapter: KeypairLedgerAdapter | null = null;

    private constructor(
        public readonly programName: WORKSPACE_PROGRAM_NAME,
        public readonly wallet: web3.Keypair,
        private readonly keypairs: KeypairMap,
    ) {
    }

    private static async initializeKeypairs<KEYS extends string>(args: KeypairLoaderConfig<KEYS>): Promise<KeypairMap> {
        let {program, keypairs, newKeypairDir = './', askYesNo = defaultAskYesNo} = args;

        const newLocalKeypairKEYS = new Set<string>();
        const keypairMap: KeypairMap = {
            local: new Map(),
            ledger: new Map(),
        };

        if (!KeypairLoader.ledgerAdapter && Object.values(keypairs).some(k => (k as string | null)?.startsWith(LEDGER_PATH_PREFIX))) {
            KeypairLoader.ledgerAdapter = await KeypairLedgerAdapter.create();
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
                        publicKey: await KeypairLoader.ledgerAdapter.getPublicKey(bip32Path),
                    });
                } else {
                    keypairMap.local.set(keypairName, KeypairLoader.readKeypairSecretFile(keypairSecretPath));
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

        logger.notice(`applying keypairs to ${program} program source code and build dir:`);
        await KeypairLoader.applyKeypairsToWorkspace(program, keypairMap, newLocalKeypairKEYS.size > 0 ? askYesNo : null);

        logger.notice(`loaded ${program} program keypairs' pubkey:`)
        for (const [name, keypair] of keypairMap.local.entries()) {
            logger.info(`${name}`.padEnd(LOG_PAD_SMALL), keypair.publicKey.toString());
            if (newLocalKeypairKEYS.has(name)) {
                const saveFilePath = path.join(newKeypairDir, `local_${name.toLowerCase()}_${keypair.publicKey.toString()}.json`);
                KeypairLoader.writeKeypairSecretFile(saveFilePath, keypair);
            }
        }
        for (const [name, keypair] of keypairMap.ledger.entries()) {
            logger.info(`${name}`.padEnd(LOG_PAD_SMALL), `${keypair.publicKey.toString()} (ledger: ${keypair.bip32Path})`);
        }

        return keypairMap;
    }

    private static async applyKeypairsToWorkspace(program: WORKSPACE_PROGRAM_NAME, keypairMap: KeypairMap, askYesNo: AskYesNo | false) {
        if (askYesNo) { // or new ledger public keys
            if (!await askYesNo(`[?] Applying newly generated keypairs will replace existing code and file, do you want to continue?`)) {
                logger.debug(`exit without updates...`);
                throw new Error('keypair loading canceled');
            }
        }

        // update program public key in source code
        const keypairKEYS = [...keypairMap.local.keys(), ...keypairMap.ledger.keys()];
        const programSrcDir = path.join(__dirname, '../../programs', program, 'src');
        for (const fileName of ['lib.rs', 'constants.rs']) {
            const filePath = path.join(programSrcDir, fileName);
            let fileSource = fs.readFileSync(filePath).toString();
            logger.debug(`checking ${filePath}`);

            let fileUpdated = 0;
            for (const keypairName of keypairKEYS) {
                const matches = fileSource.match(new RegExp(`\/\\*local:${keypairName}\\*\/"([^"]+)"\/\\*\\*\/`, 'mg'));

                if (matches) {
                    const publicKey = keypairMap.local.get(keypairName)?.publicKey || keypairMap.ledger.get(keypairName).publicKey;
                    const target = `/*local:${keypairName}*/"${publicKey.toString()}"/**/`;
                    for (const match of matches) {
                        if (match != target) {
                            fileUpdated++;
                            fileSource = fileSource.replace(match, target);
                            logger.info(`replaced a line starting with`.padEnd(LOG_PAD_SMALL), `/*local:${keypairName}*/...`);
                        }
                    }
                }
            }
            if (fileUpdated > 0) {
                fs.writeFileSync(filePath, fileSource);
            }
        }

        // update program build keypair
        const targetDeployDir = path.join(__dirname, '../../target/deploy');
        fs.mkdirSync(targetDeployDir, {recursive: true});

        const programKeypairName = `${program}-keypair.json`;
        const programKeyPairPath = path.join(targetDeployDir, programKeypairName);
        const programKeyPair = keypairMap.local.get(PROGRAM_KEYPAIR_NAME);
        logger.debug(`checking ${programKeyPairPath}`);

        if (!fs.existsSync(programKeyPairPath) || KeypairLoader.readKeypairSecretFile(programKeyPairPath)?.secretKey.toString() != programKeyPair.secretKey.toString()) {
            KeypairLoader.writeKeypairSecretFile(programKeyPairPath, programKeyPair);
            logger.info(`replaced ${programKeypairName}`);
        }
    }
}
