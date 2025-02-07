import {Keychain, KeychainConfig} from '../lib';

const keypairs = {
    'PROGRAM': './keypairs/restaking/shared_local_program_4qEHCzsLFUnw8jmhmRSmAK5VhZVoSD1iVqukAf92yHi5.json',
    'FRAGSOL_MINT': './keypairs/restaking/shared_local_fragsol_mint_Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD.json',
    'FRAGJTO_MINT': './keypairs/restaking/shared_local_fragjto_mint_bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v.json',
    'FRAGSOL_NORMALIZED_TOKEN_MINT': './keypairs/restaking/shared_local_nsol_mint_4noNmx2RpxK4zdr68Fq1CYM5VhN4yjgGZEFyuB7t2pBX.json',
    'FRAGSOL_WRAPPED_TOKEN_MINT': './keypairs/restaking/shared_local_wfragsol_mint_h7veGmqGWmFPe2vbsrKVNARvucfZ2WKCXUvJBmbJ86Q.json',
    'FRAGJTO_WRAPPED_TOKEN_MINT': './keypairs/restaking/shared_local_wfragjto_mint_EAvS1wFjAccNpDYbAkW2dwUDEiC7BMvWzwUj2tjRUkHA.json',
    'ADMIN': './keypairs/restaking/shared_local_admin_9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL.json',
    'FUND_MANAGER': './keypairs/restaking/shared_local_fund_manager_5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx.json',

    // fixtures for local test
    'ALL_MINT_AUTHORITY': './tests/mocks/all_mint_authority.json',
    'MOCK_USER1': './tests/mocks/user1.json',
    'MOCK_USER2': './tests/mocks/user2.json',
    'MOCK_USER3': './tests/mocks/user3.json',
    'MOCK_USER4': './tests/mocks/user4.json',
    'MOCK_USER5': './tests/mocks/user5.json',
    'MOCK_USER6': './tests/mocks/user6.json',
    'MOCK_USER7': './tests/mocks/user7.json',
    'MOCK_USER8': './tests/mocks/user8.json',
    'MOCK_USER9': './tests/mocks/user9.json',
    'MOCK_USER10': './tests/mocks/user10.json',
};

const local: KeychainConfig<keyof (typeof keypairs)> = {
    program: 'restaking',
    cluster: "local",
    newKeypairDir: './keypairs/restaking',
    wallet: './keypairs/shared_wallet_GiDkDCZjVC8Nk1Fd457qGSV2g3MQX62n7cV5CvgFyGfF.json',
    keypairs: keypairs,
};

const devnet: KeychainConfig<keyof (typeof keypairs)> = {
    ...local,
    cluster: "devnet",
    wallet: './keypairs/wallet.json',
    keypairs: {
        ...keypairs,
        'PROGRAM': './keypairs/restaking/devnet_program_frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ.json',
        'FRAGSOL_MINT': './keypairs/restaking/fragsol_mint_FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo.json',
        'FRAGJTO_MINT': './keypairs/restaking/fragjto_mint_FRAGJ157KSDfGvBJtCSrsTWUqFnZhrw4aC8N8LqHuoos.json',
        'FRAGSOL_NORMALIZED_TOKEN_MINT': './keypairs/restaking/nsol_mint_nSoLnkrvh2aY792pgCNT6hzx84vYtkviRzxvhf3ws8e.json',
        'ADMIN': './keypairs/restaking/devnet_admin_fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP.json',
        'FUND_MANAGER': `ledger://44'/501'/0'`,
    },
};

const mainnet: KeychainConfig<keyof (typeof keypairs)> = {
    ...local,
    cluster: "mainnet",
    wallet: './keypairs/wallet.json',
    keypairs: {
        ...keypairs,
        'PROGRAM': './keypairs/restaking/mainnet_program_fragnAis7Bp6FTsMoa6YcH8UffhEw43Ph79qAiK3iF3.json',
        'FRAGSOL_MINT': './keypairs/restaking/fragsol_mint_FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo.json',
        'FRAGJTO_MINT': './keypairs/restaking/fragjto_mint_FRAGJ157KSDfGvBJtCSrsTWUqFnZhrw4aC8N8LqHuoos.json',
        'FRAGSOL_NORMALIZED_TOKEN_MINT': './keypairs/restaking/nsol_mint_nSoLnkrvh2aY792pgCNT6hzx84vYtkviRzxvhf3ws8e.json',
        'ADMIN': './keypairs/restaking/mainnet_admin_fragSkuEpEmdoj9Bcyawk9rBdsChcVJLWHfj9JX1Gby.json',
        'FUND_MANAGER': `ledger://44'/501'/0'`,
    },
};

const envs = {local, devnet, mainnet};
export type KEYCHAIN_ENV = keyof (typeof envs);
export type KEYCHAIN_KEYS = keyof (typeof keypairs);

export function getKeychain(env: KEYCHAIN_ENV) {
    const config = envs[env];
    if (!config) {
        throw new Error(`invalid keychain env: ${env}`);
    }
    return Keychain.create(config);
}