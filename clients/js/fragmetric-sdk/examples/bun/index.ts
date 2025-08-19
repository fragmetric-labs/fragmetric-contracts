import { RestakingProgram } from '@fragmetric-labs/sdk';
import { createKeyPairSignerFromBytes } from '@solana/kit';
import * as util from 'node:util';

(async function example1() {
  const walletKeyPair = await createKeyPairSignerFromBytes(
    Uint8Array.from([
      18, 99, 108, 102, 2, 206, 6, 7, 168, 174, 190, 163, 59, 172, 204, 141,
      105, 14, 181, 146, 108, 161, 134, 128, 169, 57, 152, 205, 238, 237, 220,
      216, 150, 75, 239, 172, 33, 80, 166, 64, 55, 49, 182, 185, 30, 49, 104,
      33, 14, 163, 68, 64, 59, 209, 64, 244, 34, 15, 83, 110, 17, 139, 78, 4,
    ])
  );
  // const restaking = RestakingProgram.devnet(null, {
  //   transaction: {
  //     executionHooks: createDefaultTransactionExecutionHooks(),
  //   },
  // });
  // await restaking.fragSOL.user(walletKeyPair).deposit.execute(
  //   {
  //     assetAmount: 10n,
  //   },
  //   {
  //     feePayer: walletKeyPair,
  //   }
  // );

  // TODO: ...
  const tx = await RestakingProgram.mainnet(/* https://rpc */)
    .fragSOL.user('B7hKA5KCnPb465QCJMHwfGfyfzemQ3Nvvxsa9K7cQTUw')
    .deposit.assemble(
      {
        assetMint: null,
        assetAmount: 1000_000_000n,
        applyPresetComputeUnitLimit: false,
      },
      {
        recentBlockhash: null, // to skip fetching blockhash
      }
    );
  console.log(util.inspect(tx, false, 10, true));
})();
