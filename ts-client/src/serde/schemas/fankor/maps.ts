import {FnkBorshWriter, FnkBorshWriteSchema} from '../../serializer';
import {FnkBorshReader, FnkBorshReadSchema} from '../../deserializer';
import {FnkUIntSchema} from './unsigned';
import {RustMap} from '../maps';
import {FnkBorshSchema} from '../../index';

export function FnkMap<K, V, Sk extends FnkBorshSchema<K>, Sv extends FnkBorshSchema<V>>({
    keySchema,
    valueSchema,
}: {
    keySchema: Sk; valueSchema: Sv
}) {
    return new FnkMapSchema(keySchema, valueSchema);
}

export class FnkMapSchema<K, V, Sk extends FnkBorshSchema<K>, Sv extends FnkBorshSchema<V>>
    implements FnkBorshReadSchema<RustMap<K, V>>, FnkBorshWriteSchema<RustMap<K, V>> {
    readonly keySchema: Sk;
    readonly valueSchema: Sv;

    // CONSTRUCTOR ------------------------------------------------------------

    constructor(keySchema: Sk, valueSchema: Sv) {
        this.keySchema = keySchema;
        this.valueSchema = valueSchema;
    }

    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: RustMap<K, V>) {
        new FnkUIntSchema().serialize(writer, value.length);

        for (const item of value) {
            this.keySchema.serialize(writer, item.key);
            this.valueSchema.serialize(writer, item.value);
        }
    }

    deserialize(reader: FnkBorshReader): RustMap<K, V> {
        const size = new FnkUIntSchema().deserialize(reader).toNumber();
        const result: RustMap<K, V> = [];

        for (let i = 0; i < size; i++) {
            const key = this.keySchema.deserialize(reader);
            const value = this.valueSchema.deserialize(reader);

            result.push({
                key,
                value,
            });
        }

        return result;
    }
}