import { FragmetricMetadataContext, TokenMintAccountContext } from '../../context';
import { RestakingReceiptTokenMintAccountContext } from './receipt_token_mint';
import { createMintTransactionTemplate } from './mint';

export class RestakingNormalizedTokenMintAccountContext extends TokenMintAccountContext<RestakingReceiptTokenMintAccountContext> {
  constructor(readonly parent: RestakingReceiptTokenMintAccountContext) {
    super(parent, async (parent) => {
      const fund = await parent.fund.resolveAccount(true);
      if (!fund?.data.normalizedToken?.enabled) {
        return null;
      }
      return fund.data.normalizedToken.mint;
    });
  }

  readonly metadata = FragmetricMetadataContext.from(this);

  /** authorized transactions **/
  readonly initializeMint = createMintTransactionTemplate(
    this,
    'initialize normalized token mint'
  );
}
