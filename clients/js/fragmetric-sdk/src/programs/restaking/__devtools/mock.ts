import * as token from '@solana-program/token';
import {
  AccountInfoBase,
  AccountInfoWithBase64EncodedData,
  AccountInfoWithPubkey,
  Address,
  Base64EncodedBytes,
  getBase64Decoder,
  some,
} from '@solana/kit';
import { MAX_U64 } from '../../../context/constants';
import * as jitoVault from '../../../generated/jito_vault';
import { RestakingProgram } from '../program';

export function createMockTool(program: RestakingProgram) {
  return async (key: string) => {
    let account:
      | AccountInfoWithPubkey<
          AccountInfoBase & AccountInfoWithBase64EncodedData
        >
      | undefined = undefined;
    if (key == 'fragsol_jito_nsol_vrt_mint') {
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
    }
    if (!account) {
      throw new Error(`invalid key: ${key}`);
    }
    console.log(JSON.stringify(account, null, 2));
  };
}
