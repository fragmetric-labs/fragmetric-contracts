import { address } from '@solana/kit';
import {
  AccountAddressResolverVariant,
  AccountContext,
  ProgramContext,
} from '../../context';
import { createDevTools } from './__devtools';
import { RestakingReceiptTokenMintAccountContext } from './receipt_token_mint';

export class RestakingProgram extends ProgramContext {
  async resolve(noCache = false) {
    await Promise.all(
      Object.values(this)
        .filter((value) => value instanceof AccountContext)
        .map((value) => value.resolveAccountTree(noCache))
    );
  }

  static readonly addresses = {
    mainnet: 'fragnAis7Bp6FTsMoa6YcH8UffhEw43Ph79qAiK3iF3',
    devnet: 'frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ',
    local: '4qEHCzsLFUnw8jmhmRSmAK5VhZVoSD1iVqukAf92yHi5',
  };

  readonly knownAddresses = Object.freeze({
    admin: address(
      this.runtime.cluster === 'mainnet'
        ? 'fragSkuEpEmdoj9Bcyawk9rBdsChcVJLWHfj9JX1Gby'
        : this.runtime.cluster === 'devnet'
          ? 'fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP'
          : '9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL'
    ),
    fundManager: address(
      this.runtime.cluster === 'mainnet'
        ? '79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84'
        : this.runtime.cluster == 'devnet'
          ? '5UpLTLA7Wjqp7qdfjuTtPcUw3aVtbqFA5Mgm34mxPNg2'
          : '5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx'
    ),
    fragSOL: address(
      this.runtime.cluster != 'local'
        ? 'FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo'
        : 'Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD'
    ),
    fragJTO: address(
      this.runtime.cluster != 'local'
        ? 'FRAGJ157KSDfGvBJtCSrsTWUqFnZhrw4aC8N8LqHuoos'
        : 'bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v'
    ),
    fragBTC: address(
      this.runtime.cluster != 'local'
        ? 'FRAGB4KZGLMy3wH1nBajP3Q17MHnecEvTPT6wb4pX5MB'
        : 'ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz'
    ),
  });

  readonly __dev = createDevTools(this);

  receiptTokenMint(
    addressResolver: AccountAddressResolverVariant<RestakingProgram>
  ) {
    return new RestakingReceiptTokenMintAccountContext(this, addressResolver);
  }

  readonly fragSOL = this.receiptTokenMint(this.knownAddresses.fragSOL);

  readonly fragJTO = this.receiptTokenMint(this.knownAddresses.fragJTO);

  readonly fragBTC = this.receiptTokenMint(this.knownAddresses.fragBTC);
}
