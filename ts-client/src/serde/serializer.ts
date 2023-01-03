const INITIAL_LENGTH = 1024;

export class FnkBorshWriter {
    buffer: Buffer;
    length: number;

    // CONSTRUCTORS -----------------------------------------------------------
    constructor(buffer?: Buffer) {
        this.buffer = buffer ?? Buffer.alloc(INITIAL_LENGTH);
        this.length = 0;
    }

    // METHODS ----------------------------------------------------------------

    maybeResize() {
        if (this.buffer.length < 16 + this.length) {
            this.buffer = Buffer.concat([this.buffer, Buffer.alloc(INITIAL_LENGTH)]);
        }
    }

    writeByte(value: number) {
        this.maybeResize();
        this.buffer.writeUInt8(value, this.length);
        this.length += 1;
    }

    writeBuffer(buffer: Buffer) {
        this.maybeResize();
        // Buffer.from is needed as this.buf.subarray can return plain Uint8Array in browser
        this.buffer =
            Buffer.concat([Buffer.from(this.buffer.subarray(0, this.length)), buffer, Buffer.alloc(INITIAL_LENGTH)]);
        this.length += buffer.length;
    }

    toByteArray() {
        return this.buffer.subarray(0, this.length);
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export interface FnkBorshWriteSchema<T> {
    serialize(writer: FnkBorshWriter, value: T);
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export function serialize<S extends FnkBorshWriteSchema<T>, T>(schema: S, value: T, writer = new FnkBorshWriter()) {
    schema.serialize(writer, value);
}