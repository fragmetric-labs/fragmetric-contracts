import {
  GetAccountInfoApi,
  GetEpochInfoApi,
  GetLatestBlockhashApi,
  GetMinimumBalanceForRentExemptionApi,
  GetMultipleAccountsApi,
  GetSignatureStatusesApi,
  GetSlotApi,
  GetTransactionApi,
  RequestAirdropApi,
  Rpc,
  RpcSubscriptions,
  SendTransactionApi,
  SimulateTransactionApi,
  SolanaRpcSubscriptionsApi,
} from '@solana/kit';
import { LiteSVMRuntimeConfig, createLiteSVMRuntime } from './litesvm.node';
import { SVMRuntimeConfig, createSVMRuntime } from './svm';

export interface Runtime {
  type: RuntimeType;
  cluster: RuntimeCluster;
  rpc: RuntimeRPC;
  rpcSubscriptions: RuntimeRPCSubscriptions | null;
}

export const runtimeClusters = [
  'mainnet' as 'mainnet',
  'devnet' as 'devnet',
  'testnet' as 'testnet',
  'local' as 'local',
];
export type RuntimeCluster = (typeof runtimeClusters)[number];

export type RuntimeRPCMethods = GetAccountInfoApi &
  GetMultipleAccountsApi &
  GetTransactionApi &
  SimulateTransactionApi &
  SendTransactionApi &
  GetMinimumBalanceForRentExemptionApi &
  GetLatestBlockhashApi &
  GetEpochInfoApi &
  GetSlotApi &
  GetSignatureStatusesApi;
export type RuntimeRPCOptionalMethods = RequestAirdropApi & {
  // ...
};

export type RuntimeRPC = Rpc<RuntimeRPCMethods> &
  Partial<Rpc<RuntimeRPCOptionalMethods>>;
export type RuntimeRPCSubscriptions =
  RpcSubscriptions<SolanaRpcSubscriptionsApi>;
export type RuntimeFactory<T extends RuntimeConfig> = (config: T) => Runtime;
export type RuntimeConfig = SVMRuntimeConfig | LiteSVMRuntimeConfig;
export type RuntimeType = NonNullable<RuntimeConfig['type']>;

export function createRuntime(config: RuntimeConfig): Runtime {
  switch (config.type) {
    case 'litesvm':
      return createLiteSVMRuntime(config);
    case 'svm':
    default:
      return createSVMRuntime(config);
  }
}
