import * as orca from '@orca-so/whirlpools-client';
import * as token from '@solana-program/token';
import {
  AccountInfoBase,
  AccountInfoWithBase64EncodedData,
  AccountInfoWithPubkey,
  Address,
  Base64EncodedBytes,
  getBase64Decoder,
  Lamports,
  some,
} from '@solana/kit';
import { MAX_U64 } from '../../../context/constants';
import * as jitoVault from '../../../generated/jito_vault';
import { RestakingProgram } from '../program';
import {
  getStakePoolDecoder,
  getValidatorListDecoder,
  getValidatorStakeAccountAddress,
} from './mock.stake_pool';

export function createMockTool(program: RestakingProgram) {
  const localFundManager =
    '5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx' as Address;

  return async (key: string, ...args: string[]) => {
    let account:
      | AccountInfoWithPubkey<
          AccountInfoBase & AccountInfoWithBase64EncodedData
        >
      | undefined = undefined;
    if (key == 'frag') {
      account = {
        pubkey: 'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5' as Address,
        account: {
          owner: token.TOKEN_PROGRAM_ADDRESS,
          executable: false,
          lamports: 1461600n as Lamports,
          rentEpoch: MAX_U64,
          space: 82n,
          data: [
            getBase64Decoder().decode(
              token.getMintEncoder().encode({
                mintAuthority: null,
                supply: 1_000_000_000_000_000_000n,
                decimals: 9,
                isInitialized: true,
                freezeAuthority: null,
              })
            ) as Base64EncodedBytes,
            'base64',
          ],
        },
      };
    } else if (key == 'fragvote') {
      account = {
        pubkey: 'FRAGV56ChY2z2EuWmVquTtgDBdyKPBLEBpXx4U9SKTaF' as Address,
        account: {
          owner: token.TOKEN_PROGRAM_ADDRESS,
          executable: false,
          lamports: 1461600n as Lamports,
          rentEpoch: MAX_U64,
          space: 82n,
          data: [
            getBase64Decoder().decode(
              token.getMintEncoder().encode({
                mintAuthority: localFundManager,
                supply: 1_000_000_000_000_000_000n,
                decimals: 9,
                isInitialized: true,
                freezeAuthority: null,
              })
            ) as Base64EncodedBytes,
            'base64',
          ],
        },
      };
    } else if (key == 'frag_orca_dex_liqudity_pool') {
      const tokenMintA =
        'So11111111111111111111111111111111111111112' as Address;
      const tokenMintB =
        'FRAGMEWj2z65qM62zqKhNtwNFskdfKs4ekDUDX3b4VD5' as Address;
      const src = await orca.fetchWhirlpool(
        program.runtime.rpc,
        'Hp53XEtt4S8SvPCXarsLSdGfZBuUr5mMmZmX2DRNXQKp' as Address // SOL/JitoSOL
      );

      const [whirlpool] = await orca.getWhirlpoolAddress(
        src.data.whirlpoolsConfig,
        tokenMintA,
        tokenMintB,
        src.data.tickSpacing
      );
      account = {
        pubkey: whirlpool,
        account: {
          owner: src.programAddress,
          executable: src.executable,
          lamports: src.lamports,
          rentEpoch: MAX_U64,
          space: src.space,
          data: [
            getBase64Decoder().decode(
              orca.getWhirlpoolEncoder().encode({
                ...src.data,
                tokenMintA,
                tokenMintB,
              })
            ) as Base64EncodedBytes,
            'base64',
          ],
        },
      };
    } else if (key == 'jito_vault_config_x100') {
      const cfg = await jitoVault.fetchConfig(
        program.runtime.rpc,
        'UwuSgAq4zByffCGCrWH87DsjfsewYjuqHfJEpzw1Jq3' as Address
      );
      cfg.data.epochLength = 4320n; // 432000 by default
      account = {
        pubkey: cfg.address,
        account: {
          owner: cfg.programAddress,
          executable: cfg.executable,
          lamports: cfg.lamports,
          rentEpoch: MAX_U64,
          space: cfg.space,
          data: [
            getBase64Decoder().decode(
              jitoVault.getConfigCodec().encode(cfg.data)
            ) as Base64EncodedBytes,
            'base64',
          ],
        },
      };
    } else if (key == 'fragsol_jito_nsol_vrt_mint') {
      const src = await token.fetchMint(
        program.runtime.rpc,
        'CkXLPfDG3cDawtUvnztq99HdGoQWhJceBZxqKYL2TUrg' as Address
      );
      src.data.supply = 0n;
      src.data.mintAuthority = some(
        'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S' as Address
      );
      account = {
        pubkey: src.address,
        account: {
          owner: src.programAddress,
          executable: src.executable,
          lamports: src.lamports,
          rentEpoch: MAX_U64,
          space: src.space,
          data: [
            getBase64Decoder().decode(
              token.getMintEncoder().encode(src.data)
            ) as Base64EncodedBytes,
            'base64',
          ],
        },
      };
    } else if (key == 'fragsol_jito_nsol_vault') {
      const src = await jitoVault.fetchVault(
        program.runtime.rpc,
        'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S' as Address
      );
      const localFundManager =
        '5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx' as Address;
      Object.assign(src.data, {
        ...src.data,
        additionalAssetsNeedUnstaking: 0n,
        admin: localFundManager,
        capacityAdmin: localFundManager,
        delegateAssetAdmin: localFundManager,
        delegationAdmin: localFundManager,
        delegationState: {
          ...src.data.delegationState,
          stakedAmount: 0n,
          enqueuedForCooldownAmount: 0n,
          coolingDownAmount: 0n,
        },
        depositFeeBps: 0,
        feeAdmin: localFundManager,
        feeWallet: localFundManager,
        isPaused: 0,
        lastFeeChangeSlot: 0n,
        lastFullStateUpdateSlot: 0n,
        lastStartStateUpdateSlot: 0n,
        metadataAdmin: localFundManager,
        // mintBurnAdmin: localFundManager,
        ncnAdmin: localFundManager,
        ncnCount: 0n,
        nextWithdrawalFeeBps: 0,
        operatorAdmin: localFundManager,
        operatorCount: 0n,
        programFeeBps: 10,
        rewardFeeBps: 0,
        slasherAdmin: localFundManager,
        slasherCount: 0n,
        supportedMint: '4noNmx2RpxK4zdr68Fq1CYM5VhN4yjgGZEFyuB7t2pBX', // local nSOL
        tokensDeposited: 0n,
        vaultIndex: 0n,
        vrtCoolingDownAmount: 0n,
        vrtEnqueuedForCooldownAmount: 0n,
        vrtMint: 'CkXLPfDG3cDawtUvnztq99HdGoQWhJceBZxqKYL2TUrg',
        vrtReadyToClaimAmount: 0n,
        vrtSupply: 0n,
        withdrawalFeeBps: 0,
      });
      account = {
        pubkey: src.address,
        account: {
          owner: src.programAddress,
          executable: src.executable,
          lamports: src.lamports,
          rentEpoch: MAX_U64,
          space: src.space,
          data: [
            getBase64Decoder().decode(
              jitoVault.getVaultEncoder().encode(src.data)
            ) as Base64EncodedBytes,
            'base64',
          ],
        },
      };
    } else if (key == 'fragjto_jito_jto_vrt_mint') {
      const src = await token.fetchMint(
        program.runtime.rpc,
        'FRJtoBLuU72X3qgkVeBU1wXtmgQpWQmWptYsAdyyu3qT' as Address
      );
      src.data.supply = 0n;
      src.data.mintAuthority = some(
        'BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd' as Address
      );
      account = {
        pubkey: src.address,
        account: {
          owner: src.programAddress,
          executable: src.executable,
          lamports: src.lamports,
          rentEpoch: MAX_U64,
          space: src.space,
          data: [
            getBase64Decoder().decode(
              token.getMintEncoder().encode(src.data)
            ) as Base64EncodedBytes,
            'base64',
          ],
        },
      };
    } else if (key == 'fragjto_jito_jto_vault') {
      const src = await jitoVault.fetchVault(
        program.runtime.rpc,
        'BmJvUzoiiNBRx3v2Gqsix9WvVtw8FaztrfBHQyqpMbTd' as Address
      );
      Object.assign(src.data, {
        ...src.data,
        additionalAssetsNeedUnstaking: 0n,
        admin: localFundManager,
        capacityAdmin: localFundManager,
        delegateAssetAdmin: localFundManager,
        delegationAdmin: localFundManager,
        delegationState: {
          ...src.data.delegationState,
          stakedAmount: 0n,
          enqueuedForCooldownAmount: 0n,
          coolingDownAmount: 0n,
        },
        depositFeeBps: 0,
        feeAdmin: localFundManager,
        feeWallet: localFundManager,
        isPaused: 0,
        lastFeeChangeSlot: 0n,
        lastFullStateUpdateSlot: 0n,
        lastStartStateUpdateSlot: 0n,
        metadataAdmin: localFundManager,
        // mintBurnAdmin: localFundManager,
        ncnAdmin: localFundManager,
        ncnCount: 0n,
        nextWithdrawalFeeBps: 0,
        operatorAdmin: localFundManager,
        operatorCount: 0n,
        programFeeBps: 10,
        rewardFeeBps: 0,
        slasherAdmin: localFundManager,
        slasherCount: 0n,
        supportedMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL', // local JTO (=mainnet)
        tokensDeposited: 0n,
        vaultIndex: 0n,
        vrtCoolingDownAmount: 0n,
        vrtEnqueuedForCooldownAmount: 0n,
        vrtMint: 'FRJtoBLuU72X3qgkVeBU1wXtmgQpWQmWptYsAdyyu3qT',
        vrtReadyToClaimAmount: 0n,
        vrtSupply: 0n,
        withdrawalFeeBps: 0,
      });
      account = {
        pubkey: src.address,
        account: {
          owner: src.programAddress,
          executable: src.executable,
          lamports: src.lamports,
          rentEpoch: MAX_U64,
          space: src.space,
          data: [
            getBase64Decoder().decode(
              jitoVault.getVaultEncoder().encode(src.data)
            ) as Base64EncodedBytes,
            'base64',
          ],
        },
      };
    } else if (key == 'lst') {
      const poolAddress = args[0];
      if (!poolAddress) {
        console.debug(
          `Usage: pnpm connect -u m -e "restaking.__dev.mock('lst', 'Hr9pzexrBge3vgmBNRR8u42CNQgBXdHm4UkUN2DH4a7r', 'BNSOL')"`
        );

        throw new Error(`invalid LST pool address: ${poolAddress}`);
      }
      const symbol = args[1];
      if (!symbol) {
        throw new Error(`invalid LST symbol: ${symbol}`);
      }

      const poolSrc = await program.runtime.fetchAccount(poolAddress, true);
      if (!poolSrc) {
        throw new Error(`invalid LST pool: ${poolAddress}`);
      }
      const pool = getStakePoolDecoder().decode(poolSrc.data);

      const validatorListSrc = await program.runtime.fetchAccount(
        pool.validatorList,
        false
      );
      if (!validatorListSrc) {
        throw new Error(`invalid LST validatorList: ${pool.validatorList}`);
      }

      const validatorList = getValidatorListDecoder().decode(
        validatorListSrc.data
      );
      validatorList.validators.sort((a, b) => {
        if (b.activeStakeLamports > a.activeStakeLamports) {
          return 1;
        } else if (b.activeStakeLamports < a.activeStakeLamports) {
          return -1;
        } else {
          return 0;
        }
      });
      const validatorStakeAccountsTop5 = await Promise.all(
        validatorList.validators.slice(0, 5).map((v) =>
          getValidatorStakeAccountAddress({
            program: poolSrc.programAddress,
            voteAccount: v.voteAccountAddress,
            pool: poolSrc.address,
            validatorSeedSuffix: v.validatorSeedSuffix,
          }).then(([address, _]) => address)
        )
      );

      console.log(`
# RUN BELOW COMMANDS
solana -u m account ${pool.poolMint} --output json --output-file ./programs/restaking/tests/mocks/${symbol}_mint.json
solana -u m account ${poolAddress} --output json --output-file ./programs/restaking/tests/mocks/${symbol}_stake_pool.json
solana -u m account ${pool.managerFeeAccount} --output json --output-file ./programs/restaking/tests/mocks/${symbol}_stake_pool_manager_fee.json
solana -u m account ${pool.reserveStake} --output json --output-file ./programs/restaking/tests/mocks/${symbol}_stake_pool_reserve_stake.json
solana -u m account ${pool.validatorList} --output json --output-file ./programs/restaking/tests/mocks/${symbol}_stake_pool_validator_list.json
${validatorStakeAccountsTop5
  .map(
    (address, i) =>
      `solana -u m account ${address} --output json --output-file ./programs/restaking/tests/mocks/${symbol}_stake_pool_validator_stake_${i + 1}.json`
  )
  .join('\n')}
      `);
      return;
    }
    if (!account) {
      throw new Error(`invalid key: ${key}`);
    }
    console.log(JSON.stringify(account, null, 2));
  };
}
