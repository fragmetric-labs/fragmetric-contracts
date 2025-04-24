import { address } from '@solana/kit';
import {
  AccountAddressResolverVariant,
  AccountContext,
  ProgramContext,
} from '../../context';
import { SolvVaultReceiptTokenMintAccountContext } from './receipt_token_mint';

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
    zBTC: address(
      this.runtime.cluster != 'devnet'
        ? 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg'
        : 'FaKEZbaAE42h7aCSzzUMKP8woZYBXh43v5bPzqb8CyH'
    ),
    cbBTC: address(
      this.runtime.cluster != 'devnet'
        ? 'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij'
        : 'FakEcBQk7MfreV3anJK6q136sPcieQ5dmmxXjaxfskGt'
    ),
    wBTC: address(
      this.runtime.cluster != 'devnet'
        ? '3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh'
        : 'FaKEwBj5eNHg8en4Tv1YuQYUSjXnR9TZfVLaMsy3qv7s'
    ),
    zBTCVRT: address(
      this.runtime.cluster != 'local'
        ? 'VRTZTymeYQQZHQXV9ZkPYd5ug77dg4wvXoYkZEfUnQy'
        : 'DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf'
    ),
    cbBTCVRT: address(
      this.runtime.cluster != 'local'
        ? 'VRTCayksMFJk3qtLGxL9Cgpoxi386MEiGbpr4Nbvb8i'
        : 'BDYcrsJ6Y4kPdkReieh4RV58ziMNsYnMPpnDZgyAsdmh'
    ),
    wBTCVRT: address(
      this.runtime.cluster != 'local'
        ? 'VRTWzkPMnMu57KyGNTjREFKzNjZ4BHwzMbsbvoHEe6q'
        : '4hNFn9hWmL4xxH7PxnZntFcDyEhXx5vHu4uM5rNj4fcL'
    ),
  });

  receiptTokenMint(
    mintAddressResolver: AccountAddressResolverVariant<SolvBTCVaultProgram>,
    supportedTokenMintAddress: string
  ) {
    return new SolvVaultReceiptTokenMintAccountContext(
      this,
      mintAddressResolver,
      supportedTokenMintAddress
    );
  }

  readonly zBTC = this.receiptTokenMint(
    this.knownAddresses.zBTCVRT,
    this.knownAddresses.zBTC
  );

  readonly cbBTC = this.receiptTokenMint(
    this.knownAddresses.cbBTCVRT,
    this.knownAddresses.cbBTC
  );

  readonly wBTC = this.receiptTokenMint(
    this.knownAddresses.wBTCVRT,
    this.knownAddresses.wBTC
  );
}
