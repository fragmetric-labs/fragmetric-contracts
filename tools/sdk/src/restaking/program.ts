import * as web3 from '@solana/web3.js';
import {Program} from "../program";

import idlFile from './program.idl.v0.3.3.json';
import type {Restaking} from './program.idl.v0.3.3';
export type RestakingIDL = Restaking;

export class RestakingProgram extends Program<RestakingIDL> {
    public static readonly ID = {
        mainnet: new web3.PublicKey('fragnAis7Bp6FTsMoa6YcH8UffhEw43Ph79qAiK3iF3'),
        devnet: new web3.PublicKey('frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ'),
        local: null,
    };

    public static readonly defaultClusterURL = {
        mainnet: 'https://api.mainnet-beta.solana.com',
        devnet: 'https://api.devnet.solana.com',
        local: 'http://0.0.0.0:8899',
    };

    public readonly cluster: keyof typeof RestakingProgram['ID'];

    constructor({ cluster = 'mainnet', connection, idl = <RestakingIDL>idlFile }: {
        cluster?: keyof typeof RestakingProgram['ID'],
        connection?: web3.Connection,
        idl?: RestakingIDL,
    }|undefined = {}) {
        const programID = RestakingProgram.ID[cluster] ?? new web3.PublicKey(idl.address);

        if (!connection) {
            connection = new web3.Connection(RestakingProgram.defaultClusterURL[cluster] ?? RestakingProgram.defaultClusterURL.local, {
                commitment: 'confirmed',
                disableRetryOnRateLimit: true,
            })
        }

        const usingDefaultClusterURL = Object.entries(RestakingProgram.defaultClusterURL).find(([_, url]) => url == connection!.rpcEndpoint);
        if (usingDefaultClusterURL && usingDefaultClusterURL[0] != cluster) {
            throw new Error("The provided connection URL does not match the specified cluster.");
        }

        super({ programID, idl, connection });
        this.cluster = cluster;
    }
}
