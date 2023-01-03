import BN from 'bn.js';
import {FnkBorshReader, FnkBorshReadSchema} from '../deserializer';
import {FnkBorshWriter, FnkBorshWriteSchema} from '../serializer';
import {FnkBorshSchema} from '../index';

export class I8Schema implements FnkBorshSchema<number> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: number) {
        writer.maybeResize();
        writer.buffer.writeInt8(value, writer.length);
        writer.length += 1;
    }

    deserialize(reader: FnkBorshReader): number {
        const value = reader.buffer.readInt8(reader.offset);
        reader.offset += 2;
        return value;
    }
}

export const I8 = new I8Schema();

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export class I16Schema implements FnkBorshSchema<number> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: number) {
        writer.maybeResize();
        writer.buffer.writeInt16LE(value, writer.length);
        writer.length += 2;
    }

    deserialize(reader: FnkBorshReader): number {
        const value = reader.buffer.readInt16LE(reader.offset);
        reader.offset += 2;
        return value;
    }
}

export const I16 = new I16Schema();

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export class I32Schema implements FnkBorshSchema<number> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: number) {
        writer.maybeResize();
        writer.buffer.writeInt32LE(value, writer.length);
        writer.length += 4;
    }

    deserialize(reader: FnkBorshReader): number {
        const value = reader.buffer.readInt32LE(reader.offset);
        reader.offset += 4;
        return value;
    }
}

export const I32 = new I32Schema();

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export class I64Schema implements FnkBorshReadSchema<BN | bigint | number>, FnkBorshWriteSchema<BN | bigint | number> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: BN | bigint | number) {
        value = typeof value === 'bigint' ? new BN(value.toString()) : new BN(value);

        writer.writeBuffer(Buffer.from(value.toArray('le', 8)));
    }

    deserialize(reader: FnkBorshReader): BN {
        const buffer = reader.readBuffer(8);
        return new BN(buffer, 'le');
    }
}

export const I64 = new I64Schema();

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export class I128Schema implements FnkBorshReadSchema<BN | bigint | number>, FnkBorshWriteSchema<BN | bigint | number> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: BN | bigint | number) {
        value = typeof value === 'bigint' ? new BN(value.toString()) : new BN(value);

        writer.writeBuffer(Buffer.from(value.toArray('le', 16)));
    }

    deserialize(reader: FnkBorshReader): BN {
        const buffer = reader.readBuffer(16);
        return new BN(buffer, 'le');
    }
}

export const I128 = new I128Schema();