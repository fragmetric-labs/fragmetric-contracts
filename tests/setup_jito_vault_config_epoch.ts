import fs from "fs";
import path from "path";

/// File running command: npx tsx tests/setup_jito_vault_config_epoch.ts

const DISCRIMINATOR_LENGTH = 8;
const JitoVaultConfigType = {
    admin: 32, // Pubkey
    restakingProgram: 32, // Pubkey
    epochLength: 8, // PodU64
    // ...
}

const parsePod = (podSize: number, buffer: Uint8Array, startOffset: number) => {
    const dataView = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
    switch (podSize) {
        case 16:
            return dataView.getUint16(startOffset, true);
        case 64:
            return dataView.getBigUint64(startOffset, true); // `true` for little-endian
    }
}

const convertNumberToUint8Array = (arraySize: number, value: number) => {
    const uint8Array = new Uint8Array(arraySize);
    for (let i = 0; i < arraySize; i++) {
        uint8Array[i] = value % 256;
        value = Math.floor(value / 256);
    }
    return uint8Array;
}

const setupJitoVaultConfigEpoch = () => {
    const TARGET_FILE_PREFIX = "jito_vault_config_epoch_length_";
    const EPOCH_LENGTHS = [32, 64, 128, 256, 432000];

    // 1. copy epoch_length_256 file to other epoch length files
    EPOCH_LENGTHS.map((epoch) => {
        if (epoch == 256) return;

        fs.copyFileSync(path.join(__dirname, `./mocks/mainnet/${TARGET_FILE_PREFIX}256.json`), path.join(__dirname, `./mocks/mainnet/${TARGET_FILE_PREFIX}${epoch}.json`));
    });

    // 2. setup epoch length
    fs.readdirSync(path.join(__dirname, "./mocks/mainnet/")).forEach(filename => {
        if (!filename.startsWith(TARGET_FILE_PREFIX)) return;

        const targetEpochLength = filename.slice(TARGET_FILE_PREFIX.length).split(".")[0];

        const jitoVaultConfigRaw = fs.readFileSync(path.join(__dirname, `./mocks/mainnet/${filename}`), {encoding: "utf-8"});
        const jitoVaultConfig = JSON.parse(jitoVaultConfigRaw);
        const data = Uint8Array.from(Buffer.from(jitoVaultConfig["account"]["data"][0], "base64"));

        let targetOffset = DISCRIMINATOR_LENGTH + JitoVaultConfigType.admin;
        targetOffset = targetOffset + JitoVaultConfigType.restakingProgram;

        let epochData = parsePod(64, data, targetOffset);
        console.log(`[Before] ${filename} epoch length ${epochData}`);

        const newEpochLength = convertNumberToUint8Array(JitoVaultConfigType.epochLength, Number(targetEpochLength));

        for (let i = 0; i < newEpochLength.length; i++) {
            data[targetOffset + i] = newEpochLength[i];
        }

        epochData = parsePod(64, data, targetOffset);
        console.log(`[After] ${filename} epoch length ${epochData}`);

        jitoVaultConfig["account"]["data"][0] = Buffer.from(data).toString("base64");

        fs.writeFileSync(path.join(__dirname, `./mocks/mainnet/${filename}`), JSON.stringify(jitoVaultConfig, null, 2));
    });
}
setupJitoVaultConfigEpoch();
