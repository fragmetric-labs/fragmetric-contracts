import { TransactionSigner } from '@solana/kit';
import { Context } from './context';

export type AccountAddressResolver<P extends Context<any>> = (
  parent: P
) => Promise<string | null>;

export type TransactionSignerResolver = () => Promise<TransactionSigner>;

export type AccountAddressResolverVariant<P extends Context<any> = any> =
  | string
  | AccountAddressResolver<P>
  | TransactionSigner
  | TransactionSignerResolver;

export function transformAddressResolverVariant<P extends Context<any> = any>(
  resolver: AccountAddressResolverVariant<P>
): AccountAddressResolver<P> {
  return async (parent) => {
    const resolved =
      typeof resolver == 'function' ? await resolver(parent) : resolver;
    if (!resolved) return null;
    if (typeof resolved == 'object') return resolved.address;
    return resolved;
  };
}
