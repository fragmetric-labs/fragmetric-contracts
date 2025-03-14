import * as token from '@solana-program/token';
import * as token2022 from '@solana-program/token-2022';
import { Account, type Address, EncodedAccount } from '@solana/kit';
import { Context } from '../context';
import { AccountContext } from './context';

export class TokenAccountContext<P extends Context<any>> extends AccountContext<
  P,
  Account<token.Token | token2022.Token>
> {
  async resolve(noCache = false) {
    return this.resolveAccount(noCache).then(
      (account) => account?.data ?? null
    );
  }

  static findAssociatedTokenAccountAddress(config: {
    owner: string;
    mint: string;
    tokenProgram?: string;
    associatedTokenProgram?: string;
  }) {
    return token
      .findAssociatedTokenPda(
        {
          owner: config.owner as Address,
          mint: config.mint as Address,
          tokenProgram: (config.tokenProgram ??
            token.TOKEN_PROGRAM_ADDRESS) as Address,
        },
        {
          programAddress: (config.associatedTokenProgram ??
            token.ASSOCIATED_TOKEN_PROGRAM_ADDRESS) as Address,
        }
      )
      .then((res) => res[0]);
  }

  static fromAssociatedTokenSeeds<P extends Context<any>>(
    parent: P,
    addressResolver: (parent: P) => Promise<{
      owner: string;
      mint: string;
      tokenProgram?: string;
      associatedTokenProgram?: string;
    } | null>
  ) {
    return new TokenAccountContext(parent, async (parent) => {
      const config = await addressResolver(parent);
      if (!config) {
        return null;
      }
      return TokenAccountContext.findAssociatedTokenAccountAddress(config);
    });
  }

  static fromAssociatedTokenSeeds2022<P extends Context<any>>(
    parent: P,
    addressResolver: (parent: P) => Promise<{
      owner: string;
      mint: string;
      associatedTokenProgram?: string;
    } | null>
  ) {
    return new TokenAccountContext(parent, async (parent) => {
      const config = await addressResolver(parent);
      if (!config) {
        return null;
      }
      return TokenAccountContext.findAssociatedTokenAccountAddress({
        ...config,
        tokenProgram: token2022.TOKEN_2022_PROGRAM_ADDRESS,
      });
    });
  }

  protected __decodeAccount(account: EncodedAccount) {
    if (account.programAddress == token2022.TOKEN_2022_PROGRAM_ADDRESS) {
      return token2022.decodeToken(account);
    }
    return token.decodeToken(account);
  }

  toContextDescription() {
    const desc = super.toContextDescription();
    const res = {
      ...desc,
      properties: {
        ...desc.properties,
        amount: this.__account?.data.amount,
        mint: this.__account?.data.mint,
      },
    };
    return res;
  }
}

export class TokenMintAccountContext<
  P extends Context<any>,
> extends AccountContext<P, Account<token.Mint | token2022.Mint>> {
  async resolve(noCache = false): Promise<any> {
    return this.resolveAccount(noCache).then(
      (account) => account?.data ?? null
    );
  }

  protected __decodeAccount(account: EncodedAccount) {
    if (account.programAddress == token2022.TOKEN_2022_PROGRAM_ADDRESS) {
      return token2022.decodeMint(account);
    }
    return token.decodeMint(account);
  }

  toContextDescription() {
    const desc = super.toContextDescription();
    const res = {
      ...desc,
      properties: {
        ...desc.properties,
        supply: this.__account?.data.supply,
        decimals: this.__account?.data.decimals,
      },
    };
    return res;
  }
}
