import { FnkBorshWriter } from '../../serializer';
import { FnkBorshReader } from '../../deserializer';
import { FnkUIntSchema } from './unsigned';
import { FnkBorshSchema } from '../../borsh';
import { InferFnkBorshSchemaInner } from '../maps';

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
        new FnkUIntSchema().serialize(writer, value.length);

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
