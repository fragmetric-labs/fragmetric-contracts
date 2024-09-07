import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import {RestakingPlayground} from "../../tools/restaking/playground";
import {BN} from "@coral-xyz/anchor";

describe("transfer_hook", async () => {
    const playground = await RestakingPlayground.local(anchor.AnchorProvider.env());
    const user7 = playground.keychain.getKeypair('MOCK_USER7');
    const user8 = playground.keychain.getKeypair('MOCK_USER8');

    it("try airdrop SOL to mock accounts", async function () {
        await Promise.all([
            playground.tryAirdrop(user7.publicKey, 100),
            playground.tryAirdrop(user8.publicKey, 100),
        ]);

        await playground.sleep(1); // ...block hash not found?
    });

    const amountDeposited = new BN((10 ** 9) * 10);
    it("user7 deposits SOL to mint fragSOL", async () => {
        await playground.runUserDepositSOL(user7, amountDeposited, null);
    });

    it("transfer is temporarily disabled", async () => {
        await playground.run({
            instructions: [
                spl.createTransferCheckedWithTransferHookInstruction(
                    playground.connection,
                    playground.knownAddress.fragSOLUserTokenAccount(user7.publicKey),
                    playground.knownAddress.fragSOLTokenMint,
                    playground.knownAddress.fragSOLUserTokenAccount(user8.pub qlicKey),
                    user7.publicKey,
                    BigInt(amountDeposited.div(new BN(2)).toString()),
                    9,
                    [],
                    undefined,
                    spl.TOKEN_2022_PROGRAM_ID,
                ),
            ],
            signers: [user7],
        });
    });
});
