import { FnkUIntSchema } from './unsigned';
import encoding from 'text-encoding-utf-8';
import { FnkBorshReader } from '../../deserializer';
import { FnkBorshWriter } from '../../serializer';
import { FnkBorshError } from '../../errors';
import { FnkBorshSchema } from '../../index';

export class FnkStringSchema implements FnkBorshSchema<string> {
    // METHODS ----------------------------------------------------------------
    serialize(writer: FnkBorshWriter, value: string) {
        const bytes = Buffer.from(value, 'utf8');
        new FnkUIntSchema().serialize(writer, bytes.length);
        writer.writeBuffer(bytes);
    }

    deserialize(reader: FnkBorshReader): string {
        const length = new FnkUIntSchema().deserialize(reader).toNumber();
        const endIndex = reader.offset + length;

        if (endIndex > reader.buffer.length) {
            throw new FnkBorshError(
                `Expected buffer length ${length} isn't within bounds`
            );
        }

        const buf = reader.buffer.slice(reader.offset, endIndex);
        reader.offset += length;

        try {
            // NOTE: Using TextDecoder to fail on invalid UTF-8
            const ResolvedTextDecoder =
                typeof TextDecoder !== 'function'
                    ? encoding.TextDecoder
                    : TextDecoder;
            const textDecoder = new ResolvedTextDecoder('utf-8', {
                fatal: true,
            });
            return textDecoder.decode(buf);
        } catch (e) {
            throw new FnkBorshError(`Error decoding UTF-8 string: ${e}`);
        }
    }
}

export const FnkString = new FnkStringSchema();
