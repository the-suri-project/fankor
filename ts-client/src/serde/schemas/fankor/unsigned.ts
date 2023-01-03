import BN from 'bn.js';
import { FnkBorshReader } from '../../deserializer';
import { FnkBorshWriter } from '../../serializer';
import { FnkBorshSchema } from '../../index';

const ZERO = new BN(0);
const MAX_VALUE = new BN('18446744073709551615'); // 2^64 - 1
const BN_0x3F = new BN(0x3f);
const BN_0x40 = new BN(0x40);
const BN_0x7F = new BN(0x7f);
const BN_0x80 = new BN(0x80);

export class FnkUIntSchema implements FnkBorshSchema<BN | bigint | number> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: BN | bigint | number) {
        value =
            typeof value === 'bigint'
                ? new BN(value.toString())
                : new BN(value);

        if (value.lt(ZERO)) {
            throw new RangeError('FnkUInt cannot be negative');
        }

        if (value.gt(MAX_VALUE)) {
            throw new RangeError('FnkUInt cannot be greater than 2^64 - 1');
        }

        let bit_length =
            64 -
            (value.toString(2).padStart(64, '0').match(/^0*/)?.[0]?.length ||
                0);

        if (bit_length <= 13) {
            // Flag encoding.
            let byte_length =
                bit_length <= 6 ? 1 : Math.floor((bit_length - 6 + 8) / 8) + 1;

            // Write first.
            let byte = value.and(BN_0x3F);

            // Include next flag.
            if (byte_length > 1) {
                byte = byte.or(BN_0x40);
            }

            writer.writeByte(byte.toNumber());

            // Write remaining bytes.
            let offset = 6;
            let last = byte_length - 1;
            for (let i = 1; i < byte_length; i += 1) {
                let byte = value.shrn(offset).and(BN_0x7F).or(BN_0x80);

                if (i >= last) {
                    byte = byte.and(BN_0x7F);
                }

                writer.writeByte(byte.toNumber());
                offset += 7;
            }
        } else {
            // Length encoding.
            let byte_length = Math.min(Math.floor((bit_length + 8) / 8), 8);
            const bytes = value
                .toArrayLike(Buffer, 'le', 8)
                .slice(0, byte_length);
            byte_length = byte_length | 0x80;

            writer.writeByte(byte_length);
            writer.writeBuffer(bytes);
        }
    }

    deserialize(reader: FnkBorshReader): BN {
        let first_byte = reader.readByte();

        if ((first_byte & 0x80) == 0) {
            // Flag encoding.
            let number = new BN(first_byte & 0x3f);

            if ((first_byte & 0x40) != 0) {
                // Read remaining bytes.
                let offset = 6;

                while (true) {
                    let byte = reader.readByte();
                    number = number.or(new BN(byte & 0x7f).shln(offset));

                    if ((byte & 0x80) == 0) {
                        break;
                    }

                    offset += 7;
                }
            }

            return number;
        } else {
            // Length encoding.
            let byte_length = first_byte & 0x7f;

            let number = ZERO;

            let offset = 0;
            for (let i = 0; i < byte_length; i += 1) {
                let byte = new BN(reader.readByte()).shln(offset);
                number = number.or(byte);
                offset += 8;
            }

            return number;
        }
    }
}

export const FnkUInt = new FnkUIntSchema();
