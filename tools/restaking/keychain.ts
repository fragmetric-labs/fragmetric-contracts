import {Keychain, KeychainConfig} from "../lib";

const keypairs = {
    'PROGRAM': './keypairs/restaking/shared_local_program_4qEHCzsLFUnw8jmhmRSmAK5VhZVoSD1iVqukAf92yHi5.json',
    'FRAGSOL_MINT': './keypairs/restaking/shared_local_fragsol_mint_Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD.json',
    'ADMIN': './keypairs/restaking/shared_local_admin_9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL.json',
    'FUND_MANAGER': './keypairs/restaking/shared_local_fund_manager_5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx.json',
};

const local: KeychainConfig<keyof (typeof keypairs)> = {
    program: 'restaking',
    newKeypairDir: './keypairs/restaking',
    wallet: './keypairs/shared_wallet_GiDkDCZjVC8Nk1Fd457qGSV2g3MQX62n7cV5CvgFyGfF.json',
    keypairs: keypairs,
};

const devnet: KeychainConfig<keyof (typeof keypairs)> = {
    ...local,
    wallet: './keypairs/wallet.json',
    keypairs: {
        ...keypairs,
        'PROGRAM': './keypairs/restaking/devnet_program_frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ.json',
        'FRAGSOL_MINT': './keypairs/restaking/fragsol_mint_FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo.json',
        'ADMIN': './keypairs/restaking/devnet_admin_fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP.json',
        'FUND_MANAGER': `ledger://44'/501'/0'`,
    },
};

const mainnet: KeychainConfig<keyof (typeof keypairs)> = {
    ...local,
    wallet: './keypairs/wallet.json',
    keypairs: {
        ...keypairs,
        'PROGRAM': './keypairs/restaking/mainnet_program_fragnAis7Bp6FTsMoa6YcH8UffhEw43Ph79qAiK3iF3.json',
        'FRAGSOL_MINT': './keypairs/restaking/fragsol_mint_FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo.json',
        'ADMIN': './keypairs/restaking/mainnet_admin_fragSkuEpEmdoj9Bcyawk9rBdsChcVJLWHfj9JX1Gby.json',
        'FUND_MANAGER': `ledger://44'/501'/0'`,
    },
};

export type KEYCHAIN_ENV = 'local'|'devnet'|'mainnet';
export type KEYCHAIN_KEYS = keyof (typeof keypairs);

export function getKeychain(env: KEYCHAIN_ENV) {
    const config = (() => {
        if (env == 'mainnet') {
            return mainnet;
        } else if (env == 'devnet') {
            return devnet;
        } else {
            return local;
        }
    })();
    return Keychain.create(config);
}