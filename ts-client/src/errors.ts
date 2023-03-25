import {
    FnkBorshReader,
    FnkBorshSchema,
    FnkBorshWriter,
    TEnum,
    TPublicKey,
    TString,
    TStruct,
    U64,
} from './serde';
import { clone, equals } from './utils';
import { PublicKey } from '@solana/web3.js';
import BN from 'bn.js';

export class FankorErrorCode {
    // CONSTRUCTORS -----------------------------------------------------------

    constructor(public data: FankorErrorCodeTypes) {}

    // GETTERS ----------------------------------------------------------------

    static get schema() {
        return TFankorErrorCode;
    }

    // METHODS ----------------------------------------------------------------

    serialize(buffer?: Buffer) {
        const writer = new FnkBorshWriter(buffer);
        TFankorErrorCode.serialize(writer, this);
        return writer.toBuffer();
    }

    equals(other: FankorErrorCode) {
        return (
            this.data.type === other.data.type &&
            equals((this.data as any)?.value, (other.data as any)?.value)
        );
    }

    clone(): FankorErrorCode {
        return new FankorErrorCode(clone(this.data));
    }

    // STATIC METHODS ---------------------------------------------------------

    static deserialize(buffer: Buffer, offset?: number) {
        const reader = new FnkBorshReader(buffer, offset);
        return TFankorErrorCode.deserialize(reader);
    }
}

export type FankorErrorCodeTypes =
    | FankorErrorCode_DeclaredProgramIdMismatch
    | FankorErrorCode_MissingInstructionDiscriminant
    | FankorErrorCode_InstructionDiscriminantNotFound
    | FankorErrorCode_UnusedAccounts
    | FankorErrorCode_MissingProgram
    | FankorErrorCode_CannotFindValidPdaWithProvidedSeeds
    | FankorErrorCode_InvalidPda
    | FankorErrorCode_MissingSeedsAccount
    | FankorErrorCode_MissingPdaSeeds
    | FankorErrorCode_DuplicatedWritableAccounts
    | FankorErrorCode_AccountDiscriminantMismatch
    | FankorErrorCode_InstructionDidNotDeserialize
    | FankorErrorCode_AccountNotOwnedByProgram
    | FankorErrorCode_ReadonlyAccountModification
    | FankorErrorCode_MutRefToReadonlyAccount
    | FankorErrorCode_NewFromClosedAccount
    | FankorErrorCode_AccountNotRentExempt
    | FankorErrorCode_AccountNotInitialized
    | FankorErrorCode_AccountAlreadyInitialized
    | FankorErrorCode_AccountOwnedByWrongProgram
    | FankorErrorCode_IncorrectSysvarAccount
    | FankorErrorCode_AlreadyClosedAccount
    | FankorErrorCode_InvalidProgram
    | FankorErrorCode_ProgramIsNotExecutable
    | FankorErrorCode_NotEnoughAccountKeys
    | FankorErrorCode_NotEnoughDataToDeserializeInstruction
    | FankorErrorCode_NotAccountsExpected
    | FankorErrorCode_NotEnoughValidAccountForVec
    | FankorErrorCode_AccountConstraintOwnerMismatch
    | FankorErrorCode_AccountConstraintAddressMismatch
    | FankorErrorCode_AccountConstraintNotInitialized
    | FankorErrorCode_AccountConstraintInitialized
    | FankorErrorCode_AccountConstraintNotWritable
    | FankorErrorCode_AccountConstraintWritable
    | FankorErrorCode_AccountConstraintNotExecutable
    | FankorErrorCode_AccountConstraintExecutable
    | FankorErrorCode_AccountConstraintNotRentExempt
    | FankorErrorCode_AccountConstraintRentExempt
    | FankorErrorCode_AccountConstraintNotSigner
    | FankorErrorCode_AccountConstraintSigner
    | FankorErrorCode_AccountConstraintMinimumMismatch
    | FankorErrorCode_AccountConstraintMaximumMismatch
    | FankorErrorCode_AccountConstraintFailed
    | FankorErrorCode_DuplicatedAccountWithDifferentType
    | FankorErrorCode_AccountNotDefault
    | FankorErrorCode_EmptyIntermediateBuffer
    | FankorErrorCode_IntermediateBufferIncorrectProgramId
    | FankorErrorCode_TooManyAccounts
    | FankorErrorCode_ZeroCopyCannotDeserialize
    | FankorErrorCode_ZeroCopyNotEnoughLength
    | FankorErrorCode_ZeroCopyInvalidEnumDiscriminant
    | FankorErrorCode_ZeroCopyPossibleDeadlock
    | FankorErrorCode_ZeroCopyLengthFieldOverflow
    | FankorErrorCode_ZeroCopyIncorrectPrecedingField
    | FankorErrorCode_ZeroCopyInvalidMove;

export interface FankorErrorCode_DeclaredProgramIdMismatch {
    type: 'DeclaredProgramIdMismatch';
}

export interface FankorErrorCode_MissingInstructionDiscriminant {
    type: 'MissingInstructionDiscriminant';
}

export interface FankorErrorCode_InstructionDiscriminantNotFound {
    type: 'InstructionDiscriminantNotFound';
}

export interface FankorErrorCode_UnusedAccounts {
    type: 'UnusedAccounts';
}

export interface FankorErrorCode_MissingProgram {
    type: 'MissingProgram';
    value: { address: PublicKey; name: string };
}

export interface FankorErrorCode_CannotFindValidPdaWithProvidedSeeds {
    type: 'CannotFindValidPdaWithProvidedSeeds';
    value: { programId: PublicKey };
}

export interface FankorErrorCode_InvalidPda {
    type: 'InvalidPda';
    value: { expected: PublicKey; actual: PublicKey };
}

export interface FankorErrorCode_MissingSeedsAccount {
    type: 'MissingSeedsAccount';
}

export interface FankorErrorCode_MissingPdaSeeds {
    type: 'MissingPdaSeeds';
    value: { account: PublicKey };
}

export interface FankorErrorCode_DuplicatedWritableAccounts {
    type: 'DuplicatedWritableAccounts';
    value: { address: PublicKey };
}

export interface FankorErrorCode_AccountDiscriminantMismatch {
    type: 'AccountDiscriminantMismatch';
    value: { account: string };
}

export interface FankorErrorCode_InstructionDidNotDeserialize {
    type: 'InstructionDidNotDeserialize';
    value: { account: string };
}

export interface FankorErrorCode_AccountNotOwnedByProgram {
    type: 'AccountNotOwnedByProgram';
    value: { address: PublicKey; action: string };
}

export interface FankorErrorCode_ReadonlyAccountModification {
    type: 'ReadonlyAccountModification';
    value: { address: PublicKey; action: string };
}

export interface FankorErrorCode_MutRefToReadonlyAccount {
    type: 'MutRefToReadonlyAccount';
    value: { address: PublicKey };
}

export interface FankorErrorCode_NewFromClosedAccount {
    type: 'NewFromClosedAccount';
    value: { address: PublicKey };
}

export interface FankorErrorCode_AccountNotRentExempt {
    type: 'AccountNotRentExempt';
    value: { account: PublicKey };
}

export interface FankorErrorCode_AccountNotInitialized {
    type: 'AccountNotInitialized';
    value: { address: PublicKey };
}

export interface FankorErrorCode_AccountAlreadyInitialized {
    type: 'AccountAlreadyInitialized';
    value: { address: PublicKey };
}

export interface FankorErrorCode_AccountOwnedByWrongProgram {
    type: 'AccountOwnedByWrongProgram';
    value: { address: PublicKey; expected: PublicKey; actual: PublicKey };
}

export interface FankorErrorCode_IncorrectSysvarAccount {
    type: 'IncorrectSysvarAccount';
    value: { actual: PublicKey; expected: PublicKey };
}

export interface FankorErrorCode_AlreadyClosedAccount {
    type: 'AlreadyClosedAccount';
    value: { address: PublicKey; action: string };
}

export interface FankorErrorCode_InvalidProgram {
    type: 'InvalidProgram';
    value: { expected: PublicKey; actual: PublicKey };
}

export interface FankorErrorCode_ProgramIsNotExecutable {
    type: 'ProgramIsNotExecutable';
    value: { program: PublicKey };
}

export interface FankorErrorCode_NotEnoughAccountKeys {
    type: 'NotEnoughAccountKeys';
}

export interface FankorErrorCode_NotEnoughDataToDeserializeInstruction {
    type: 'NotEnoughDataToDeserializeInstruction';
}

export interface FankorErrorCode_NotAccountsExpected {
    type: 'NotAccountsExpected';
}

export interface FankorErrorCode_NotEnoughValidAccountForVec {
    type: 'NotEnoughValidAccountForVec';
}

export interface FankorErrorCode_AccountConstraintOwnerMismatch {
    type: 'AccountConstraintOwnerMismatch';
    value: { actual: PublicKey; expected: PublicKey; account: string };
}

export interface FankorErrorCode_AccountConstraintAddressMismatch {
    type: 'AccountConstraintAddressMismatch';
    value: { actual: PublicKey; expected: PublicKey; account: string };
}

export interface FankorErrorCode_AccountConstraintNotInitialized {
    type: 'AccountConstraintNotInitialized';
    value: { account: string };
}

export interface FankorErrorCode_AccountConstraintInitialized {
    type: 'AccountConstraintInitialized';
    value: { account: string };
}

export interface FankorErrorCode_AccountConstraintNotWritable {
    type: 'AccountConstraintNotWritable';
    value: { account: string };
}

export interface FankorErrorCode_AccountConstraintWritable {
    type: 'AccountConstraintWritable';
    value: { account: string };
}

export interface FankorErrorCode_AccountConstraintNotExecutable {
    type: 'AccountConstraintNotExecutable';
    value: { account: string };
}

export interface FankorErrorCode_AccountConstraintExecutable {
    type: 'AccountConstraintExecutable';
    value: { account: string };
}

export interface FankorErrorCode_AccountConstraintNotRentExempt {
    type: 'AccountConstraintNotRentExempt';
    value: { account: string };
}

export interface FankorErrorCode_AccountConstraintRentExempt {
    type: 'AccountConstraintRentExempt';
    value: { account: string };
}

export interface FankorErrorCode_AccountConstraintNotSigner {
    type: 'AccountConstraintNotSigner';
    value: { account: string };
}

export interface FankorErrorCode_AccountConstraintSigner {
    type: 'AccountConstraintSigner';
    value: { account: string };
}

export interface FankorErrorCode_AccountConstraintMinimumMismatch {
    type: 'AccountConstraintMinimumMismatch';
    value: { actual: BN; expected: BN; account: string };
}

export interface FankorErrorCode_AccountConstraintMaximumMismatch {
    type: 'AccountConstraintMaximumMismatch';
    value: { actual: BN; expected: BN; account: string };
}

export interface FankorErrorCode_AccountConstraintFailed {
    type: 'AccountConstraintFailed';
    value: { account: string; constraint: string };
}

export interface FankorErrorCode_DuplicatedAccountWithDifferentType {
    type: 'DuplicatedAccountWithDifferentType';
    value: { address: PublicKey };
}

export interface FankorErrorCode_AccountNotDefault {
    type: 'AccountNotDefault';
}

export interface FankorErrorCode_EmptyIntermediateBuffer {
    type: 'EmptyIntermediateBuffer';
}

export interface FankorErrorCode_IntermediateBufferIncorrectProgramId {
    type: 'IntermediateBufferIncorrectProgramId';
    value: { actual: PublicKey; expected: PublicKey };
}

export interface FankorErrorCode_TooManyAccounts {
    type: 'TooManyAccounts';
    value: { size: BN };
}

export interface FankorErrorCode_ZeroCopyCannotDeserialize {
    type: 'ZeroCopyCannotDeserialize';
    value: { typeName: string };
}

export interface FankorErrorCode_ZeroCopyNotEnoughLength {
    type: 'ZeroCopyNotEnoughLength';
    value: { typeName: string };
}

export interface FankorErrorCode_ZeroCopyInvalidEnumDiscriminant {
    type: 'ZeroCopyInvalidEnumDiscriminant';
    value: { typeName: string };
}

export interface FankorErrorCode_ZeroCopyPossibleDeadlock {
    type: 'ZeroCopyPossibleDeadlock';
    value: { typeName: string };
}

export interface FankorErrorCode_ZeroCopyLengthFieldOverflow {
    type: 'ZeroCopyLengthFieldOverflow';
}

export interface FankorErrorCode_ZeroCopyIncorrectPrecedingField {
    type: 'ZeroCopyIncorrectPrecedingField';
}

export interface FankorErrorCode_ZeroCopyInvalidMove {
    type: 'ZeroCopyInvalidMove';
}

export class FankorErrorCodeSchema implements FnkBorshSchema<FankorErrorCode> {
    innerSchema = null as any as ReturnType<
        FankorErrorCodeSchema['initSchema']
    >;

    // METHODS ----------------------------------------------------------------

    initSchema() {
        const innerSchema = TEnum([
            [1000, 'DeclaredProgramIdMismatch'],
            [1001, 'MissingInstructionDiscriminant'],
            [1002, 'InstructionDiscriminantNotFound'],
            [1003, 'UnusedAccounts'],
            [
                1004,
                'MissingProgram',
                TStruct([
                    ['address', TPublicKey],
                    ['name', TString],
                ] as const),
            ],
            [
                1005,
                'CannotFindValidPdaWithProvidedSeeds',
                TStruct([['programId', TPublicKey]] as const),
            ],
            [
                1006,
                'InvalidPda',
                TStruct([
                    ['expected', TPublicKey],
                    ['actual', TPublicKey],
                ] as const),
            ],
            [1007, 'MissingSeedsAccount'],
            [
                1008,
                'MissingPdaSeeds',
                TStruct([['account', TPublicKey]] as const),
            ],
            [
                1500,
                'DuplicatedWritableAccounts',
                TStruct([['address', TPublicKey]] as const),
            ],
            [
                1501,
                'AccountDiscriminantMismatch',
                TStruct([['account', TString]] as const),
            ],
            [
                1502,
                'InstructionDidNotDeserialize',
                TStruct([['account', TString]] as const),
            ],
            [
                1503,
                'AccountNotOwnedByProgram',
                TStruct([
                    ['address', TPublicKey],
                    ['action', TString],
                ] as const),
            ],
            [
                1504,
                'ReadonlyAccountModification',
                TStruct([
                    ['address', TPublicKey],
                    ['action', TString],
                ] as const),
            ],
            [
                1505,
                'MutRefToReadonlyAccount',
                TStruct([['address', TPublicKey]] as const),
            ],
            [
                1506,
                'NewFromClosedAccount',
                TStruct([['address', TPublicKey]] as const),
            ],
            [
                1507,
                'AccountNotRentExempt',
                TStruct([['account', TPublicKey]] as const),
            ],
            [
                1508,
                'AccountNotInitialized',
                TStruct([['address', TPublicKey]] as const),
            ],
            [
                1509,
                'AccountAlreadyInitialized',
                TStruct([['address', TPublicKey]] as const),
            ],
            [
                1510,
                'AccountOwnedByWrongProgram',
                TStruct([
                    ['address', TPublicKey],
                    ['expected', TPublicKey],
                    ['actual', TPublicKey],
                ] as const),
            ],
            [
                1511,
                'IncorrectSysvarAccount',
                TStruct([
                    ['actual', TPublicKey],
                    ['expected', TPublicKey],
                ] as const),
            ],
            [
                1512,
                'AlreadyClosedAccount',
                TStruct([
                    ['address', TPublicKey],
                    ['action', TString],
                ] as const),
            ],
            [
                1513,
                'InvalidProgram',
                TStruct([
                    ['expected', TPublicKey],
                    ['actual', TPublicKey],
                ] as const),
            ],
            [
                1514,
                'ProgramIsNotExecutable',
                TStruct([['program', TPublicKey]] as const),
            ],
            [1515, 'NotEnoughAccountKeys'],
            [1516, 'NotEnoughDataToDeserializeInstruction'],
            [1517, 'NotAccountsExpected'],
            [1518, 'NotEnoughValidAccountForVec'],
            [
                1519,
                'AccountConstraintOwnerMismatch',
                TStruct([
                    ['actual', TPublicKey],
                    ['expected', TPublicKey],
                    ['account', TString],
                ] as const),
            ],
            [
                1520,
                'AccountConstraintAddressMismatch',
                TStruct([
                    ['actual', TPublicKey],
                    ['expected', TPublicKey],
                    ['account', TString],
                ] as const),
            ],
            [
                1521,
                'AccountConstraintNotInitialized',
                TStruct([['account', TString]] as const),
            ],
            [
                1522,
                'AccountConstraintInitialized',
                TStruct([['account', TString]] as const),
            ],
            [
                1523,
                'AccountConstraintNotWritable',
                TStruct([['account', TString]] as const),
            ],
            [
                1524,
                'AccountConstraintWritable',
                TStruct([['account', TString]] as const),
            ],
            [
                1525,
                'AccountConstraintNotExecutable',
                TStruct([['account', TString]] as const),
            ],
            [
                1526,
                'AccountConstraintExecutable',
                TStruct([['account', TString]] as const),
            ],
            [
                1527,
                'AccountConstraintNotRentExempt',
                TStruct([['account', TString]] as const),
            ],
            [
                1528,
                'AccountConstraintRentExempt',
                TStruct([['account', TString]] as const),
            ],
            [
                1529,
                'AccountConstraintNotSigner',
                TStruct([['account', TString]] as const),
            ],
            [
                1530,
                'AccountConstraintSigner',
                TStruct([['account', TString]] as const),
            ],
            [
                1531,
                'AccountConstraintMinimumMismatch',
                TStruct([
                    ['actual', U64],
                    ['expected', U64],
                    ['account', TString],
                ] as const),
            ],
            [
                1532,
                'AccountConstraintMaximumMismatch',
                TStruct([
                    ['actual', U64],
                    ['expected', U64],
                    ['account', TString],
                ] as const),
            ],
            [
                1533,
                'AccountConstraintFailed',
                TStruct([
                    ['account', TString],
                    ['constraint', TString],
                ] as const),
            ],
            [
                1534,
                'DuplicatedAccountWithDifferentType',
                TStruct([['address', TPublicKey]] as const),
            ],
            [1535, 'AccountNotDefault'],
            [2000, 'EmptyIntermediateBuffer'],
            [
                2001,
                'IntermediateBufferIncorrectProgramId',
                TStruct([
                    ['actual', TPublicKey],
                    ['expected', TPublicKey],
                ] as const),
            ],
            [2002, 'TooManyAccounts', TStruct([['size', U64]] as const)],
            [
                2500,
                'ZeroCopyCannotDeserialize',
                TStruct([['typeName', TString]] as const),
            ],
            [
                2501,
                'ZeroCopyNotEnoughLength',
                TStruct([['typeName', TString]] as const),
            ],
            [
                2502,
                'ZeroCopyInvalidEnumDiscriminant',
                TStruct([['typeName', TString]] as const),
            ],
            [
                2503,
                'ZeroCopyPossibleDeadlock',
                TStruct([['typeName', TString]] as const),
            ],
            [2504, 'ZeroCopyLengthFieldOverflow'],
            [2505, 'ZeroCopyIncorrectPrecedingField'],
            [2506, 'ZeroCopyInvalidMove'],
        ] as const);
        this.innerSchema = innerSchema;
        return innerSchema;
    }

    serialize(writer: FnkBorshWriter, value: FankorErrorCode) {
        this.innerSchema.serialize(writer, value.data);
    }

    deserialize(reader: FnkBorshReader) {
        const result = this.innerSchema.deserialize(reader);
        return new FankorErrorCode(result as FankorErrorCodeTypes);
    }
}

const useFankorErrorCodeSchema = (() => {
    let variable: FankorErrorCodeSchema | null = null;
    return () => {
        if (variable === null) {
            variable = new FankorErrorCodeSchema();
            variable.initSchema();
        }

        return variable;
    };
})();
export const TFankorErrorCode = useFankorErrorCodeSchema();
