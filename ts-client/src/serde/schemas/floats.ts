import { FnkBorshReader } from '../deserializer';
import { FnkBorshWriter } from '../serializer';
import { FnkBorshSchema } from '../borsh';

export class F32Schema implements FnkBorshSchema<number> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: number) {
        let buffer = Buffer.alloc(4);
        buffer.writeFloatLE(value, 0);

        writer.writeBuffer(buffer);
    }

    deserialize(reader: FnkBorshReader): number {
        const buffer = reader.readBuffer(4);
        return buffer.readFloatLE(0);
    }
}

export const F32 = new F32Schema();

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export class F64Schema implements FnkBorshSchema<number> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: number) {
        let buffer = Buffer.alloc(8);
        buffer.writeDoubleLE(value, 0);

        writer.writeBuffer(buffer);
    }

    deserialize(reader: FnkBorshReader): number {
        const buffer = reader.readBuffer(8);
        return buffer.readDoubleLE(0);
    }
}

export const F64 = new F64Schema();
