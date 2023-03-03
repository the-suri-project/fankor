import BN from 'bn.js';
import { FnkBorshWriter } from '../../serializer';
import { FnkBorshReader } from '../../deserializer';
import { FnkBorshSchema } from '../../borsh';

const ZERO = new BN(0);
const MIN_VALUE = new BN(1).shln(63).neg(); // -2^63
const MIN_I64_ABS = new BN(1).shln(63); // 2^63
const MAX_VALUE = new BN(1).shln(63).subn(1); // 2^63 - 1
const BN_0x1F = new BN(0x1f);
const BN_0x20 = new BN(0x20);
const BN_0x40 = new BN(0x40);
const FLAG_ENCODING_LIMIT = new BN(1).shln(13); // 2^13

export class FnkIntSchema implements FnkBorshSchema<BN> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: BN) {
        if (value.lt(MIN_VALUE)) {
            throw new RangeError('FnkInt cannot be greater than -2^63');
        }

        if (value.gt(MAX_VALUE)) {
            throw new RangeError('FnkInt cannot be greater than 2^63 - 1');
        }

        let isNegative = value.isNeg();
        value = value.abs();

        if (value.lt(FLAG_ENCODING_LIMIT)) {
            // Flag encoding.
            let remaining = value;

            // Write first.
            let byte = value.and(BN_0x1F);
            remaining = remaining.shrn(5);

            // Include next flag.
            if (!remaining.isZero()) {
                byte = byte.or(BN_0x40);
            }

            // Include sign bit.
            if (isNegative) {
                byte = byte.or(BN_0x20);
            }

            writer.writeByte(byte.toNumber());

            // Write second byte.
            if (!remaining.isZero()) {
                writer.writeByte(remaining.toNumber());
            }
        } else {
            // Length encoding.
            let byteLength = 8;
            let bytes = value.toArrayLike(Buffer, 'le', 8);

            for (let i = 7; i >= 0; i -= 1) {
                if (bytes[i] != 0) {
                    break;
                }

                byteLength -= 1;
            }

            bytes = bytes.slice(0, byteLength);
            byteLength = (byteLength - 2) | 0x80;

            // Include sign bit.
            if (isNegative) {
                byteLength |= 0x40;
            }

            writer.writeByte(byteLength);
            writer.writeBuffer(bytes);
        }
    }

    deserialize(reader: FnkBorshReader): BN {
        let firstByte = reader.readByte();

        if ((firstByte & 0x80) === 0) {
            // Flag encoding.
            let number = new BN(firstByte & 0x1f);

            if ((firstByte & 0x40) !== 0) {
                // Read second byte.
                let byte = reader.readByte();
                number = number.or(new BN(byte).shln(5));
            }

            // Process sign bit.
            if ((firstByte & 0x20) !== 0) {
                number = number.neg();
            }

            return number;
        } else {
            // Length encoding.
            let byteLength = firstByte & 0x3f;

            if (byteLength >= 7) {
                throw new RangeError('Incorrect FnkInt length');
            }

            byteLength += 2;

            let number = ZERO;
            let offset = 0;

            for (let i = 0; i < byteLength; i += 1) {
                let byte = new BN(reader.readByte()).shln(offset);
                number = number.or(byte);
                offset += 8;
            }

            if ((firstByte & 0x40) === 0) {
                if (number.gt(MAX_VALUE)) {
                    throw new RangeError('Incorrect FnkInt value');
                }
            } else {
                if (number.lte(MIN_I64_ABS)) {
                    number = number.neg();
                } else {
                    throw new RangeError('Incorrect FnkInt value');
                }
            }

            return number;
        }
    }
}

export const FnkInt = new FnkIntSchema();
