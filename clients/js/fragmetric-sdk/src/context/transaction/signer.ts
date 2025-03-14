import { TransactionSignerResolver } from '../address';

export const HardwareWalletSymbol = Symbol.for('HardwareWalletSymbol');

export type HardwareWalletSignerResolver = TransactionSignerResolver & {
  [HardwareWalletSymbol]: true;
};

export function markAsHardwareWalletSignerResolver(
  resolver: TransactionSignerResolver
): HardwareWalletSignerResolver {
  Object.defineProperty(resolver, HardwareWalletSymbol, {
    value: true,
    enumerable: false,
    configurable: false,
  });
  return resolver as HardwareWalletSignerResolver;
}

export function isHardwareWalletSignerResolver(
  resolver: TransactionSignerResolver
): resolver is HardwareWalletSignerResolver {
  return resolver && (resolver as any)[HardwareWalletSymbol] == true;
}

export type HardwareWalletSignerResolverOptions = {
  // BIP derivation path; default is 44'/501'/0'/0'
  derivationPath?: string;
  connectionTimeoutSeconds?: number;
};
