import fs from "fs";
import { web3 } from "@coral-xyz/anchor";

const adminOffset = 440;
const delegationAdminOffset = 472;
// const operatorAdminOffset = 504;
// const ncnAdminOffset = 536;
// const slasherAdminOffsete = 568;
// const capacityAdminOffset = 600;
// const feeAdmin = 632;
// const delegateAssetAdmin = 664;
// const feeWallet = 696;
// const mintBurnAdmin = 728;
// const metadataAdmin = 760;
const vaultIndexOffset = 792;
const operatorCountOffset = 808;
const lastFeeChangeSlotOffset = 824;
const lastFullStateUpdateSlotOffset = 832;
const lastStartStateUpdateSlotOffset = 852;

/// npx tsx tests/mocks/reset_jito_vault_slots.ts [filepath] [vaultIndex]

let ptr = 2;
const vaultFilePath = process.argv[ptr++];
const vaultIndex = Number(process.argv[ptr++]);
const operatorCount = Number(process.argv[ptr++]);
const vaultAdmin = new web3.PublicKey(process.argv[ptr++]);
const delegationAdmin = new web3.PublicKey(process.argv[ptr++]);
resetVault(vaultFilePath, vaultIndex, operatorCount, vaultAdmin, delegationAdmin);

function resetVault(
    vaultFilePath: string,
    vaultIndex: number,
    operatorCount: number,
    vaultAdmin: web3.PublicKey,
    delegationAdmin: web3.PublicKey,
) {
    // Load
    let vaultFileData = JSON.parse(fs.readFileSync(vaultFilePath, "utf8"));

    // Overwrite
    let vaultData = Buffer.from(vaultFileData.account.data[0], "base64");
    setVaultAdmin(vaultData, vaultAdmin);
    vaultData.fill(Uint8Array.from(delegationAdmin.toBuffer()), delegationAdminOffset, delegationAdminOffset + 32);
    vaultData.writeBigUInt64LE(BigInt(vaultIndex), vaultIndexOffset);
    vaultData.writeBigUInt64LE(BigInt(operatorCount), operatorCountOffset);
    vaultData.writeBigUInt64LE(BigInt(0), lastFeeChangeSlotOffset);
    vaultData.writeBigUInt64LE(BigInt(0), lastFullStateUpdateSlotOffset);
    vaultData.writeBigUInt64LE(BigInt(0), lastStartStateUpdateSlotOffset);
    
    // Save
    vaultFileData.account.data[0] = vaultData.toString("base64");
    fs.writeFileSync(vaultFilePath, JSON.stringify(vaultFileData, null, 2));
}

function setVaultAdmin(
    vaultData: Buffer,
    vaultAdmin: web3.PublicKey,
) {
    let oldAdmin = new web3.PublicKey(vaultData.subarray(adminOffset, adminOffset + 32));
    vaultData.fill(Uint8Array.from(vaultAdmin.toBuffer()), adminOffset, adminOffset + 32);
    for (let offset = delegationAdminOffset; offset < vaultIndexOffset; offset += 32) {
        let secondaryAdmin = new web3.PublicKey(vaultData.subarray(offset, offset + 32));
        if (secondaryAdmin.equals(oldAdmin)) {
            vaultData.fill(Uint8Array.from(vaultAdmin.toBuffer()), offset, offset + 32);
        }
    }
}
