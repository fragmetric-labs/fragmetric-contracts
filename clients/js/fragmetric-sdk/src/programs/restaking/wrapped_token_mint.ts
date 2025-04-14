import {
  FragmetricMetadataContext,
  TokenMintAccountContext,
} from '../../context';
import { createMintTransactionTemplate } from './mint';
import { RestakingReceiptTokenMintAccountContext } from './receipt_token_mint';

export class RestakingWrappedTokenMintAccountContext extends TokenMintAccountContext<RestakingReceiptTokenMintAccountContext> {
  constructor(readonly parent: RestakingReceiptTokenMintAccountContext) {
    super(parent, async (parent) => {
      const fund = await parent.fund.resolveAccount(true);
      if (!fund?.data.wrappedToken?.enabled) {
        return null;
      }
      return fund.data.wrappedToken.mint;
    });
  }

  readonly metadata = FragmetricMetadataContext.from(this);

  /** authorized transactions **/
  readonly initializeMint = createMintTransactionTemplate(
    this,
    'initialize wrapped token mint'
  );
}
