import fs from "fs";
import path from "path";

const vaultIndexOffset = 792;
const lastFeeChangeSlotOffset = 824;
const lastFullStateUpdateSlotOffset = 832;
const lastStartStateUpdateSlotOffset = 852;

/// npx tsx tests/mocks/reset_jito_vault_slots.ts [filepath] [vaultIndex]

if (process.argv.length >= 4) {
    const vaultFilePath = process.argv[2];
    const vaultIndex = Number(process.argv[3]);
    resetSlots(vaultFilePath, vaultIndex);
} else {
    throw "not enough arguments";
}

function resetSlots(vaultFilePath: string, vaultIndex: number) {
    // Load
    let vaultFileData = JSON.parse(fs.readFileSync(vaultFilePath, "utf8"));

    // Overwrite
    let vaultData = Buffer.from(vaultFileData.account.data[0], "base64");
    vaultData.writeBigUInt64LE(BigInt(vaultIndex), vaultIndexOffset);
    vaultData.writeBigUInt64LE(BigInt(0), lastFeeChangeSlotOffset);
    vaultData.writeBigUInt64LE(BigInt(0), lastFullStateUpdateSlotOffset);
    vaultData.writeBigUInt64LE(BigInt(0), lastStartStateUpdateSlotOffset);
    vaultFileData.account.data[0] = vaultData.toString("base64");

    // Save
    fs.writeFileSync(vaultFilePath, JSON.stringify(vaultFileData, null, 2));
}
