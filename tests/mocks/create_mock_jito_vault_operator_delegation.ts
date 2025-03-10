import fs from "fs";
import { web3 } from "@coral-xyz/anchor";

if (process.argv.length >= 8) {
    const receiptToken = process.argv[2].toLowerCase();
    const vst = process.argv[3].toLowerCase();
    const vault = new web3.PublicKey(process.argv[4]);
    const fileIndex = Number(process.argv[5]);
    const operator = new web3.PublicKey(process.argv[6]);
    const index = Number(process.argv[7]);
    resetVaultOperatorDelegation(receiptToken, vst, fileIndex, vault, operator, index);
} else {
    throw "not enough arguments";
}

function resetVaultOperatorDelegation(
    receiptToken: string,
    vst: string,
    fileIndex: number,
    vault: web3.PublicKey,
    operator: web3.PublicKey,
    index: number,
) {
    const filePath = `tests/mocks/local/${receiptToken}_jito_${vst}_vault_operator_delegation_${fileIndex}.json`;
    const owner = "Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8"
    const lamports = 5289600;
    const [pubkey, bump] = web3.PublicKey.findProgramAddressSync(
        [Buffer.from("vault_operator_delegation"), vault.toBuffer(), operator.toBuffer()],
        new web3.PublicKey(owner),
    );
    const executable = false;
    const rentEpoch = 18446744073709551615;
    const space = 632;

    let buffer = Buffer.alloc(space);
    let offset = 0;

    // discriminator
    buffer.writeBigUInt64LE(BigInt(4), offset); offset += 8;

    // vault: Pubkey
    buffer.fill(Uint8Array.from(vault.toBuffer()), offset); offset += 32;

    // operator: Pubkey
    buffer.fill(Uint8Array.from(operator.toBuffer()), offset); offset += 32;
    // staked_amount: u64
    buffer.writeBigUInt64LE(BigInt(0), offset); offset += 8;
    // enqueued_for_cooldown_amount: u64
    buffer.writeBigUInt64LE(BigInt(0), offset); offset += 8;
    // cooling_down_amount: u64
    buffer.writeBigUInt64LE(BigInt(0), offset); offset += 8;
    // reserved: [u8; 256]
    offset += 256;
    // last_update_slot: u64
    buffer.writeBigUInt64LE(BigInt(0), offset); offset += 8;
    // index: u64
    buffer.writeBigUInt64LE(BigInt(index), offset); offset += 8;
    // bump: u8
    buffer.writeUInt8(bump, offset); offset += 1;

    // Create file
    const fileData = {
        pubkey,
        account: {
            lamports,
            data: [buffer.toString("base64"), "base64"],
            owner,
            executable,
            rentEpoch,
            space,
        }
    };
    fs.writeFileSync(filePath, JSON.stringify(fileData, null, 2));
}
