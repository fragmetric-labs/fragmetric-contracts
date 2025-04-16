import { address, Address, createNoopSigner, generateKeyPairSigner, isSome } from '@solana/kit';
import * as v from 'valibot';
import {
  TransactionTemplateContext,
  transformAddressResolverVariant,
} from '../../../context';
import { RestakingProgram } from '../program';
import * as token from '@solana-program/token';
import * as orca from '@orca-so/whirlpools-client';

export function createOrcaTool(program: RestakingProgram) {
  return {
    // e.g. create fake SOL/zBTC pool on devnet
    // restaking.__dev.orca.createPool.execute({
    //   mintA: 'FaKEZbaAE42h7aCSzzUMKP8woZYBXh43v5bPzqb8CyH',
    //   mintB: 'So11111111111111111111111111111111111111112',
    //   tickSpacing: 16,
    //   initialSqrtPrice: 1_506_170_346_543_264_150_413n,
    // })
    createPool: new TransactionTemplateContext(
      program,
      v.object({
        mintA: v.string(),
        mintB: v.string(),
        tickSpacing: v.number(),
        initialSqrtPrice: v.bigint(), // FLOOR(SQRT(tokenB/tokenA) * 2^64)
      }),
      {
        description:
          'initialize Orca WhirlPool',
        instructions: [
          async (parent, args, overrides) => {
            const [payer] = await Promise.all([
              transformAddressResolverVariant(
                overrides.feePayer ??
                program.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
              )(program),
            ]);
            if (!payer) throw new Error('invalid context');
            const admin = program.knownAddresses.admin;

            const whirlpoolConfigAddress = address(program.runtime.cluster == 'devnet' ? 'FcrweFY1G9HJAHG5inkGB6pKg1HZ6x9UC2WioAfWrGkR' : "2LecshUwdy9xi7meFgHtFJQNSKk4KdTrcpvaB56dP2NQ");
            const tokenMintA = address(args.mintA);
            const tokenMintB = address(args.mintB);
            const [tokenBadgeA] = await orca.getTokenBadgeAddress(whirlpoolConfigAddress, tokenMintA)
            const [tokenBadgeB] = await orca.getTokenBadgeAddress(whirlpoolConfigAddress, tokenMintB)
            const tickSpacing = args.tickSpacing;
            const [whirlpool] = await orca.getWhirlpoolAddress(whirlpoolConfigAddress, tokenMintA, tokenMintB, tickSpacing);
            const tokenVaultA = await generateKeyPairSigner();
            const tokenVaultB = await generateKeyPairSigner();
            const [feeTier] = await orca.getFeeTierAddress(whirlpoolConfigAddress, tickSpacing);
            const initialSqrtPrice = args.initialSqrtPrice;

            return Promise.all([
              orca.getInitializePoolV2Instruction({
                whirlpoolsConfig: whirlpoolConfigAddress,
                tokenMintA,
                tokenMintB,
                tokenBadgeA,
                tokenBadgeB,
                funder: createNoopSigner(admin),
                whirlpool,
                tokenVaultA,
                tokenVaultB,
                feeTier,
                tickSpacing,
                tokenProgramA: token.TOKEN_PROGRAM_ADDRESS,
                tokenProgramB: token.TOKEN_PROGRAM_ADDRESS,
                initialSqrtPrice
              }),
            ]);
          },
        ],
      }
    ),
  }
}
