import {BinaryReader, BinaryWriter} from 'borsh';
import BN from 'bn.js';

const ZERO = new BN(0);
const MAX_VALUE = new BN('18446744073709551615'); // 2^64 - 1

export class FnkUInt {
    readonly value: BN;

    // CONSTRUCTORS -----------------------------------------------------------

    constructor(value: BN | number | string) {
        value = new BN(value);

        if (value.lt(ZERO)) {
            throw new RangeError('FnkUInt cannot be negative');
        }

        if (value.gt(MAX_VALUE)) {
            throw new RangeError('FnkUInt cannot be greater than 2^64 - 1');
        }

        this.value = new BN(value);
    }

    static zero(): FnkUInt {
        return new FnkUInt(ZERO);
    }

    static maxValue(): FnkUInt {
        return new FnkUInt(MAX_VALUE);
    }

    // GETTERS AND SETTERS ----------------------------------------------------

    get valueAsNumber(): number {
        return this.value.toNumber();
    }

    // METHODS ----------------------------------------------------------------

    borshSerialize(writer: BinaryWriter) {
        writer.writeFnkUInt(this);
    }

    borshDeserialize(reader: BinaryReader): FnkUInt {
        return reader.readFnkUInt();
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

declare module 'borsh' {
    export interface BinaryWriter {
        writeFnkUInt(value: FnkUInt);
    }

    export interface BinaryReader {
        readFnkUInt(): FnkUInt;
    }
}

const BN_0x3F = new BN(0x3F);
const BN_0x40 = new BN(0x40);
const BN_0x7F = new BN(0x7F);
const BN_0x80 = new BN(0x80);
(BinaryWriter.prototype as any).writeFnkUInt = function (value: FnkUInt) {
    const writer = this as unknown as BinaryWriter;
    const bn_value = value.value;
    let bit_length = 64 - (bn_value.toString(2).padStart(64, '0').match(/^0*/)?.[0]?.length || 0);

    if (bit_length <= 13) {
        // Flag encoding.
        let byte_length = (bit_length <= 6) ? 1 : Math.floor((bit_length - 6 + 8) / 8) + 1;

        // Write first.
        let byte = (bn_value.and(BN_0x3F));

        // Include next flag.
        if (byte_length > 1) {
            byte = byte.or(BN_0x40);
        }

        writer.writeU8(byte.toNumber());

        // Write remaining bytes.
        let offset = 6;
        let last = byte_length - 1;
        for (let i = 1; i < byte_length; i += 1) {
            let byte = (bn_value.shrn(offset).and(BN_0x7F)).or(BN_0x80);

            if (i >= last) {
                byte = byte.and(BN_0x7F);
            }

            writer.writeU8(byte.toNumber());
            offset += 7;
        }
    } else {
        // Length encoding.
        let byte_length = Math.min(Math.floor((bit_length + 8) / 8), 8);
        const bytes = bn_value.toArrayLike(Buffer, 'le', 8).slice(0, byte_length);
        byte_length = byte_length | 0x80;

        writer.writeU8(byte_length);
        writer.writeFixedArray(bytes);
    }
};

(BinaryReader.prototype as any).readFnkUInt = function () {
    const reader = this as unknown as BinaryReader;

    let first_byte = reader.readU8();

    if ((first_byte & 0x80) == 0) {
        // Flag encoding.
        let number = new BN(first_byte & 0x3F);

        if ((first_byte & 0x40) != 0) {
            // Read remaining bytes.
            let offset = 6;

            while (true) {
                let byte = reader.readU8();
                number = number.or(new BN(byte & 0x7F).shln(offset));

                if ((byte & 0x80) == 0) {
                    break;
                }

                offset += 7;
            }
        }

        return new FnkUInt(number);
    } else {
        // Length encoding.
        let byte_length = first_byte & 0x7F;

        let number = ZERO;

        let offset = 0;
        for (let i = 0; i < byte_length; i += 1) {
            let byte = new BN(reader.readU8()).shln(offset);
            number = number.or(byte);
            offset += 8;
        }

        return new FnkUInt(number);
    }
};