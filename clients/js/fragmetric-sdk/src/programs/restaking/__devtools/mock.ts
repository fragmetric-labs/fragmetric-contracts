import {
  AccountInfoBase,
  AccountInfoWithBase64EncodedData,
  AccountInfoWithPubkey,
  Address,
  Base64EncodedBytes,
  getBase64Decoder,
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
    switch (key) {
      case 'fragsol_jito_nsol_vault':
        const src = await jitoVault.fetchVault(
          program.runtime.rpc,
          'HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S' as Address
        );
        src.data.base =
          'AQ9mUxtF2SpiarjoipFKw7i2t3aUSVVantSSyuVYZDeq' as Address;
        src.data.vrtMint =
          'FRJtoBLuU72X3qgkVeBU1wXtmgQpWQmWptYsAdyyu3qT' as Address;
        src.data.operatorCount = 0n;
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
        break;
    }
    if (!account) {
      throw new Error(`invalid key: ${key}`);
    }
    console.log(JSON.stringify(account, null, 2));
  };
}
