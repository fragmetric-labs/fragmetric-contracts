import { createSolanaRpc, createSolanaRpcSubscriptions } from '@solana/kit';
import { describe, expect, test } from 'vitest';
import { RestakingProgram } from './program';

const devnetRpcUrl = process.env.SOLANA_RPC_DEVNET
  ? process.env.SOLANA_RPC_DEVNET
  : 'https://api.devnet.solana.com';
const devnetRpcSubscriptionsUrl = devnetRpcUrl
  .replace('https://', 'wss://')
  .replace('http://', 'ws://');

const mainnetRpcUrl = process.env.SOLANA_RPC_MAINNET
  ? process.env.SOLANA_RPC_MAINNET
  : 'https://api.mainnet-beta.solana.com';
const mainnetRpcSubscriptionsUrl = mainnetRpcUrl
  .replace('https://', 'wss://')
  .replace('http://', 'ws://');

describe.each([
  [
    'devnet',
    RestakingProgram.connect({
      rpc: createSolanaRpc(devnetRpcUrl),
      rpcSubscriptions: createSolanaRpcSubscriptions(devnetRpcSubscriptionsUrl),
      cluster: 'devnet',
    }),
  ],
  [
    'mainnet',
    RestakingProgram.connect({
      rpc: createSolanaRpc(mainnetRpcUrl),
      rpcSubscriptions: createSolanaRpcSubscriptions(
        mainnetRpcSubscriptionsUrl
      ),
      cluster: 'mainnet',
    }),
  ] as [string, RestakingProgram],
])('RestakingProgram basic test on %s', (cluster, program) => {
  test('derive program address based on cluster', async () => {
    expect(program.address).equals(
      cluster == 'mainnet'
        ? 'fragnAis7Bp6FTsMoa6YcH8UffhEw43Ph79qAiK3iF3'
        : 'frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ'
    );
  });

  test('resolve fund/reward/receipt-token-mint accounts', async () => {
    // resolve mint data
    await program.fragSOL.resolveAccount();

    // fund account is not resolved yet
    expect(program.fragSOL.fund.account).toBeUndefined();

    expect(await program.fragSOL.fund.resolveAccount()).toMatchObject({
      data: {
        receiptTokenMint: 'FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo',
        receiptTokenDecimals: 9,
        // ... omitted
      },
    });

    // mint authority is given to the fund
    expect(program.fragSOL.account).toMatchObject({
      data: {
        mintAuthority: {
          __option: 'Some',
          value: program.fragSOL.fund.address,
        },
      },
    });

    // also reward account is not resolved yet
    expect(program.fragSOL.reward.account).toBeUndefined();

    expect(await program.fragSOL.reward.resolveAccount()).toMatchObject({
      data: {
        receiptTokenMint: 'FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo',
        // ... omitted
      },
    });

    // fund wrap account has a user reward account
    await expect(
      program.fragSOL.fund.wrap.reward.resolveAccount()
    ).resolves.toMatchObject({
      data: {
        receiptTokenMint: 'FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo',
        user: await program.fragSOL.fund.wrap.resolveAddress(),
      },
    });
  });

  test('initialization with non-existing accounts for operation and testing', async () => {
    const invalidTokenMint = program.receiptTokenMint(
      'ComputeBudget111111111111111111111111111111'
    );
    await expect(invalidTokenMint.resolveAccount()).rejects.toThrow(
      'Failed to decode account'
    );
    await expect(invalidTokenMint.fund.resolveAddress()).resolves.toBeTypeOf(
      'string'
    );
    await expect(invalidTokenMint.fund.resolveAccount()).resolves.toBeNull();
  });
});

test('can traverse context graph', async () => {
  expect(
    RestakingProgram.devnet(process.env.SOLANA_RPC_DEVNET).toContextTreeString()
  ).toMatchInlineSnapshot(`
      "(this)                                                RestakingProgram address=frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ
      ├── fragSOL                                           RestakingReceiptTokenMintAccount address=undefined, supply=undefined, decimals=undefined
      │   ├── metadata                                      FragmetricMetadata address=undefined
      │   ├── fund                                          RestakingFundAccount address=undefined
      │   │   ├── reserve                                   RestakingFundReserveAccount address=undefined, lamports=undefined
      │   │   │   ├── supportedTokens                       IterativeAccount length=undefined, types=undefined
      │   │   │   ├── normalizedToken                       TokenAccount address=undefined, amount=undefined, mint=undefined
      │   │   │   └── restakingVaultReceiptTokens           IterativeAccount length=undefined, types=undefined
      │   │   ├── lockedReceiptToken                        TokenAccount address=undefined, amount=undefined, mint=undefined
      │   │   ├── latestWithdrawalBatches                   IterativeAccount length=undefined, types=undefined
      │   │   ├── restakingVaults                           IterativeAccount length=undefined, types=undefined
      │   │   ├── wrap                                      RestakingFundWrapAccount address=undefined, lamports=undefined
      │   │   │   ├── reward                                RestakingFundWrapRewardAccount address=undefined
      │   │   │   │   ├── updatePools                       TransactionTemplate args=null, events=userUpdatedRewardPool, description=manually triggers contribution synchronization for the user reward pools
      │   │   │   │   ├── initializeOrUpdateAccount         TransactionTemplate args=null, events=userCreatedOrUpdatedRewardAccount, description=initialize or update user reward account
      │   │   │   │   ├── claim                             TransactionTemplate args=delegate,isBonus,mint,amount,recipient, events=userClaimedReward, description=claim rewards
      │   │   │   │   ├── delegate                          TransactionTemplate args=delegate,newDelegate, events=userDelegatedRewardAccount, description=delegate user reward account
      │   │   │   │   └── resetDelegate                     TransactionTemplate args=null, events=userDelegatedRewardAccount, description=reset delegate of reward account (in case of either fund wrap or wra
      │   │   │   ├── receiptToken                          TokenAccount address=undefined, amount=undefined, mint=undefined
      │   │   │   └── holders                               IterativeAccount length=undefined, types=undefined
      │   │   ├── treasury                                  RestakingFundTreasuryAccount address=undefined, lamports=undefined
      │   │   │   └── supportedTokens                       IterativeAccount length=undefined, types=undefined
      │   │   ├── addressLookupTable                        RestakingFundAddressLookupTableAccount address=undefined, length=undefined, lastExtendedSlot=undefined
      │   │   │   └── initializeOrUpdateAccount             TransactionTemplate (chained) args=addresses, events=undefined, description=initialize or update address lookup table
      │   │   ├── updatePrices                              TransactionTemplate args=null, events=operatorUpdatedFundPrices, description=manually triggers price updates for the receipt token and underlying 
      │   │   ├── donate                                    TransactionTemplate args=assetMint,assetAmount,offsetReceivable,applyPresetComputeUnitLimit, events=operatorDonatedToFund, description=donate supp
      │   │   ├── runCommand                                TransactionTemplate (chained) args=null, events=operatorRanFundCommand, description=execute the next fund command to circulate assets
      │   │   ├── initializeOrUpdateAccount                 TransactionTemplate (chained) args=targetVersion, events=undefined, description=initialize or update fund account
      │   │   ├── updateGeneralStrategy                     TransactionTemplate args=depositEnabled,donationEnabled,transferEnabled,withdrawalEnabled,withdrawalBatchThresholdSeconds,withdrawalFeeRateBps, ev
      │   │   ├── addSupportedToken                         TransactionTemplate args=mint,program,pricingSource, events=fundManagerUpdatedFund, description=add a new supported token
      │   │   ├── updateAssetStrategy                       TransactionTemplate args=null, events=fundManagerUpdatedFund, description=update asset strategy of the fund
      │   │   ├── addTokenSwapStrategy                      TransactionTemplate args=fromTokenMint,toTokenMint,swapSource, events=fundManagerUpdatedFund, description=add a new token swap strategy
      │   │   ├── addRestakingVault                         TransactionTemplate args=vault,pricingSource, events=fundManagerUpdatedFund, description=add a new restaking vault
      │   │   ├── addRestakingVaultDelegation               TransactionTemplate args=vault,operator, events=fundManagerUpdatedFund, description=add a new operator delegation to a restaking vault
      │   │   ├── updateRestakingVaultStrategy              TransactionTemplate (chained) args=null, events=fundManagerUpdatedFund, description=update restaking vault strategy of the fund
      │   │   ├── addRestakingVaultCompoundingReward        TransactionTemplate args=vault,rewardTokenMint, events=fundManagerUpdatedFund, description=add a new compounding reward to a restaking vault
      │   │   ├── removeRestakingVaultCompoundingReward     TransactionTemplate args=vault,rewardTokenMint, events=fundManagerUpdatedFund, description=remove a compounding reward from a restaking vault
      │   │   ├── addRestakingVaultDistributingReward       TransactionTemplate args=vault,rewardTokenMint, events=fundManagerUpdatedFund, description=add a new distributing reward to a restaking vault
      │   │   ├── removeRestakingVaultDistributingReward    TransactionTemplate args=vault,rewardTokenMint, events=fundManagerUpdatedFund, description=remove a distributing reward from a restaking vault
      │   │   ├── initializeNormalizedToken                 TransactionTemplate args=mint, events=fundManagerUpdatedFund, description=initialize normalized token pool account and enable
      │   │   ├── initializeWrappedToken                    TransactionTemplate args=mint, events=fundManagerUpdatedFund, description=enable wrapped token
      │   │   └── initializeWrappedTokenHolder              TransactionTemplate args=wrappedTokenAccount, events=userCreatedOrUpdatedRewardAccount,fundManagerUpdatedFund, description=add new wrapped token h
      │   ├── reward                                        RestakingRewardAccount address=undefined
      │   │   ├── reserve                                   RestakingRewardReserveAccount address=undefined, lamports=undefined
      │   │   │   └── rewardTokens                          IterativeAccount length=undefined, types=undefined
      │   │   ├── updatePools                               TransactionTemplate args=null, events=operatorUpdatedRewardPools, description=manually triggers contribution synchronization for the global reward
      │   │   ├── initializeOrUpdateAccount                 TransactionTemplate (chained) args=targetVersion, events=undefined, description=initialize or update reward account
      │   │   ├── addReward                                 TransactionTemplate args=mint,program,decimals,name,description, events=fundManagerUpdatedRewardPool, description=register a new reward (non-claim
      │   │   ├── updateReward                              TransactionTemplate args=mint,newMint,newProgram,newDecimals,claimable, events=fundManagerUpdatedRewardPool, description=update a non-claimable re
      │   │   └── settleReward                              TransactionTemplate args=isBonus,mint,amount, events=fundManagerUpdatedRewardPool, description=settle a reward
      │   ├── normalizedTokenMint                           RestakingNormalizedTokenMintAccount address=undefined, supply=undefined, decimals=undefined
      │   │   ├── metadata                                  FragmetricMetadata address=undefined
      │   │   └── initializeMint                            TransactionTemplate args=mint,name,symbol,uri,description,decimals, events=undefined, description=initialize normalized token mint
      │   ├── normalizedTokenPool                           RestakingNormalizedTokenPoolAccount address=undefined
      │   │   ├── supportedTokens                           IterativeAccount length=undefined, types=undefined
      │   │   ├── updatePrices                              TransactionTemplate args=null, events=operatorUpdatedNormalizedTokenPoolPrices, description=manually triggers price updates for the normalized tok
      │   │   └── addSupportedToken                         TransactionTemplate args=mint,program,pricingSource, events=operatorUpdatedNormalizedTokenPoolPrices, description=add a new supported token to the
      │   ├── wrappedTokenMint                              RestakingWrappedTokenMintAccount address=undefined, supply=undefined, decimals=undefined
      │   │   ├── metadata                                  FragmetricMetadata address=undefined
      │   │   └── initializeMint                            TransactionTemplate args=mint,name,symbol,uri,description,decimals, events=undefined, description=initialize wrapped token mint
      │   ├── payer                                         RestakingUserAccount address=undefined, lamports=undefined
      │   │   ├── fund                                      RestakingUserFundAccount address=undefined
      │   │   │   └── initializeOrUpdateAccount             TransactionTemplate args=null, events=userCreatedOrUpdatedFundAccount, description=initialize or update user fund account
      │   │   ├── reward                                    RestakingUserRewardAccount address=undefined
      │   │   │   ├── updatePools                           TransactionTemplate args=null, events=userUpdatedRewardPool, description=manually triggers contribution synchronization for the user reward pools
      │   │   │   ├── initializeOrUpdateAccount             TransactionTemplate args=null, events=userCreatedOrUpdatedRewardAccount, description=initialize or update user reward account
      │   │   │   ├── claim                                 TransactionTemplate args=delegate,isBonus,mint,amount,recipient, events=userClaimedReward, description=claim rewards
      │   │   │   ├── delegate                              TransactionTemplate args=delegate,newDelegate, events=userDelegatedRewardAccount, description=delegate user reward account
      │   │   │   └── resetDelegate                         TransactionTemplate args=null, events=userDelegatedRewardAccount, description=reset delegate of reward account (in case of either fund wrap or wra
      │   │   ├── receiptToken                              TokenAccount address=undefined, amount=undefined, mint=undefined
      │   │   ├── wrappedToken                              TokenAccount address=undefined, amount=undefined, mint=undefined
      │   │   ├── supportedTokens                           IterativeAccount length=undefined, types=undefined
      │   │   ├── rewardTokens                              IterativeAccount length=undefined, types=undefined
      │   │   ├── deposit                                   TransactionTemplate args=assetMint,assetAmount,metadata,applyPresetComputeUnitLimit, events=userDepositedToFund,userCreatedOrUpdatedFundAccount,us
      │   │   ├── requestWithdrawal                         TransactionTemplate args=assetMint,receiptTokenAmount, events=userRequestedWithdrawalFromFund,userCreatedOrUpdatedFundAccount,userCreatedOrUpdated
      │   │   ├── cancelWithdrawalRequest                   TransactionTemplate args=assetMint,requestId, events=userCanceledWithdrawalRequestFromFund,userCreatedOrUpdatedFundAccount,userCreatedOrUpdatedRew
      │   │   ├── withdraw                                  TransactionTemplate args=assetMint,requestId, events=userWithdrewFromFund,userCreatedOrUpdatedFundAccount,userCreatedOrUpdatedRewardAccount, descr
      │   │   ├── wrap                                      TransactionTemplate args=receiptTokenAmount,receiptTokenAmountAsTargetBalance, events=userWrappedReceiptToken, description=convert receipt tokens 
      │   │   ├── unwrap                                    TransactionTemplate args=wrappedTokenAmount, events=userUnwrappedReceiptToken, description=convert wrapped tokens back into receipt tokens
      │   │   └── transfer                                  TransactionTemplate args=receiptTokenAmount,recipient, events=userTransferredReceiptToken, description=transfer receipt token
      │   ├── initializeMint                                TransactionTemplate args=name,symbol,uri,description,decimals, events=undefined, description=initialize receipt token mint
      │   └── initializeOrUpdateExtraAccountMetaList        TransactionTemplate args=null, events=undefined, description=initialize or update extra account meta list
      ├── fragJTO                                           RestakingReceiptTokenMintAccount address=undefined, supply=undefined, decimals=undefined
      │   ├── metadata                                      FragmetricMetadata address=undefined
      │   ├── fund                                          RestakingFundAccount address=undefined
      │   │   ├── reserve                                   RestakingFundReserveAccount address=undefined, lamports=undefined
      │   │   │   ├── supportedTokens                       IterativeAccount length=undefined, types=undefined
      │   │   │   ├── normalizedToken                       TokenAccount address=undefined, amount=undefined, mint=undefined
      │   │   │   └── restakingVaultReceiptTokens           IterativeAccount length=undefined, types=undefined
      │   │   ├── lockedReceiptToken                        TokenAccount address=undefined, amount=undefined, mint=undefined
      │   │   ├── latestWithdrawalBatches                   IterativeAccount length=undefined, types=undefined
      │   │   ├── restakingVaults                           IterativeAccount length=undefined, types=undefined
      │   │   ├── wrap                                      RestakingFundWrapAccount address=undefined, lamports=undefined
      │   │   │   ├── reward                                RestakingFundWrapRewardAccount address=undefined
      │   │   │   │   ├── updatePools                       TransactionTemplate args=null, events=userUpdatedRewardPool, description=manually triggers contribution synchronization for the user reward pools
      │   │   │   │   ├── initializeOrUpdateAccount         TransactionTemplate args=null, events=userCreatedOrUpdatedRewardAccount, description=initialize or update user reward account
      │   │   │   │   ├── claim                             TransactionTemplate args=delegate,isBonus,mint,amount,recipient, events=userClaimedReward, description=claim rewards
      │   │   │   │   ├── delegate                          TransactionTemplate args=delegate,newDelegate, events=userDelegatedRewardAccount, description=delegate user reward account
      │   │   │   │   └── resetDelegate                     TransactionTemplate args=null, events=userDelegatedRewardAccount, description=reset delegate of reward account (in case of either fund wrap or wra
      │   │   │   ├── receiptToken                          TokenAccount address=undefined, amount=undefined, mint=undefined
      │   │   │   └── holders                               IterativeAccount length=undefined, types=undefined
      │   │   ├── treasury                                  RestakingFundTreasuryAccount address=undefined, lamports=undefined
      │   │   │   └── supportedTokens                       IterativeAccount length=undefined, types=undefined
      │   │   ├── addressLookupTable                        RestakingFundAddressLookupTableAccount address=undefined, length=undefined, lastExtendedSlot=undefined
      │   │   │   └── initializeOrUpdateAccount             TransactionTemplate (chained) args=addresses, events=undefined, description=initialize or update address lookup table
      │   │   ├── updatePrices                              TransactionTemplate args=null, events=operatorUpdatedFundPrices, description=manually triggers price updates for the receipt token and underlying 
      │   │   ├── donate                                    TransactionTemplate args=assetMint,assetAmount,offsetReceivable,applyPresetComputeUnitLimit, events=operatorDonatedToFund, description=donate supp
      │   │   ├── runCommand                                TransactionTemplate (chained) args=null, events=operatorRanFundCommand, description=execute the next fund command to circulate assets
      │   │   ├── initializeOrUpdateAccount                 TransactionTemplate (chained) args=targetVersion, events=undefined, description=initialize or update fund account
      │   │   ├── updateGeneralStrategy                     TransactionTemplate args=depositEnabled,donationEnabled,transferEnabled,withdrawalEnabled,withdrawalBatchThresholdSeconds,withdrawalFeeRateBps, ev
      │   │   ├── addSupportedToken                         TransactionTemplate args=mint,program,pricingSource, events=fundManagerUpdatedFund, description=add a new supported token
      │   │   ├── updateAssetStrategy                       TransactionTemplate args=null, events=fundManagerUpdatedFund, description=update asset strategy of the fund
      │   │   ├── addTokenSwapStrategy                      TransactionTemplate args=fromTokenMint,toTokenMint,swapSource, events=fundManagerUpdatedFund, description=add a new token swap strategy
      │   │   ├── addRestakingVault                         TransactionTemplate args=vault,pricingSource, events=fundManagerUpdatedFund, description=add a new restaking vault
      │   │   ├── addRestakingVaultDelegation               TransactionTemplate args=vault,operator, events=fundManagerUpdatedFund, description=add a new operator delegation to a restaking vault
      │   │   ├── updateRestakingVaultStrategy              TransactionTemplate (chained) args=null, events=fundManagerUpdatedFund, description=update restaking vault strategy of the fund
      │   │   ├── addRestakingVaultCompoundingReward        TransactionTemplate args=vault,rewardTokenMint, events=fundManagerUpdatedFund, description=add a new compounding reward to a restaking vault
      │   │   ├── removeRestakingVaultCompoundingReward     TransactionTemplate args=vault,rewardTokenMint, events=fundManagerUpdatedFund, description=remove a compounding reward from a restaking vault
      │   │   ├── addRestakingVaultDistributingReward       TransactionTemplate args=vault,rewardTokenMint, events=fundManagerUpdatedFund, description=add a new distributing reward to a restaking vault
      │   │   ├── removeRestakingVaultDistributingReward    TransactionTemplate args=vault,rewardTokenMint, events=fundManagerUpdatedFund, description=remove a distributing reward from a restaking vault
      │   │   ├── initializeNormalizedToken                 TransactionTemplate args=mint, events=fundManagerUpdatedFund, description=initialize normalized token pool account and enable
      │   │   ├── initializeWrappedToken                    TransactionTemplate args=mint, events=fundManagerUpdatedFund, description=enable wrapped token
      │   │   └── initializeWrappedTokenHolder              TransactionTemplate args=wrappedTokenAccount, events=userCreatedOrUpdatedRewardAccount,fundManagerUpdatedFund, description=add new wrapped token h
      │   ├── reward                                        RestakingRewardAccount address=undefined
      │   │   ├── reserve                                   RestakingRewardReserveAccount address=undefined, lamports=undefined
      │   │   │   └── rewardTokens                          IterativeAccount length=undefined, types=undefined
      │   │   ├── updatePools                               TransactionTemplate args=null, events=operatorUpdatedRewardPools, description=manually triggers contribution synchronization for the global reward
      │   │   ├── initializeOrUpdateAccount                 TransactionTemplate (chained) args=targetVersion, events=undefined, description=initialize or update reward account
      │   │   ├── addReward                                 TransactionTemplate args=mint,program,decimals,name,description, events=fundManagerUpdatedRewardPool, description=register a new reward (non-claim
      │   │   ├── updateReward                              TransactionTemplate args=mint,newMint,newProgram,newDecimals,claimable, events=fundManagerUpdatedRewardPool, description=update a non-claimable re
      │   │   └── settleReward                              TransactionTemplate args=isBonus,mint,amount, events=fundManagerUpdatedRewardPool, description=settle a reward
      │   ├── normalizedTokenMint                           RestakingNormalizedTokenMintAccount address=undefined, supply=undefined, decimals=undefined
      │   │   ├── metadata                                  FragmetricMetadata address=undefined
      │   │   └── initializeMint                            TransactionTemplate args=mint,name,symbol,uri,description,decimals, events=undefined, description=initialize normalized token mint
      │   ├── normalizedTokenPool                           RestakingNormalizedTokenPoolAccount address=undefined
      │   │   ├── supportedTokens                           IterativeAccount length=undefined, types=undefined
      │   │   ├── updatePrices                              TransactionTemplate args=null, events=operatorUpdatedNormalizedTokenPoolPrices, description=manually triggers price updates for the normalized tok
      │   │   └── addSupportedToken                         TransactionTemplate args=mint,program,pricingSource, events=operatorUpdatedNormalizedTokenPoolPrices, description=add a new supported token to the
      │   ├── wrappedTokenMint                              RestakingWrappedTokenMintAccount address=undefined, supply=undefined, decimals=undefined
      │   │   ├── metadata                                  FragmetricMetadata address=undefined
      │   │   └── initializeMint                            TransactionTemplate args=mint,name,symbol,uri,description,decimals, events=undefined, description=initialize wrapped token mint
      │   ├── payer                                         RestakingUserAccount address=undefined, lamports=undefined
      │   │   ├── fund                                      RestakingUserFundAccount address=undefined
      │   │   │   └── initializeOrUpdateAccount             TransactionTemplate args=null, events=userCreatedOrUpdatedFundAccount, description=initialize or update user fund account
      │   │   ├── reward                                    RestakingUserRewardAccount address=undefined
      │   │   │   ├── updatePools                           TransactionTemplate args=null, events=userUpdatedRewardPool, description=manually triggers contribution synchronization for the user reward pools
      │   │   │   ├── initializeOrUpdateAccount             TransactionTemplate args=null, events=userCreatedOrUpdatedRewardAccount, description=initialize or update user reward account
      │   │   │   ├── claim                                 TransactionTemplate args=delegate,isBonus,mint,amount,recipient, events=userClaimedReward, description=claim rewards
      │   │   │   ├── delegate                              TransactionTemplate args=delegate,newDelegate, events=userDelegatedRewardAccount, description=delegate user reward account
      │   │   │   └── resetDelegate                         TransactionTemplate args=null, events=userDelegatedRewardAccount, description=reset delegate of reward account (in case of either fund wrap or wra
      │   │   ├── receiptToken                              TokenAccount address=undefined, amount=undefined, mint=undefined
      │   │   ├── wrappedToken                              TokenAccount address=undefined, amount=undefined, mint=undefined
      │   │   ├── supportedTokens                           IterativeAccount length=undefined, types=undefined
      │   │   ├── rewardTokens                              IterativeAccount length=undefined, types=undefined
      │   │   ├── deposit                                   TransactionTemplate args=assetMint,assetAmount,metadata,applyPresetComputeUnitLimit, events=userDepositedToFund,userCreatedOrUpdatedFundAccount,us
      │   │   ├── requestWithdrawal                         TransactionTemplate args=assetMint,receiptTokenAmount, events=userRequestedWithdrawalFromFund,userCreatedOrUpdatedFundAccount,userCreatedOrUpdated
      │   │   ├── cancelWithdrawalRequest                   TransactionTemplate args=assetMint,requestId, events=userCanceledWithdrawalRequestFromFund,userCreatedOrUpdatedFundAccount,userCreatedOrUpdatedRew
      │   │   ├── withdraw                                  TransactionTemplate args=assetMint,requestId, events=userWithdrewFromFund,userCreatedOrUpdatedFundAccount,userCreatedOrUpdatedRewardAccount, descr
      │   │   ├── wrap                                      TransactionTemplate args=receiptTokenAmount,receiptTokenAmountAsTargetBalance, events=userWrappedReceiptToken, description=convert receipt tokens 
      │   │   ├── unwrap                                    TransactionTemplate args=wrappedTokenAmount, events=userUnwrappedReceiptToken, description=convert wrapped tokens back into receipt tokens
      │   │   └── transfer                                  TransactionTemplate args=receiptTokenAmount,recipient, events=userTransferredReceiptToken, description=transfer receipt token
      │   ├── initializeMint                                TransactionTemplate args=name,symbol,uri,description,decimals, events=undefined, description=initialize receipt token mint
      │   └── initializeOrUpdateExtraAccountMetaList        TransactionTemplate args=null, events=undefined, description=initialize or update extra account meta list
      ├── fragBTC                                           RestakingReceiptTokenMintAccount address=undefined, supply=undefined, decimals=undefined
      │   ├── metadata                                      FragmetricMetadata address=undefined
      │   ├── fund                                          RestakingFundAccount address=undefined
      │   │   ├── reserve                                   RestakingFundReserveAccount address=undefined, lamports=undefined
      │   │   │   ├── supportedTokens                       IterativeAccount length=undefined, types=undefined
      │   │   │   ├── normalizedToken                       TokenAccount address=undefined, amount=undefined, mint=undefined
      │   │   │   └── restakingVaultReceiptTokens           IterativeAccount length=undefined, types=undefined
      │   │   ├── lockedReceiptToken                        TokenAccount address=undefined, amount=undefined, mint=undefined
      │   │   ├── latestWithdrawalBatches                   IterativeAccount length=undefined, types=undefined
      │   │   ├── restakingVaults                           IterativeAccount length=undefined, types=undefined
      │   │   ├── wrap                                      RestakingFundWrapAccount address=undefined, lamports=undefined
      │   │   │   ├── reward                                RestakingFundWrapRewardAccount address=undefined
      │   │   │   │   ├── updatePools                       TransactionTemplate args=null, events=userUpdatedRewardPool, description=manually triggers contribution synchronization for the user reward pools
      │   │   │   │   ├── initializeOrUpdateAccount         TransactionTemplate args=null, events=userCreatedOrUpdatedRewardAccount, description=initialize or update user reward account
      │   │   │   │   ├── claim                             TransactionTemplate args=delegate,isBonus,mint,amount,recipient, events=userClaimedReward, description=claim rewards
      │   │   │   │   ├── delegate                          TransactionTemplate args=delegate,newDelegate, events=userDelegatedRewardAccount, description=delegate user reward account
      │   │   │   │   └── resetDelegate                     TransactionTemplate args=null, events=userDelegatedRewardAccount, description=reset delegate of reward account (in case of either fund wrap or wra
      │   │   │   ├── receiptToken                          TokenAccount address=undefined, amount=undefined, mint=undefined
      │   │   │   └── holders                               IterativeAccount length=undefined, types=undefined
      │   │   ├── treasury                                  RestakingFundTreasuryAccount address=undefined, lamports=undefined
      │   │   │   └── supportedTokens                       IterativeAccount length=undefined, types=undefined
      │   │   ├── addressLookupTable                        RestakingFundAddressLookupTableAccount address=undefined, length=undefined, lastExtendedSlot=undefined
      │   │   │   └── initializeOrUpdateAccount             TransactionTemplate (chained) args=addresses, events=undefined, description=initialize or update address lookup table
      │   │   ├── updatePrices                              TransactionTemplate args=null, events=operatorUpdatedFundPrices, description=manually triggers price updates for the receipt token and underlying 
      │   │   ├── donate                                    TransactionTemplate args=assetMint,assetAmount,offsetReceivable,applyPresetComputeUnitLimit, events=operatorDonatedToFund, description=donate supp
      │   │   ├── runCommand                                TransactionTemplate (chained) args=null, events=operatorRanFundCommand, description=execute the next fund command to circulate assets
      │   │   ├── initializeOrUpdateAccount                 TransactionTemplate (chained) args=targetVersion, events=undefined, description=initialize or update fund account
      │   │   ├── updateGeneralStrategy                     TransactionTemplate args=depositEnabled,donationEnabled,transferEnabled,withdrawalEnabled,withdrawalBatchThresholdSeconds,withdrawalFeeRateBps, ev
      │   │   ├── addSupportedToken                         TransactionTemplate args=mint,program,pricingSource, events=fundManagerUpdatedFund, description=add a new supported token
      │   │   ├── updateAssetStrategy                       TransactionTemplate args=null, events=fundManagerUpdatedFund, description=update asset strategy of the fund
      │   │   ├── addTokenSwapStrategy                      TransactionTemplate args=fromTokenMint,toTokenMint,swapSource, events=fundManagerUpdatedFund, description=add a new token swap strategy
      │   │   ├── addRestakingVault                         TransactionTemplate args=vault,pricingSource, events=fundManagerUpdatedFund, description=add a new restaking vault
      │   │   ├── addRestakingVaultDelegation               TransactionTemplate args=vault,operator, events=fundManagerUpdatedFund, description=add a new operator delegation to a restaking vault
      │   │   ├── updateRestakingVaultStrategy              TransactionTemplate (chained) args=null, events=fundManagerUpdatedFund, description=update restaking vault strategy of the fund
      │   │   ├── addRestakingVaultCompoundingReward        TransactionTemplate args=vault,rewardTokenMint, events=fundManagerUpdatedFund, description=add a new compounding reward to a restaking vault
      │   │   ├── removeRestakingVaultCompoundingReward     TransactionTemplate args=vault,rewardTokenMint, events=fundManagerUpdatedFund, description=remove a compounding reward from a restaking vault
      │   │   ├── addRestakingVaultDistributingReward       TransactionTemplate args=vault,rewardTokenMint, events=fundManagerUpdatedFund, description=add a new distributing reward to a restaking vault
      │   │   ├── removeRestakingVaultDistributingReward    TransactionTemplate args=vault,rewardTokenMint, events=fundManagerUpdatedFund, description=remove a distributing reward from a restaking vault
      │   │   ├── initializeNormalizedToken                 TransactionTemplate args=mint, events=fundManagerUpdatedFund, description=initialize normalized token pool account and enable
      │   │   ├── initializeWrappedToken                    TransactionTemplate args=mint, events=fundManagerUpdatedFund, description=enable wrapped token
      │   │   └── initializeWrappedTokenHolder              TransactionTemplate args=wrappedTokenAccount, events=userCreatedOrUpdatedRewardAccount,fundManagerUpdatedFund, description=add new wrapped token h
      │   ├── reward                                        RestakingRewardAccount address=undefined
      │   │   ├── reserve                                   RestakingRewardReserveAccount address=undefined, lamports=undefined
      │   │   │   └── rewardTokens                          IterativeAccount length=undefined, types=undefined
      │   │   ├── updatePools                               TransactionTemplate args=null, events=operatorUpdatedRewardPools, description=manually triggers contribution synchronization for the global reward
      │   │   ├── initializeOrUpdateAccount                 TransactionTemplate (chained) args=targetVersion, events=undefined, description=initialize or update reward account
      │   │   ├── addReward                                 TransactionTemplate args=mint,program,decimals,name,description, events=fundManagerUpdatedRewardPool, description=register a new reward (non-claim
      │   │   ├── updateReward                              TransactionTemplate args=mint,newMint,newProgram,newDecimals,claimable, events=fundManagerUpdatedRewardPool, description=update a non-claimable re
      │   │   └── settleReward                              TransactionTemplate args=isBonus,mint,amount, events=fundManagerUpdatedRewardPool, description=settle a reward
      │   ├── normalizedTokenMint                           RestakingNormalizedTokenMintAccount address=undefined, supply=undefined, decimals=undefined
      │   │   ├── metadata                                  FragmetricMetadata address=undefined
      │   │   └── initializeMint                            TransactionTemplate args=mint,name,symbol,uri,description,decimals, events=undefined, description=initialize normalized token mint
      │   ├── normalizedTokenPool                           RestakingNormalizedTokenPoolAccount address=undefined
      │   │   ├── supportedTokens                           IterativeAccount length=undefined, types=undefined
      │   │   ├── updatePrices                              TransactionTemplate args=null, events=operatorUpdatedNormalizedTokenPoolPrices, description=manually triggers price updates for the normalized tok
      │   │   └── addSupportedToken                         TransactionTemplate args=mint,program,pricingSource, events=operatorUpdatedNormalizedTokenPoolPrices, description=add a new supported token to the
      │   ├── wrappedTokenMint                              RestakingWrappedTokenMintAccount address=undefined, supply=undefined, decimals=undefined
      │   │   ├── metadata                                  FragmetricMetadata address=undefined
      │   │   └── initializeMint                            TransactionTemplate args=mint,name,symbol,uri,description,decimals, events=undefined, description=initialize wrapped token mint
      │   ├── payer                                         RestakingUserAccount address=undefined, lamports=undefined
      │   │   ├── fund                                      RestakingUserFundAccount address=undefined
      │   │   │   └── initializeOrUpdateAccount             TransactionTemplate args=null, events=userCreatedOrUpdatedFundAccount, description=initialize or update user fund account
      │   │   ├── reward                                    RestakingUserRewardAccount address=undefined
      │   │   │   ├── updatePools                           TransactionTemplate args=null, events=userUpdatedRewardPool, description=manually triggers contribution synchronization for the user reward pools
      │   │   │   ├── initializeOrUpdateAccount             TransactionTemplate args=null, events=userCreatedOrUpdatedRewardAccount, description=initialize or update user reward account
      │   │   │   ├── claim                                 TransactionTemplate args=delegate,isBonus,mint,amount,recipient, events=userClaimedReward, description=claim rewards
      │   │   │   ├── delegate                              TransactionTemplate args=delegate,newDelegate, events=userDelegatedRewardAccount, description=delegate user reward account
      │   │   │   └── resetDelegate                         TransactionTemplate args=null, events=userDelegatedRewardAccount, description=reset delegate of reward account (in case of either fund wrap or wra
      │   │   ├── receiptToken                              TokenAccount address=undefined, amount=undefined, mint=undefined
      │   │   ├── wrappedToken                              TokenAccount address=undefined, amount=undefined, mint=undefined
      │   │   ├── supportedTokens                           IterativeAccount length=undefined, types=undefined
      │   │   ├── rewardTokens                              IterativeAccount length=undefined, types=undefined
      │   │   ├── deposit                                   TransactionTemplate args=assetMint,assetAmount,metadata,applyPresetComputeUnitLimit, events=userDepositedToFund,userCreatedOrUpdatedFundAccount,us
      │   │   ├── requestWithdrawal                         TransactionTemplate args=assetMint,receiptTokenAmount, events=userRequestedWithdrawalFromFund,userCreatedOrUpdatedFundAccount,userCreatedOrUpdated
      │   │   ├── cancelWithdrawalRequest                   TransactionTemplate args=assetMint,requestId, events=userCanceledWithdrawalRequestFromFund,userCreatedOrUpdatedFundAccount,userCreatedOrUpdatedRew
      │   │   ├── withdraw                                  TransactionTemplate args=assetMint,requestId, events=userWithdrewFromFund,userCreatedOrUpdatedFundAccount,userCreatedOrUpdatedRewardAccount, descr
      │   │   ├── wrap                                      TransactionTemplate args=receiptTokenAmount,receiptTokenAmountAsTargetBalance, events=userWrappedReceiptToken, description=convert receipt tokens 
      │   │   ├── unwrap                                    TransactionTemplate args=wrappedTokenAmount, events=userUnwrappedReceiptToken, description=convert wrapped tokens back into receipt tokens
      │   │   └── transfer                                  TransactionTemplate args=receiptTokenAmount,recipient, events=userTransferredReceiptToken, description=transfer receipt token
      │   ├── initializeMint                                TransactionTemplate args=name,symbol,uri,description,decimals, events=undefined, description=initialize receipt token mint
      │   └── initializeOrUpdateExtraAccountMetaList        TransactionTemplate args=null, events=undefined, description=initialize or update extra account meta list
      └── parent                                            Runtime type=svm, cluster=devnet"
    `);
});

test('can marshal into JSON', async () => {
  const program = RestakingProgram.devnet(process.env.SOLANA_RPC_DEVNET);
  await program.fragSOL.fund.wrap.receiptToken.resolveAccount();
  expect(program.fragSOL.fund.wrap.receiptToken.toJSON()).toMatchObject({
    label: 'TokenAccount',
    mutable: false,
    properties: {
      address: 'CyT5oQnGkggbkDbfyanYpQwq3PDSGNgiEtZj7EWHVWsa',
      // amount: 19688502711n,
      mint: 'FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo',
    },
    unresolved: false,
    unused: false,
  });
});

test('can deduplicate requests and utilize cache', async () => {
  const program = RestakingProgram.devnet(process.env.SOLANA_RPC_DEVNET, {
    rpc: {
      accountCacheTTLSeconds: 2,
      accountDeduplicationIntervalSeconds: 1,
    },
  });

  // should deduplicate requests
  await expect(
    Promise.all(
      new Array(10)
        .fill(null)
        .map((_, index) =>
          (index % 2 == 0
            ? program.fragSOL
            : program.fragJTO
          ).resolveAccountTree()
        )
    )
  ).resolves.not.toThrowError();

  // should refetch accounts as cache is stale now
  await new Promise((resolve) => setTimeout(resolve, 2000));
  await expect(
    Promise.all(
      new Array(10)
        .fill(null)
        .map((_, index) =>
          (index % 2 == 0
            ? program.fragSOL
            : program.fragJTO
          ).resolveAccountTree()
        )
    )
  ).resolves.not.toThrowError();

  // should always refetch accounts
  await new Promise((resolve) => setTimeout(resolve, 2000));
  await expect(
    Promise.all(
      new Array(10)
        .fill(null)
        .map((_, index) =>
          (index % 2 == 0
            ? program.fragSOL
            : program.fragJTO
          ).resolveAccountTree(true)
        )
    )
  ).resolves.not.toThrowError();
});
