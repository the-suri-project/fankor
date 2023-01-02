import {BinaryReader, BinaryWriter} from 'borsh';
import BN from 'bn.js';

const ZERO = new BN(0);
const MIN_VALUE = new BN('-9223372036854775808'); // -2^63
const MIN_VALUE_ABS = new BN('9223372036854775808'); // 2^63
const MAX_VALUE = new BN('9223372036854775807'); // 2^63 - 1

export class FnkInt {
    readonly value: BN;

    // CONSTRUCTORS -----------------------------------------------------------

    constructor(value: BN | number | string) {
        value = new BN(value);

        if (value.lt(MIN_VALUE)) {
            throw new RangeError('FnkInt cannot be greater than -2^63');
        }

        if (value.gt(MAX_VALUE)) {
            throw new RangeError('FnkInt cannot be greater than 2^63 - 1');
        }

        this.value = value;
    }

    static zero(): FnkInt {
        return new FnkInt(ZERO);
    }

    static minValue(): FnkInt {
        return new FnkInt(MIN_VALUE);
    }

    static maxValue(): FnkInt {
        return new FnkInt(MAX_VALUE);
    }

    // GETTERS AND SETTERS ----------------------------------------------------
    get valueAsNumber(): number {
        return this.value.toNumber();
    }

    // METHODS ----------------------------------------------------------------

    borshSerialize(writer: BinaryWriter) {
        writer.writeFnkInt(this);
    }

    borshDeserialize(reader: BinaryReader): FnkInt {
        return reader.readFnkInt();
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

declare module 'borsh' {
    export interface BinaryWriter {
        writeFnkInt(value: FnkInt);
    }

    export interface BinaryReader {
        readFnkInt(): FnkInt;
    }
}

const BN_0x1F = new BN(0x1F);
const BN_0x20 = new BN(0x20);
const BN_0x40 = new BN(0x40);
const BN_0x7F = new BN(0x7F);
const BN_0x80 = new BN(0x80);
(BinaryWriter.prototype as any).writeFnkInt = function (value: FnkInt) {
    const writer = this as unknown as BinaryWriter;
    const bn_value = value.value.abs();
    let bit_length = 64 - (bn_value.toString(2).padStart(64, '0').match(/^0*/)?.[0]?.length || 0);

    if (bit_length <= 12) {
        // Flag encoding.
        let byte_length = (bit_length <= 5) ? 1 : Math.floor((bit_length - 5 + 8) / 8) + 1;

        // Write first.
        let byte = (bn_value.and(BN_0x1F));

        // Include next flag.
        if (byte_length > 1) {
            byte = byte.or(BN_0x40);
        }

        // Negative flag.
        if (value.value.isNeg()) {
            byte = byte.or(BN_0x20);
        }

        writer.writeU8(byte.toNumber());

        // Write remaining bytes.
        let offset = 5;
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

        // Negative flag.
        if (value.value.isNeg()) {
            byte_length = byte_length | 0x40;
        }

        writer.writeU8(byte_length);
        writer.writeFixedArray(bytes);
    }
};

(BinaryReader.prototype as any).readFnkInt = function () {
    const reader = this as unknown as BinaryReader;

    let first_byte = reader.readU8();

    if ((first_byte & 0x80) == 0) {
        // Flag encoding.
        let number = new BN(first_byte & 0x1F);

        if ((first_byte & 0x40) != 0) {
            // Read remaining bytes.
            let offset = 5;

            while (true) {
                let byte = reader.readU8();
                number = number.or(new BN(byte & 0x7F).shln(offset));

                if ((byte & 0x80) == 0) {
                    break;
                }

                offset += 7;
            }
        }

        // Negative.
        if ((first_byte & 0x20) != 0) {
            return new FnkInt(-number);
        } else {
            return new FnkInt(number);
        }
    } else {
        // Length encoding.
        let byte_length = first_byte & 0x3F;

        let number = ZERO;

        let offset = 0;
        for (let i = 0; i < byte_length; i += 1) {
            let byte = new BN(reader.readU8()).shln(offset);
            number = number.or(byte);
            offset += 8;
        }

        // Negative.
        if ((first_byte & 0x40) != 0) {
            if (number.eq(MIN_VALUE_ABS)) {
                return new FnkInt(MIN_VALUE);
            } else if (number.gt(MAX_VALUE)) {
                throw new RangeError('Number underflow');
            } else {
                return new FnkInt(number.neg());
            }
        } else {
            if (number.gt(MAX_VALUE)) {
                throw new RangeError('Number overflow');
            }

            return new FnkInt(number);
        }
    }
};