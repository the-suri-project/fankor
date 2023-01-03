import {FnkBorshWriter} from '../../serializer';
import {FnkBorshReader} from '../../deserializer';
import {FnkUIntSchema} from './unsigned';
import {FnkBorshSchema} from '../../index';

export function FnkVec<T, S extends FnkBorshSchema<T>>(schema: S) {
    return new FnkVecSchema(schema);
}

export class FnkVecSchema<T, S extends FnkBorshSchema<T>> implements FnkBorshSchema<T[]> {
    readonly schema: S;

    // CONSTRUCTOR ------------------------------------------------------------

    constructor(schema: S) {
        this.schema = schema;
    }

    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: T[]) {
        new FnkUIntSchema().serialize(writer, value.length);

        for (const item of value) {
            this.schema.serialize(writer, item);
        }
    }

    deserialize(reader: FnkBorshReader): T[] {
        const size = new FnkUIntSchema().deserialize(reader).toNumber();
        const result: T[] = [];

        for (let i = 0; i < size; i++) {
            result.push(this.schema.deserialize(reader));
        }

        return result;
    }
}