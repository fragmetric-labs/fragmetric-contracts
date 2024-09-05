import {Restaking} from '../../target/types/restaking';
import {AnchorPlayground, AnchorPlaygroundConfig, getLogger, Keychain} from "../lib";
import * as anchor from "@coral-xyz/anchor";
import {getKeychain, KEYCHAIN_KEYS} from "./keychain";

export class RestakingPlayground extends AnchorPlayground<Restaking, KEYCHAIN_KEYS> {
    // The term "local" in this context doesn't necessarily refer to the localnet.
    // It can also be applied in devnet or mainnet environments while utilizing existing local keypairs.
    // and a different Anchor provider. This allows for flexibility in testing across various networks.
    public static local(provider?: anchor.AnchorProvider) {
        return getKeychain('local')
            .then(keychain => {
                return new RestakingPlayground({
                    keychain,
                    provider: new anchor.AnchorProvider(
                        provider?.connection ?? new anchor.web3.Connection('http://0.0.0.0:8899'),
                        new anchor.Wallet(keychain.wallet),
                    ),
                })
            });
    }

    public static devnet(args: Pick<AnchorPlaygroundConfig<Restaking, any>, 'provider'>) {
        return getKeychain('devnet')
            .then(keychain =>
                new RestakingPlayground({
                    keychain,
                    provider: new anchor.AnchorProvider(
                        new anchor.web3.Connection(anchor.web3.clusterApiUrl('devnet')),
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
                        new anchor.web3.Connection(anchor.web3.clusterApiUrl('mainnet-beta')),
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

    public async tryAirdropToMockAccounts() {
        await Promise.all(
            [
                this.tryAirdrop(this.keychain.wallet.publicKey),
                this.tryAirdrop(this.keychain.getPublicKey('MOCK_USER1')),
                this.tryAirdrop(this.keychain.getPublicKey('MOCK_USER2')),
                this.tryAirdrop(this.keychain.getPublicKey('MOCK_USER3')),
            ]
        );
    }

    public get pricingSourceAccounts(): anchor.web3.AccountMeta[] {
        if (this.isMaybeMainnetBeta) {
            return [
                {
                    pubkey: new anchor.web3.PublicKey(this.getConstant('mainnetBsolStakePoolAddress')),
                    isSigner: false,
                    isWritable: false,
                },
                {
                    pubkey: new anchor.web3.PublicKey(this.getConstant('mainnetJitosolStakePoolAddress')),
                    isSigner: false,
                    isWritable: false,
                },
                {
                    pubkey: new anchor.web3.PublicKey(this.getConstant('mainnetMsolStakePoolAddress')),
                    isSigner: false,
                    isWritable: false,
                },
            ];
        } else if (this.isMaybeDevnet) {
            return [
                {
                    pubkey: new anchor.web3.PublicKey(this.getConstant('devnetBsolStakePoolAddress')),
                    isSigner: false,
                    isWritable: false,
                },
                {
                    pubkey: new anchor.web3.PublicKey(this.getConstant('devnetMsolStakePoolAddress')),
                    isSigner: false,
                    isWritable: false,
                },
            ];
        } else {
            // would be cloned to local-test-validator
            return [
                {
                    pubkey: new anchor.web3.PublicKey(this.getConstant('devnetBsolStakePoolAddress')),
                    isSigner: false,
                    isWritable: false,
                },
                {
                    pubkey: new anchor.web3.PublicKey(this.getConstant('mainnetMsolStakePoolAddress')),
                    isSigner: false,
                    isWritable: false,
                },
                {
                    pubkey: new anchor.web3.PublicKey(this.getConstant('mainnetJitosolStakePoolAddress')),
                    isSigner: false,
                    isWritable: false,
                },
            ];
        }
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