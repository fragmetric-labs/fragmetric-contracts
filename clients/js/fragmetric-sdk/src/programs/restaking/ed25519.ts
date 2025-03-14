import {
  address,
  getStructCodec,
  getU16Codec,
  getU8Codec,
  type IInstruction,
  type IInstructionWithData,
  ReadonlyUint8Array,
} from '@solana/kit';
import * as polyfill from '@solana/webcrypto-ed25519-polyfill';

const ed25519Layout = getStructCodec([
  ['numSignatures', getU8Codec()],
  ['padding', getU8Codec()],
  ['signatureOffset', getU16Codec()],
  ['signatureInstructionIndex', getU16Codec()],
  ['publicKeyOffset', getU16Codec()],
  ['publicKeyInstructionIndex', getU16Codec()],
  ['messageDataOffset', getU16Codec()],
  ['messageDataSize', getU16Codec()],
  ['messageInstructionIndex', getU16Codec()],
]);

export async function getEd25519Instruction(input: {
  publicKey: ReadonlyUint8Array;
  message: ReadonlyUint8Array;
  signature: ReadonlyUint8Array;
  instructionIndex?: number;
}): Promise<IInstruction & IInstructionWithData<ReadonlyUint8Array>> {
  const { publicKey, signature, message, instructionIndex = 0xffff } = input;

  const PUBKEY_OFFSET = 16;
  const SIG_OFFSET = PUBKEY_OFFSET + publicKey.length; // 48
  const MSG_OFFSET = SIG_OFFSET + signature.length; // 112

  const header = ed25519Layout.encode({
    numSignatures: 1,
    padding: 0,
    publicKeyOffset: PUBKEY_OFFSET, // length=32
    publicKeyInstructionIndex: instructionIndex,
    signatureOffset: SIG_OFFSET, // length=64
    signatureInstructionIndex: instructionIndex,
    messageDataOffset: MSG_OFFSET,
    messageDataSize: message.length,
    messageInstructionIndex: instructionIndex,
  });

  const result = new Uint8Array(
    ed25519Layout.fixedSize +
      signature.length +
      publicKey.length +
      message.length
  );
  result.set(header, 0);
  result.set(publicKey, PUBKEY_OFFSET);
  result.set(signature, SIG_OFFSET);
  result.set(message, MSG_OFFSET);

  return {
    programAddress: address('Ed25519SigVerify111111111111111111111111111'),
    accounts: [],
    data: result,
  };
}

let polyfillInstalled = false;

export async function signMessageWithEd25519Keypair(
  keypair: CryptoKeyPair,
  data: ReadonlyUint8Array
) {
  if (!polyfillInstalled) {
    polyfill.install();
    polyfillInstalled = true;
  }
  return {
    publicKey: new Uint8Array(
      await crypto.subtle.exportKey('raw', keypair.publicKey)
    ),
    signature: new Uint8Array(
      await crypto.subtle.sign('Ed25519', keypair.privateKey, data)
    ),
  };
}
