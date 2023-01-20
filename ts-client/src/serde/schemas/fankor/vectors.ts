import { FnkBorshWriter } from '../../serializer';
import { FnkBorshReader } from '../../deserializer';
import { FnkUIntSchema } from './unsigned';
import { FnkBorshSchema } from '../../borsh';
import { InferFnkBorshSchemaInner } from '../maps';
import { FnkBorshError } from '../../errors';
import { numberToBN } from '../../../utils';

export class FnkByteVecSchema implements FnkBorshSchema<Uint8Array> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: Uint8Array) {
        new FnkUIntSchema().serialize(writer, numberToBN(value.length));

        const buffer = Buffer.from(value);
        writer.writeBuffer(buffer);
    }

    deserialize(reader: FnkBorshReader): Uint8Array {
        const size = new FnkUIntSchema().deserialize(reader).toNumber();
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

export const FnkByteVec = new FnkByteVecSchema();

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export function FnkVec<S extends FnkBorshSchema<any>>(schema: S) {
    return new FnkVecSchema(schema);
}

export class FnkVecSchema<S extends FnkBorshSchema<any>>
    implements FnkBorshSchema<InferFnkBorshSchemaInner<S>[]>
{
    readonly schema: S;

    // CONSTRUCTOR ------------------------------------------------------------

    constructor(schema: S) {
        this.schema = schema;
    }

    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: InferFnkBorshSchemaInner<S>[]) {
        new FnkUIntSchema().serialize(writer, numberToBN(value.length));

        for (const item of value) {
            this.schema.serialize(writer, item);
        }
    }

    deserialize(reader: FnkBorshReader): InferFnkBorshSchemaInner<S>[] {
        const size = new FnkUIntSchema().deserialize(reader).toNumber();
        const result: InferFnkBorshSchemaInner<S>[] = [];

        for (let i = 0; i < size; i++) {
            result.push(this.schema.deserialize(reader));
        }

        return result;
    }
}
