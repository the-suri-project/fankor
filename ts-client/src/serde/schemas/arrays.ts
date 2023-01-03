import {FnkBorshReader} from '../deserializer';
import {FnkBorshWriter} from '../serializer';
import {FnkBorshError} from '../errors';
import {FnkBorshSchema} from '../index';

export const ByteArray = (size: number) => new ByteArraySchema(size);

export class ByteArraySchema implements FnkBorshSchema<Uint8Array> {
    readonly size: number;

    // CONSTRUCTOR ------------------------------------------------------------

    constructor(size: number) {
        this.size = size;
    }

    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: Uint8Array) {
        if (value.length !== this.size) {
            throw new Error(`ArraySchema: expected ${this.size} items, got ${value.length}`);
        }

        const buffer = Buffer.from(value);
        writer.writeBuffer(buffer);
    }

    deserialize(reader: FnkBorshReader): Uint8Array {
        const endIndex = reader.offset + this.size;

        if (endIndex > reader.buffer.length) {
            throw new FnkBorshError(`Expected buffer length ${this.size} isn't within bounds`);
        }

        const buffer = reader.buffer.slice(reader.offset, endIndex);
        reader.offset += this.size;

        return buffer.subarray();
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export function TArray<T, S extends FnkBorshSchema<T>>({
    schema,
    size,
}: { schema: S, size: number }) {
    return new ArraySchema(schema, size);
}

export class ArraySchema<T, S extends FnkBorshSchema<T>> implements FnkBorshSchema<T[]> {
    readonly schema: S;
    readonly size: number;

    // CONSTRUCTOR ------------------------------------------------------------

    constructor(schema: S, size: number) {
        this.schema = schema;
        this.size = size;
    }

    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: T[]) {
        if (value.length !== this.size) {
            throw new Error(`ArraySchema: expected ${this.size} items, got ${value.length}`);
        }

        for (const item of value) {
            this.schema.serialize(writer, item);
        }
    }

    deserialize(reader: FnkBorshReader): T[] {
        const result: T[] = new Array(this.size);

        for (let i = 0; i < this.size; i++) {
            result.push(this.schema.deserialize(reader));
        }

        return result;
    }
}