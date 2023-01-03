import BN from 'bn.js';
import { FnkBorshReader } from '../deserializer';
import { FnkBorshWriter } from '../serializer';
import { FnkBorshSchema } from '../borsh';

export class U8Schema implements FnkBorshSchema<number> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: number) {
        writer.writeByte(value);
    }

    deserialize(reader: FnkBorshReader): number {
        return reader.readByte();
    }
}

export const U8 = new U8Schema();

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export class U16Schema implements FnkBorshSchema<number> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: number) {
        writer.maybeResize();
        writer.buffer.writeUInt16LE(value, writer.length);
        writer.length += 2;
    }

    deserialize(reader: FnkBorshReader): number {
        const value = reader.buffer.readUInt16LE(reader.offset);
        reader.offset += 2;
        return value;
    }
}

export const U16 = new U16Schema();

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export class U32Schema implements FnkBorshSchema<number> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: number) {
        writer.maybeResize();
        writer.buffer.writeUInt32LE(value, writer.length);
        writer.length += 4;
    }

    deserialize(reader: FnkBorshReader): number {
        const value = reader.buffer.readUInt32LE(reader.offset);
        reader.offset += 4;
        return value;
    }
}

export const U32 = new U32Schema();

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export class U64Schema implements FnkBorshSchema<BN | bigint | number> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: BN | bigint | number) {
        value =
            typeof value === 'bigint'
                ? new BN(value.toString())
                : new BN(value);

        writer.writeBuffer(Buffer.from(value.toArray('le', 8)));
    }

    deserialize(reader: FnkBorshReader): BN {
        const buffer = reader.readBuffer(8);
        return new BN(buffer, 'le');
    }
}

export const U64 = new U64Schema();

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export class U128Schema implements FnkBorshSchema<BN | bigint | number> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: BN | bigint | number) {
        value =
            typeof value === 'bigint'
                ? new BN(value.toString())
                : new BN(value);

        writer.writeBuffer(Buffer.from(value.toArray('le', 16)));
    }

    deserialize(reader: FnkBorshReader): BN {
        const buffer = reader.readBuffer(16);
        return new BN(buffer, 'le');
    }
}

export const U128 = new U128Schema();
