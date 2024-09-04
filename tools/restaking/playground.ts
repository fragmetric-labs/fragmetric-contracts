import {Restaking} from '../../target/types/restaking';
import {AnchorPlayground, AnchorPlaygroundConfig, getLogger, Keychain} from "../lib";
import * as anchor from "@coral-xyz/anchor";
import {getKeychain, KEYCHAIN_KEYS} from "./keychain";


export class RestakingPlayground extends AnchorPlayground<Restaking, KEYCHAIN_KEYS> {
    public static local() {
        return getKeychain('local')
            .then(keychain =>
                new RestakingPlayground({
                    keychain,
                    provider: new anchor.AnchorProvider(
                        new anchor.web3.Connection('http://127.0.0.1:8899'),
                        new anchor.Wallet(keychain.wallet),
                    ),
                }),
            );
    }

    public static env() {
        return getKeychain('local')
            .then(keychain => {
                const provider = anchor.AnchorProvider.env();
                if (provider.wallet.publicKey.toString() != keychain.wallet.publicKey.toString()) {
                    // TODO: ... set wallet here.. and then rewrite tests while extending this.
                    // and then create a runbook
                    console.log('SHOULD OVERRDIE');
                    Object.assign(provider, {wallet: new anchor.Wallet(keychain.wallet)});
                }
                return new RestakingPlayground({
                    keychain,
                    provider,
                })
            });
    }

    public static devnet(args: Pick<AnchorPlaygroundConfig<Restaking, any>, 'provider'>) {
        return getKeychain('devnet')
            .then(keychain =>
                new RestakingPlayground({
                    keychain,
                    provider: new anchor.AnchorProvider(
                        new anchor.web3.Connection('https://api.devnet.solana.com'),
                        new anchor.Wallet(keychain.wallet),
                    ),
                }),
            );
    }

    public static mainnet(args: Pick<AnchorPlaygroundConfig<Restaking, any>, 'provider'>) {
        return getKeychain('mainnet')
            .then(keychain =>
                new RestakingPlayground({
                    keychain,
                    provider: new anchor.AnchorProvider(
                        new anchor.web3.Connection('https://api.mainnet-beta.solana.com'),
                        new anchor.Wallet(keychain.wallet),
                    ),
                }),
            );
    }

    private constructor(args: Pick<AnchorPlaygroundConfig<Restaking, any>, 'provider'|'keychain'>) {
        super({
            provider: args.provider,
            keychain: args.keychain,
            idl: require('../../target/idl/restaking.json') as Restaking,
        });
    }
}

// (async function run() {
//     const { logger } = getLogger('restaking');
//     try {
//         const local = await RestakingPlayground.env();
//         const result = await local.run({
//             instructions: [
//                 local.methods
//                     .fundManagerSettleReward(0, 0, new anchor.BN(1000))
//                     .accounts({
//                         rewardTokenMint: local.keychain.wallet.publicKey,
//                         rewardTokenProgram: local.keychain.wallet.publicKey,
//                     })
//                     .instruction(),
//             ],
//             signerNames: ['FUND_MANAGER'],
//         });
//         logger.info('ok', result);
//         process.exit(0);
//     } catch (err) {
//         logger.error('err', err);
//         process.exit(0);
//     }
// })()