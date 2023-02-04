import BN from 'bn.js';
import { FnkBorshReader } from '../../deserializer';
import { FnkBorshWriter } from '../../serializer';
import { FnkBorshSchema } from '../../borsh';

const ZERO = new BN(0);
const MAX_VALUE = new BN('18446744073709551615'); // 2^64 - 1
const BN_0x3F = new BN(0x3f);
const BN_0x40 = new BN(0x40);
const FLAG_ENCODING_LIMIT = new BN(1).shln(14); // 2^14

export class FnkUIntSchema implements FnkBorshSchema<BN> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: BN) {
        if (value.lt(ZERO)) {
            throw new RangeError('FnkUInt cannot be negative');
        }

        if (value.gt(MAX_VALUE)) {
            throw new RangeError('FnkUInt cannot be greater than 2^64 - 1');
        }

        if (value.lt(FLAG_ENCODING_LIMIT)) {
            // Flag encoding.
            let remaining = value;

            // Write first.
            let byte = value.and(BN_0x3F);
            remaining = remaining.shrn(6);

            // Include next flag.
            if (!remaining.isZero()) {
                byte = byte.or(BN_0x40);
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

            writer.writeByte(byteLength);
            writer.writeBuffer(bytes);
        }
    }

    deserialize(reader: FnkBorshReader): BN {
        let firstByte = reader.readByte();

        if ((firstByte & 0x80) === 0) {
            // Flag encoding.
            let number = new BN(firstByte & 0x3f);

            if ((firstByte & 0x40) !== 0) {
                // Read second byte.
                let byte = reader.readByte();
                number = number.or(new BN(byte).shln(6));
            }

            return number;
        } else {
            // Length encoding.
            let byteLength = firstByte & 0x7f;

            if (byteLength >= 7) {
                throw new RangeError('Incorrect FnkUInt length');
            }

            byteLength += 2;

            let number = ZERO;
            let offset = 0;

            for (let i = 0; i < byteLength; i += 1) {
                let byte = new BN(reader.readByte()).shln(offset);
                number = number.or(byte);
                offset += 8;
            }

            return number;
        }
    }
}

export const FnkUInt = new FnkUIntSchema();
