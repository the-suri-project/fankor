import BN from 'bn.js';
import {FnkBorshWriter} from '../../serializer';
import {FnkBorshReader} from '../../deserializer';
import {FnkBorshSchema} from '../../index';

const ZERO = new BN(0);
const MIN_VALUE = new BN('-9223372036854775808'); // -2^63
const MIN_VALUE_ABS = new BN('9223372036854775808'); // 2^63
const MAX_VALUE = new BN('9223372036854775807'); // 2^63 - 1
const BN_0x1F = new BN(0x1F);
const BN_0x20 = new BN(0x20);
const BN_0x40 = new BN(0x40);
const BN_0x7F = new BN(0x7F);
const BN_0x80 = new BN(0x80);

export class FnkIntSchema implements FnkBorshSchema<BN | bigint | number> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: BN | bigint | number) {
        value = typeof value === 'bigint' ? new BN(value.toString()) : new BN(value);

        if (value.lt(MIN_VALUE)) {
            throw new RangeError('FnkInt cannot be greater than -2^63');
        }

        if (value.gt(MAX_VALUE)) {
            throw new RangeError('FnkInt cannot be greater than 2^63 - 1');
        }

        let isNegative = value.isNeg();
        value = value.abs();

        let bit_length = 64 - (value.toString(2).padStart(64, '0').match(/^0*/)?.[0]?.length || 0);

        if (bit_length <= 12) {
            // Flag encoding.
            let byte_length = (bit_length <= 5) ? 1 : Math.floor((bit_length - 5 + 8) / 8) + 1;

            // Write first.
            let byte = (value.and(BN_0x1F));

            // Include next flag.
            if (byte_length > 1) {
                byte = byte.or(BN_0x40);
            }

            // Negative flag.
            if (isNegative) {
                byte = byte.or(BN_0x20);
            }

            writer.writeByte(byte.toNumber());

            // Write remaining bytes.
            let offset = 5;
            let last = byte_length - 1;
            for (let i = 1; i < byte_length; i += 1) {
                let byte = (value.shrn(offset).and(BN_0x7F)).or(BN_0x80);

                if (i >= last) {
                    byte = byte.and(BN_0x7F);
                }

                writer.writeByte(byte.toNumber());
                offset += 7;
            }
        } else {
            // Length encoding.
            let byte_length = Math.min(Math.floor((bit_length + 8) / 8), 8);
            const bytes = value.toArrayLike(Buffer, 'le', 8).slice(0, byte_length);
            byte_length = byte_length | 0x80;

            // Negative flag.
            if (isNegative) {
                byte_length = byte_length | 0x40;
            }

            writer.writeByte(byte_length);
            writer.writeBuffer(bytes);
        }
    }

    deserialize(reader: FnkBorshReader): BN {
        let first_byte = reader.readByte();

        if ((first_byte & 0x80) == 0) {
            // Flag encoding.
            let number = new BN(first_byte & 0x1F);

            if ((first_byte & 0x40) != 0) {
                // Read remaining bytes.
                let offset = 5;

                while (true) {
                    let byte = reader.readByte();
                    number = number.or(new BN(byte & 0x7F).shln(offset));

                    if ((byte & 0x80) == 0) {
                        break;
                    }

                    offset += 7;
                }
            }

            // Negative.
            if ((first_byte & 0x20) != 0) {
                return number.neg();
            } else {
                return number;
            }
        } else {
            // Length encoding.
            let byte_length = first_byte & 0x3F;

            let number = ZERO;

            let offset = 0;
            for (let i = 0; i < byte_length; i += 1) {
                let byte = new BN(reader.readByte()).shln(offset);
                number = number.or(byte);
                offset += 8;
            }

            // Negative.
            if ((first_byte & 0x40) != 0) {
                if (number.eq(MIN_VALUE_ABS)) {
                    return MIN_VALUE;
                } else if (number.gt(MAX_VALUE)) {
                    throw new RangeError('Number underflow');
                } else {
                    return number.neg();
                }
            } else {
                if (number.gt(MAX_VALUE)) {
                    throw new RangeError('Number overflow');
                }

                return number;
            }
        }
    }
}

export const FnkInt = new FnkIntSchema();