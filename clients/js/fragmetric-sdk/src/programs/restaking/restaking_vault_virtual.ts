import {
  Address,
  EncodedAccount,
  getAddressEncoder,
  getBytesEncoder,
  getProgramDerivedAddress,
} from '@solana/kit';
import {
  AccountContext,
  IterativeAccountContext,
  TokenAccountContext,
  TokenMintAccountContext,
} from '../../context';
import { RestakingFundAccountContext } from './fund';

export class VirtualVaultAccountContext extends AccountContext<
  RestakingFundAccountContext,
  EncodedAccount
> {
  async resolve(noCache = false): Promise<any> {
    return this.resolveAccount(noCache);
  }

  protected __decodeAccount(account: EncodedAccount): EncodedAccount {
    return account;
  }

  constructor(readonly parent: RestakingFundAccountContext) {
    super(parent, async (parent) => {
      const fundAddress = await parent.resolveAddress();
      const [vaultAddress] = await getProgramDerivedAddress({
        programAddress: parent.program.address as unknown as Address,
        seeds: [
          getBytesEncoder().encode(Buffer.from('virtual_vault')),
          getAddressEncoder().encode(
            'VVRTiZKXoPdME1ssmRdzowNG2VFVFG6Rmy9VViXaWa8' as Address
          ),
          getAddressEncoder().encode(fundAddress!),
        ],
      });
      return vaultAddress;
    });
  }

  readonly receiptTokenMint = new TokenMintAccountContext(
    this,
    'VVRTiZKXoPdME1ssmRdzowNG2VFVFG6Rmy9VViXaWa8'
  );

  readonly rewardTokens = new IterativeAccountContext(
    this,
    async (parent) => {
      const [self, fund] = await Promise.all([
        parent.resolveAddress(),
        parent.parent.resolveAccount(true),
      ]);
      const vaultConfig = fund?.data.restakingVaults.find(
        (v) => v.vault == self
      );
      if (!(self && vaultConfig)) return null;

      const rewardTokenMints = vaultConfig.compoundingRewardTokenMints
        .slice(0, vaultConfig.numCompoundingRewardTokens)
        .concat(
          vaultConfig.distributingRewardTokens
            .slice(0, vaultConfig.numDistributingRewardTokens)
            .map((r) => r.mint)
        );

      return Promise.all(
        rewardTokenMints.map((tokenMint) => {
          return TokenAccountContext.findAssociatedTokenAccountAddress({
            owner: self,
            mint: tokenMint,
          });
        })
      );
    },
    async (parent, address) => {
      return new TokenAccountContext(parent, address);
    }
  );
}
