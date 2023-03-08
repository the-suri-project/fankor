import { FnkBorshError } from './errors';

export class FnkBorshReader {
    buffer: Buffer;
    offset: number;

    // CONSTRUCTORS -----------------------------------------------------------

    constructor(buffer: Buffer, offset?: number) {
        this.buffer = buffer;
        this.offset = offset ?? 0;
    }

    peekByte(): number {
        if (this.offset + 1 > this.buffer.length) {
            throw new FnkBorshError(
                `Expected buffer length(${1}) isn't within bounds`
            );
        }

        return this.buffer.readUInt8(this.offset);
    }

    readByte(): number {
        const value = this.peekByte();
        this.offset += 1;
        return value;
    }

    peekBuffer(length: number): Buffer {
        if (this.offset + length > this.buffer.length) {
            throw new FnkBorshError(
                `Expected buffer length(${length}) isn't within bounds`
            );
        }
        return this.buffer.slice(this.offset, this.offset + length);
    }

    readBuffer(length: number): Buffer {
        const result = this.peekBuffer(length);
        this.offset += length;
        return result;
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export interface FnkBorshReadSchema<T> {
    deserialize(reader: FnkBorshReader): T;
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export function deserialize<S extends FnkBorshReadSchema<T>, T>(
    schema: S,
    reader: FnkBorshReader
): T {
    const result = schema.deserialize(reader);

    if (reader.offset < reader.buffer.length) {
        throw new FnkBorshError(
            `Unexpected ${
                reader.buffer.length - reader.offset
            } bytes after deserialized data`
        );
    }

    return result;
}
