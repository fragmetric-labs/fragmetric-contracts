import {
  Address,
  Decoder,
  Encoder,
  fixDecoderSize,
  fixEncoderSize,
  getAddressDecoder,
  getAddressEncoder,
  getArrayDecoder,
  getBytesDecoder,
  getBytesEncoder,
  getDiscriminatedUnionDecoder,
  getDiscriminatedUnionEncoder,
  getOptionDecoder,
  getOptionEncoder,
  getProgramDerivedAddress,
  getStructDecoder,
  getStructEncoder,
  getU32Decoder,
  getU64Decoder,
  getU64Encoder,
  getU8Decoder,
  getU8Encoder,
  getUnitDecoder,
  getUnitEncoder,
  Option,
  ReadonlyUint8Array,
} from '@solana/kit';

// ref: https://github.com/solana-foundation/anchor/blob/master/ts/packages/spl-stake-pool/idl.json
// ref: https://github.com/solana-program/stake-pool/blob/main/program/src/state.rs
export type StakePool = {
  accountType: ReadonlyUint8Array;
  manager: Address;
  staker: Address;
  stakeDepositAuthority: Address;
  stakeWithdrawBumpSeed: number;
  validatorList: Address;
  reserveStake: Address;
  poolMint: Address;
  managerFeeAccount: Address;
  tokenProgramId: Address;
  totalLamports: bigint;
  poolTokenSupply: bigint;
  lastUpdateEpoch: bigint;
  lockUp: ReadonlyUint8Array;
  epochFee: ReadonlyUint8Array;
  nextEpochFee: FutureEpoch;
  preferredDepositValidatorVoteAddress: Option<Address>;
  preferredWithdrawValidatorVoteAddress: Option<Address>;
  stakeDepositFee: ReadonlyUint8Array;
  stakeWithdrawalFee: ReadonlyUint8Array;
  nextStakeWithdrawalFee: FutureEpoch;
  stakeReferralFee: number;
  solDepositAuthority: Option<Address>;
  solDepositFee: ReadonlyUint8Array;
  solReferralFee: number;
  solWithdrawAuthority: Option<Address>;
  solWithdrawalFee: ReadonlyUint8Array;
  nextSolWithdrawalFee: FutureEpoch;
  lastEpochPoolTokenSupply: bigint;
  lastEpochTotalLamports: bigint;
};

export type FutureEpoch =
  | { __kind: 'None' }
  | { __kind: 'One'; fields: ReadonlyUint8Array }
  | { __kind: 'Two'; fields: ReadonlyUint8Array };

const getFutureEpochDecoder = (): Decoder<FutureEpoch> =>
  getDiscriminatedUnionDecoder([
    ['None', getUnitDecoder()],
    [
      'One',
      getStructDecoder([['fields', fixDecoderSize(getBytesDecoder(), 16)]]),
    ],
    [
      'Two',
      getStructDecoder([['fields', fixDecoderSize(getBytesDecoder(), 16)]]),
    ],
  ]);

const getFutureEpochEncoder = (): Encoder<FutureEpoch> =>
  getDiscriminatedUnionEncoder([
    ['None', getUnitEncoder()],
    [
      'One',
      getStructEncoder([['fields', fixEncoderSize(getBytesEncoder(), 16)]]),
    ],
    [
      'Two',
      getStructEncoder([['fields', fixEncoderSize(getBytesEncoder(), 16)]]),
    ],
  ]);

export function getStakePoolDecoder(): Decoder<StakePool> {
  return getStructDecoder([
    ['accountType', fixDecoderSize(getBytesDecoder(), 1)],
    ['manager', getAddressDecoder()],
    ['staker', getAddressDecoder()],
    ['stakeDepositAuthority', getAddressDecoder()],
    ['stakeWithdrawBumpSeed', getU8Decoder()],
    ['validatorList', getAddressDecoder()],
    ['reserveStake', getAddressDecoder()],
    ['poolMint', getAddressDecoder()],
    ['managerFeeAccount', getAddressDecoder()],
    ['tokenProgramId', getAddressDecoder()],
    ['totalLamports', getU64Decoder()],
    ['poolTokenSupply', getU64Decoder()],
    ['lastUpdateEpoch', getU64Decoder()],
    ['lockUp', fixDecoderSize(getBytesDecoder(), 48)],
    ['epochFee', fixDecoderSize(getBytesDecoder(), 16)],
    ['nextEpochFee', getFutureEpochDecoder()],
    [
      'preferredDepositValidatorVoteAddress',
      getOptionDecoder(getAddressDecoder()),
    ],
    [
      'preferredWithdrawValidatorVoteAddress',
      getOptionDecoder(getAddressDecoder()),
    ],
    ['stakeDepositFee', fixDecoderSize(getBytesDecoder(), 16)],
    ['stakeWithdrawalFee', fixDecoderSize(getBytesDecoder(), 16)],
    ['nextStakeWithdrawalFee', getFutureEpochDecoder()],
    ['stakeReferralFee', getU8Decoder()],
    ['solDepositAuthority', getOptionDecoder(getAddressDecoder())],
    ['solDepositFee', fixDecoderSize(getBytesDecoder(), 16)],
    ['solReferralFee', getU8Decoder()],
    ['solWithdrawAuthority', getOptionDecoder(getAddressDecoder())],
    ['solWithdrawalFee', fixDecoderSize(getBytesDecoder(), 16)],
    ['nextSolWithdrawalFee', getFutureEpochDecoder()],
    ['lastEpochPoolTokenSupply', getU64Decoder()],
    ['lastEpochTotalLamports', getU64Decoder()],
  ]);
}

export function getStakePoolEncoder(): Encoder<StakePool> {
  return getStructEncoder([
    ['accountType', fixEncoderSize(getBytesEncoder(), 1)],
    ['manager', getAddressEncoder()],
    ['staker', getAddressEncoder()],
    ['stakeDepositAuthority', getAddressEncoder()],
    ['stakeWithdrawBumpSeed', getU8Encoder()],
    ['validatorList', getAddressEncoder()],
    ['reserveStake', getAddressEncoder()],
    ['poolMint', getAddressEncoder()],
    ['managerFeeAccount', getAddressEncoder()],
    ['tokenProgramId', getAddressEncoder()],
    ['totalLamports', getU64Encoder()],
    ['poolTokenSupply', getU64Encoder()],
    ['lastUpdateEpoch', getU64Encoder()],
    ['lockUp', fixEncoderSize(getBytesEncoder(), 48)],
    ['epochFee', fixEncoderSize(getBytesEncoder(), 16)],
    ['nextEpochFee', getFutureEpochEncoder()],
    [
      'preferredDepositValidatorVoteAddress',
      getOptionEncoder(getAddressEncoder()),
    ],
    [
      'preferredWithdrawValidatorVoteAddress',
      getOptionEncoder(getAddressEncoder()),
    ],
    ['stakeDepositFee', fixEncoderSize(getBytesEncoder(), 16)],
    ['stakeWithdrawalFee', fixEncoderSize(getBytesEncoder(), 16)],
    ['nextStakeWithdrawalFee', getFutureEpochEncoder()],
    ['stakeReferralFee', getU8Encoder()],
    ['solDepositAuthority', getOptionEncoder(getAddressEncoder())],
    ['solDepositFee', fixEncoderSize(getBytesEncoder(), 16)],
    ['solReferralFee', getU8Encoder()],
    ['solWithdrawAuthority', getOptionEncoder(getAddressEncoder())],
    ['solWithdrawalFee', fixEncoderSize(getBytesEncoder(), 16)],
    ['nextSolWithdrawalFee', getFutureEpochEncoder()],
    ['lastEpochPoolTokenSupply', getU64Encoder()],
    ['lastEpochTotalLamports', getU64Encoder()],
  ]);
}

export type ValidatorList = {
  accountType: ReadonlyUint8Array;
  maxValidators: number;
  validators: ValidatorStakeInfo[];
};

export function getValidatorListDecoder(): Decoder<ValidatorList> {
  return getStructDecoder([
    ['accountType', fixDecoderSize(getBytesDecoder(), 1)],
    ['maxValidators', getU32Decoder()],
    ['validators', getArrayDecoder(getValidatorStakeInfoDecoder())],
  ]);
}

export type ValidatorStakeInfo = {
  activeStakeLamports: bigint;
  transientStakeLamports: bigint;
  lastUpdateEpoch: bigint;
  transientSeedSuffix: ReadonlyUint8Array;
  unused: ReadonlyUint8Array;
  validatorSeedSuffix: ReadonlyUint8Array;
  status: ReadonlyUint8Array;
  voteAccountAddress: Address;
};

export function getValidatorStakeInfoDecoder(): Decoder<ValidatorStakeInfo> {
  return getStructDecoder([
    ['activeStakeLamports', getU64Decoder()],
    ['transientStakeLamports', getU64Decoder()],
    ['lastUpdateEpoch', getU64Decoder()],
    ['transientSeedSuffix', fixDecoderSize(getBytesDecoder(), 8)],
    ['unused', fixDecoderSize(getBytesDecoder(), 4)],
    ['validatorSeedSuffix', fixDecoderSize(getBytesDecoder(), 4)],
    ['status', fixDecoderSize(getBytesDecoder(), 1)],
    ['voteAccountAddress', getAddressDecoder()],
  ]);
}

export function getValidatorStakeAccountAddress(seeds: {
  program: Address;
  voteAccount: Address;
  pool: Address;
  validatorSeedSuffix: ReadonlyUint8Array;
}) {
  return getProgramDerivedAddress({
    programAddress: seeds.program,
    seeds: [
      getAddressEncoder().encode(seeds.voteAccount),
      getAddressEncoder().encode(seeds.pool),
      seeds.validatorSeedSuffix.every((v) => v == 0)
        ? Uint8Array.from([])
        : seeds.validatorSeedSuffix,
    ],
  });
}
