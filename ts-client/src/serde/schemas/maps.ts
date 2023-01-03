import {FnkBorshReader} from '../deserializer';
import {FnkBorshWriter} from '../serializer';
import {U32Schema} from './unsigned';
import {FnkBorshSchema} from '../index';

export type RustMap<K, V> = { key: K, value: V }[];

export function TMap<K, V, Sk extends FnkBorshSchema<K>, Sv extends FnkBorshSchema<V>>({
    keySchema,
    valueSchema,
}: {
    keySchema: Sk; valueSchema: Sv
}) {
    return new MapSchema(keySchema, valueSchema);
}

export class MapSchema<K, V, Sk extends FnkBorshSchema<K>, Sv extends FnkBorshSchema<V>>
    implements FnkBorshSchema<RustMap<K, V>> {
    readonly keySchema: Sk;
    readonly valueSchema: Sv;

    // CONSTRUCTOR ------------------------------------------------------------

    constructor(keySchema: Sk, valueSchema: Sv) {
        this.keySchema = keySchema;
        this.valueSchema = valueSchema;
    }

    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: RustMap<K, V>) {
        new U32Schema().serialize(writer, value.length);

        for (const item of value) {
            this.keySchema.serialize(writer, item.key);
            this.valueSchema.serialize(writer, item.value);
        }
    }

    deserialize(reader: FnkBorshReader): RustMap<K, V> {
        const size = new U32Schema().deserialize(reader);
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