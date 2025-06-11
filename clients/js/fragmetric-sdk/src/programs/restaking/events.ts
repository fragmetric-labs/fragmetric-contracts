import { type Decoder, ReadonlyUint8Array } from '@solana/kit';
import {
  getFundManagerUpdatedFundDecoder,
  getFundManagerUpdatedFundDiscriminatorBytes,
  getFundManagerUpdatedRewardPoolDecoder,
  getFundManagerUpdatedRewardPoolDiscriminatorBytes,
  getOperatorDonatedToFundDecoder,
  getOperatorDonatedToFundDiscriminatorBytes,
  getOperatorRanFundCommandDecoder,
  getOperatorRanFundCommandDiscriminatorBytes,
  getOperatorUpdatedFundPricesDecoder,
  getOperatorUpdatedFundPricesDiscriminatorBytes,
  getOperatorUpdatedNormalizedTokenPoolPricesDecoder,
  getOperatorUpdatedNormalizedTokenPoolPricesDiscriminatorBytes,
  getOperatorUpdatedRewardPoolsDecoder,
  getOperatorUpdatedRewardPoolsDiscriminatorBytes,
  getUserCanceledWithdrawalRequestFromFundDecoder,
  getUserCanceledWithdrawalRequestFromFundDiscriminatorBytes,
  getUserClaimedRewardDecoder,
  getUserClaimedRewardDiscriminatorBytes,
  getUserCreatedOrUpdatedFundAccountDecoder,
  getUserCreatedOrUpdatedFundAccountDiscriminatorBytes,
  getUserCreatedOrUpdatedRewardAccountDecoder,
  getUserCreatedOrUpdatedRewardAccountDiscriminatorBytes,
  getUserDelegatedRewardAccountDecoder,
  getUserDelegatedRewardAccountDiscriminatorBytes,
  getUserDepositedToFundDecoder,
  getUserDepositedToFundDiscriminatorBytes,
  getUserRequestedWithdrawalFromFundDecoder,
  getUserRequestedWithdrawalFromFundDiscriminatorBytes,
  getUserTransferredReceiptTokenDecoder,
  getUserTransferredReceiptTokenDiscriminatorBytes,
  getUserUnwrappedReceiptTokenDecoder,
  getUserUnwrappedReceiptTokenDiscriminatorBytes,
  getUserUpdatedRewardPoolDecoder,
  getUserUpdatedRewardPoolDiscriminatorBytes,
  getUserWithdrewFromFundDecoder,
  getUserWithdrewFromFundDiscriminatorBytes,
  getUserWrappedReceiptTokenDecoder,
  getUserWrappedReceiptTokenDiscriminatorBytes,
} from '../../generated/restaking';

export type RestakingEventName = keyof typeof restakingAnchorEventDecoders;

export function getRestakingAnchorEventDecoders<
  EVENT_NAMES extends RestakingEventName,
>(
  ...eventNames: EVENT_NAMES[]
): Pick<typeof restakingAnchorEventDecoders, EVENT_NAMES> {
  const partial: Partial<typeof restakingAnchorEventDecoders> = {};

  for (const eventName of eventNames) {
    partial[eventName] = restakingAnchorEventDecoders[eventName];
  }

  return partial as Pick<typeof restakingAnchorEventDecoders, EVENT_NAMES>;
}

// 19 events on v0.6.1
export const restakingAnchorEventDecoders = {
  // fund manager
  fundManagerUpdatedFund: {
    discriminator: getFundManagerUpdatedFundDiscriminatorBytes(),
    decoder: getFundManagerUpdatedFundDecoder(),
  },
  fundManagerUpdatedRewardPool: {
    discriminator: getFundManagerUpdatedRewardPoolDiscriminatorBytes(),
    decoder: getFundManagerUpdatedRewardPoolDecoder(),
  },

  // operator
  operatorRanFundCommand: {
    discriminator: getOperatorRanFundCommandDiscriminatorBytes(),
    decoder: getOperatorRanFundCommandDecoder(),
  },
  operatorDonatedToFund: {
    discriminator: getOperatorDonatedToFundDiscriminatorBytes(),
    decoder: getOperatorDonatedToFundDecoder(),
  },
  operatorUpdatedFundPrices: {
    discriminator: getOperatorUpdatedFundPricesDiscriminatorBytes(),
    decoder: getOperatorUpdatedFundPricesDecoder(),
  },
  operatorUpdatedNormalizedTokenPoolPrices: {
    discriminator:
      getOperatorUpdatedNormalizedTokenPoolPricesDiscriminatorBytes(),
    decoder: getOperatorUpdatedNormalizedTokenPoolPricesDecoder(),
  },
  operatorUpdatedRewardPools: {
    discriminator: getOperatorUpdatedRewardPoolsDiscriminatorBytes(),
    decoder: getOperatorUpdatedRewardPoolsDecoder(),
  },

  // user
  userCreatedOrUpdatedFundAccount: {
    discriminator: getUserCreatedOrUpdatedFundAccountDiscriminatorBytes(),
    decoder: getUserCreatedOrUpdatedFundAccountDecoder(),
  },
  userCreatedOrUpdatedRewardAccount: {
    discriminator: getUserCreatedOrUpdatedRewardAccountDiscriminatorBytes(),
    decoder: getUserCreatedOrUpdatedRewardAccountDecoder(),
  },
  userDepositedToFund: {
    discriminator: getUserDepositedToFundDiscriminatorBytes(),
    decoder: getUserDepositedToFundDecoder(),
  },
  userRequestedWithdrawalFromFund: {
    discriminator: getUserRequestedWithdrawalFromFundDiscriminatorBytes(),
    decoder: getUserRequestedWithdrawalFromFundDecoder(),
  },
  userCanceledWithdrawalRequestFromFund: {
    discriminator: getUserCanceledWithdrawalRequestFromFundDiscriminatorBytes(),
    decoder: getUserCanceledWithdrawalRequestFromFundDecoder(),
  },
  userWithdrewFromFund: {
    discriminator: getUserWithdrewFromFundDiscriminatorBytes(),
    decoder: getUserWithdrewFromFundDecoder(),
  },
  userWrappedReceiptToken: {
    discriminator: getUserWrappedReceiptTokenDiscriminatorBytes(),
    decoder: getUserWrappedReceiptTokenDecoder(),
  },
  userUnwrappedReceiptToken: {
    discriminator: getUserUnwrappedReceiptTokenDiscriminatorBytes(),
    decoder: getUserUnwrappedReceiptTokenDecoder(),
  },
  userTransferredReceiptToken: {
    discriminator: getUserTransferredReceiptTokenDiscriminatorBytes(),
    decoder: getUserTransferredReceiptTokenDecoder(),
  },
  userUpdatedRewardPool: {
    discriminator: getUserUpdatedRewardPoolDiscriminatorBytes(),
    decoder: getUserUpdatedRewardPoolDecoder(),
  },
  userClaimedReward: {
    discriminator: getUserClaimedRewardDiscriminatorBytes(),
    decoder: getUserClaimedRewardDecoder(),
  },
  userDelegatedRewardAccount: {
    discriminator: getUserDelegatedRewardAccountDiscriminatorBytes(),
    decoder: getUserDelegatedRewardAccountDecoder(),
  },
} satisfies {
  [k in string]: {
    discriminator: ReadonlyUint8Array;
    decoder: Decoder<any>;
  };
};
