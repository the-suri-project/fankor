import {BinaryReader, BinaryWriter, BorshError} from 'borsh';
import {FnkUInt} from './unsigned';
import encoding from 'text-encoding-utf-8';

export class FnkString {
    readonly value: string;

    // CONSTRUCTORS -----------------------------------------------------------

    constructor(value: string) {
        this.value = value;
    }

    // METHODS ----------------------------------------------------------------

    borshSerialize(writer: BinaryWriter) {
        writer.writeFnkString(this);
    }

    borshDeserialize(reader: BinaryReader): FnkString {
        return reader.readFnkString();
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

declare module 'borsh' {
    export interface BinaryWriter {
        writeFnkString(value: FnkString);
    }

    export interface BinaryReader {
        readFnkString(): FnkString;
    }
}

(BinaryWriter.prototype as any).writeFnkString = function (value: FnkString) {
    const writer = this as unknown as BinaryWriter;

    const b = Buffer.from(value.value, 'utf8');
    writer.writeFnkUInt(new FnkUInt(b.length));
    this.writeBuffer(b);
};

(BinaryReader.prototype as any).readFnkString = function () {
    const reader = this as unknown as BinaryReader;

    const len = reader.readFnkUInt();
    const len2 = len.value.toNumber();
    const startIndex = reader.offset + len2;

    if (startIndex > this.buf.length) {
        throw new BorshError(`Expected buffer length ${len} isn't within bounds`);
    }

    const buf = reader.buf.slice(reader.offset, startIndex);
    reader.offset += len2;

    try {
        // NOTE: Using TextDecoder to fail on invalid UTF-8
        const ResolvedTextDecoder = typeof TextDecoder !== 'function' ? encoding.TextDecoder : TextDecoder;
        const textDecoder = new ResolvedTextDecoder('utf-8', {fatal: true});
        return new FnkString(textDecoder.decode(buf));
    } catch (e) {
        throw new BorshError(`Error decoding UTF-8 string: ${e}`);
    }
};