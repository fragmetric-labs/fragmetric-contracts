import fs from "fs";
import { web3 } from "@coral-xyz/anchor";

const delegationAdminOffset = 472;
const vaultIndexOffset = 792;
const operatorCountOffset = 808;
const lastFeeChangeSlotOffset = 824;
const lastFullStateUpdateSlotOffset = 832;
const lastStartStateUpdateSlotOffset = 852;

/// npx tsx tests/mocks/reset_jito_vault_slots.ts [filepath] [vaultIndex]

if (process.argv.length >= 6) {
    const vaultFilePath = process.argv[2];
    const vaultIndex = Number(process.argv[3]);
    const operatorCount = Number(process.argv[4]);
    const delegationAdmin = new web3.PublicKey(process.argv[5]);
    resetVault(vaultFilePath, vaultIndex, operatorCount, delegationAdmin);
} else {
    throw "not enough arguments";
}

function resetVault(
    vaultFilePath: string,
    vaultIndex: number,
    operatorCount: number,
    delegationAdmin: web3.PublicKey,
) {
    // Load
    let vaultFileData = JSON.parse(fs.readFileSync(vaultFilePath, "utf8"));

    // Overwrite
    let vaultData = Buffer.from(vaultFileData.account.data[0], "base64");
    let ptr = delegationAdminOffset;
    for(let byte of delegationAdmin.toBuffer()) {
        vaultData.writeUInt8(byte, ptr++);
    }
    vaultData.writeBigUInt64LE(BigInt(vaultIndex), vaultIndexOffset);
    vaultData.writeBigUInt64LE(BigInt(operatorCount), operatorCountOffset);
    vaultData.writeBigUInt64LE(BigInt(0), lastFeeChangeSlotOffset);
    vaultData.writeBigUInt64LE(BigInt(0), lastFullStateUpdateSlotOffset);
    vaultData.writeBigUInt64LE(BigInt(0), lastStartStateUpdateSlotOffset);
    vaultFileData.account.data[0] = vaultData.toString("base64");

    // Save
    fs.writeFileSync(vaultFilePath, JSON.stringify(vaultFileData, null, 2));
}
