import * as web3 from '@solana/web3.js';
import BN from 'bn.js';

export type RestakingFundSupportedAsset = {
    isNativeSOL: boolean;
    mint: web3.PublicKey | null;
    program: web3.PublicKey | null;
    decimals: number;
    oneTokenAsSOL: BN;
    oneTokenAsReceiptToken: BN; // can use this to estimate receipt token minting amount on deposit
    depositable: boolean;
    withdrawable: boolean;
    accumulatedDepositCapacityAmount: BN | null; // null means there is no cap limit
    accumulatedDepositAmount: BN;
};

export type RestakingFundNormalizedToken = {
    mint: web3.PublicKey;
    program: web3.PublicKey;
    decimals: number;
    oneTokenAsSOL: BN;
    updatedSlot: BN;
};

export type RestakingFundReceiptToken = {
    mint: web3.PublicKey;
    program: web3.PublicKey;
    decimals: number;
    supplyAmount: BN;
    oneTokenAsSOL: BN;
    updatedSlot: BN;
    withdrawalFeePercent: number;
    wrappedTokenMint: web3.PublicKey | null;
};
