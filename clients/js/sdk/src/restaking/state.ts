import * as web3 from '@solana/web3.js';
import BN from 'bn.js';

export type RestakingFundSupportedAsset = {
    isNativeSOL: boolean;
    tokenMint: web3.PublicKey | null;
    tokenProgram: web3.PublicKey | null;
    decimals: number;
    oneTokenAsSOL: BN;
    oneTokenAsReceiptToken: BN; // can estimate minting receipt token using it
    depositable: boolean;
    withdrawable: boolean;
    accumulatedDepositCapacityAmount: BN | null; // null means there is no cap limit
    accumulatedDepositAmount: BN;
}
