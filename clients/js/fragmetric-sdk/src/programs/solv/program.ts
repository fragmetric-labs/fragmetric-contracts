import { address } from '@solana/kit';
import {
  AccountAddressResolverVariant,
  AccountContext,
  ProgramContext,
} from '../../context';
import { SolvVaultAccountContext } from './vault';

export class SolvBTCVaultProgram extends ProgramContext {
  async resolve(noCache = false) {
    await Promise.all(
      Object.values(this)
        .filter((value) => value instanceof AccountContext)
        .map((value) => value.resolveAccountTree(noCache))
    );
  }

  static readonly addresses = {
    mainnet: 'FSoLvf9dv17a4DzMGYKxqFnDGj9EiXRW5wKrwQ39UDH',
    devnet: 'FsoLVPaSSXfsfZHYaxtQSZs6npFFUeRCyXB6dcC8ckn',
    local: '9beGuWXNoKPKCApT6xJUm5435Fz8EMGzoTTXgkcf3zAz',
  };

  readonly knownAddresses = Object.freeze({
    zBTCVST: address(
      this.runtime.cluster != 'devnet'
        ? 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg'
        : 'FaKEZbaAE42h7aCSzzUMKP8woZYBXh43v5bPzqb8CyH'
    ),
    zBTCVRT: address(
      this.runtime.cluster != 'local'
        ? 'VRTZTymeYQQZHQXV9ZkPYd5ug77dg4wvXoYkZEfUnQy'
        : 'DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf'
    ),
    cbBTCVST: address(
      this.runtime.cluster != 'devnet'
        ? 'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij'
        : 'FakEcBQk7MfreV3anJK6q136sPcieQ5dmmxXjaxfskGt'
    ),
    cbBTCVRT: address(
      this.runtime.cluster != 'local'
        ? 'VRTCayksMFJk3qtLGxL9Cgpoxi386MEiGbpr4Nbvb8i'
        : 'BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh'
    ),
    wBTCVST: address(
      this.runtime.cluster != 'devnet'
        ? '3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh'
        : 'FaKEwBj5eNHg8en4Tv1YuQYUSjXnR9TZfVLaMsy3qv7s'
    ),
    wBTCVRT: address(
      this.runtime.cluster != 'local'
        ? 'VRTWzkPMnMu57KyGNTjREFKzNjZ4BHwzMbsbvoHEe6q'
        : '4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL'
    ),
  });

  vault(
    vaultAddressResolver: AccountAddressResolverVariant<SolvBTCVaultProgram>
  ) {
    return new SolvVaultAccountContext(this, vaultAddressResolver);
  }

  readonly zBTC = SolvVaultAccountContext.fromSeeds(this, {
    supportedTokenMint: this.knownAddresses.zBTCVST,
    receiptTokenMint: this.knownAddresses.zBTCVRT,
  });

  readonly cbBTC = SolvVaultAccountContext.fromSeeds(this, {
    supportedTokenMint: this.knownAddresses.cbBTCVST,
    receiptTokenMint: this.knownAddresses.cbBTCVRT,
  });

  readonly wBTC = SolvVaultAccountContext.fromSeeds(this, {
    supportedTokenMint: this.knownAddresses.wBTCVST,
    receiptTokenMint: this.knownAddresses.wBTCVRT,
  });
}
