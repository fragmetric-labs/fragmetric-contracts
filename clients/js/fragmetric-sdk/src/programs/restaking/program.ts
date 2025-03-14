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

  readonly knownAddresses = Object.freeze(
    (() => {
      switch (this.runtime.cluster) {
        case 'mainnet':
          return {
            admin: address('fragSkuEpEmdoj9Bcyawk9rBdsChcVJLWHfj9JX1Gby'),
            fundManager: address(
              '79AHDsvEiM4MNrv8GPysgiGPj1ZPmxviF3dw29akYC84'
            ),
            fragSOL: address('FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo'),
            fragJTO: address('FRAGJ157KSDfGvBJtCSrsTWUqFnZhrw4aC8N8LqHuoos'),
          };
        case 'devnet':
          return {
            admin: address('fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP'),
            fundManager: address(
              '5UpLTLA7Wjqp7qdfjuTtPcUw3aVtbqFA5Mgm34mxPNg2'
            ),
            fragSOL: address('FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo'),
            fragJTO: address('FRAGJ157KSDfGvBJtCSrsTWUqFnZhrw4aC8N8LqHuoos'),
          };
        default:
          return {
            admin: address('9b2RSMDYskVvjVbwF4cVwEhZUaaaUgyYSxvESmnoS4LL'),
            fundManager: address(
              '5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx'
            ),
            fragSOL: address('Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD'),
            fragJTO: address('bxn2sjQkkoe1MevsZHWQdVeaY18uTNr9KYUjJsYmC7v'),
          };
      }
    })()
  );

  readonly __dev = createDevTools(this);

  receiptTokenMint(
    addressResolver: AccountAddressResolverVariant<RestakingProgram>
  ) {
    return new RestakingReceiptTokenMintAccountContext(this, addressResolver);
  }

  readonly fragSOL = this.receiptTokenMint(this.knownAddresses.fragSOL);

  readonly fragJTO = this.receiptTokenMint(this.knownAddresses.fragJTO);
}
