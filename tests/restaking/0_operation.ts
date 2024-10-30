// @ts-ignore
import * as anchor from "@coral-xyz/anchor";
import { step } from "mocha-steps";
import { Keychain } from "../../tools/lib";
import { restakingPlayground } from "../restaking";

function numberToBuffer(num) {
  const buffer = Buffer.allocUnsafe(8); // 8 바이트의 비할당 Buffer 생성
  buffer.writeBigUInt64LE(num);

  return buffer;
}

describe("operation", async () => {
  const restaking = await restakingPlayground;

  step("deposit to jito vault", async function () {
    const program_id = new anchor.web3.PublicKey("Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8");
    const config = new anchor.web3.PublicKey("Cx2tQmB4RdCQADK8dGzt9sbXDpQ9o2pcjuhKnN42NxbK");
    const vault_info = new anchor.web3.PublicKey("8bCy6TWfxc7H2ib61ijR1LzGynZNuVspdeUNra9AS9Lg");
    const signer = Keychain.readKeypairSecretFile("./new-keypair.json"); //TODO: change keypair path

    const config_account_info = await restaking.connection.getAccountInfo(config, "confirmed");
    const ncn_epoch_buffer = config_account_info.data.slice(72, 80);
    const epochLength = BigInt(ncn_epoch_buffer[0] + ncn_epoch_buffer[1] * 256 + ncn_epoch_buffer[2] * 65536 + ncn_epoch_buffer[3] * 16777216);
    const slot = await restaking.connection.getSlot();
    const ncn_epoch = BigInt(slot) / epochLength;

    const vault_update_state_tracker_pda = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("vault_update_state_tracker"), vault_info.toBuffer(), numberToBuffer(ncn_epoch)
    ], program_id);

    const accounts = restaking.methods.operatorRun().accounts({
      vaultUpdateStateTracker: vault_update_state_tracker_pda[0],
      user: signer.publicKey,
    })

    const tx = new anchor.web3.Transaction();
    tx.add(await accounts.instruction())
    await anchor.web3.sendAndConfirmTransaction(restaking.connection, tx, [
        signer,
    ]);
  });
});
