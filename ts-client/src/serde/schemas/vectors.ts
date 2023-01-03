import { FnkBorshReader } from '../deserializer';
import { FnkBorshWriter } from '../serializer';
import { FnkBorshError } from '../errors';
import { U32Schema } from './unsigned';
import { FnkBorshSchema } from '../index';

export class ByteVecSchema implements FnkBorshSchema<Uint8Array> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: Uint8Array) {
        new U32Schema().serialize(writer, value.length);

        const buffer = Buffer.from(value);
        writer.writeBuffer(buffer);
    }

    deserialize(reader: FnkBorshReader): Uint8Array {
        const size = new U32Schema().deserialize(reader);
        const endIndex = reader.offset + size;

        if (endIndex > reader.buffer.length) {
            throw new FnkBorshError(
                `Expected buffer length ${size} isn't within bounds`
            );
        }

        const buffer = reader.buffer.slice(reader.offset, endIndex);
        reader.offset += size;

        return buffer.subarray();
    }
}

export const ByteVec = new ByteVecSchema();

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export function Vec<T, S extends FnkBorshSchema<T>>(schema: S) {
    return new VecSchema(schema);
}

export class VecSchema<T, S extends FnkBorshSchema<T>>
    implements FnkBorshSchema<T[]>
{
    readonly schema: S;

    // CONSTRUCTOR ------------------------------------------------------------

    constructor(schema: S) {
        this.schema = schema;
    }

    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: T[]) {
        new U32Schema().serialize(writer, value.length);

        for (const item of value) {
            this.schema.serialize(writer, item);
        }
    }

    deserialize(reader: FnkBorshReader): T[] {
        const size = new U32Schema().deserialize(reader);
        const result: T[] = [];

        for (let i = 0; i < size; i++) {
            result.push(this.schema.deserialize(reader));
        }

        return result;
    }
}
