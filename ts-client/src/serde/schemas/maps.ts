import { FnkBorshReader } from '../deserializer';
import { FnkBorshWriter } from '../serializer';
import { U32Schema } from './unsigned';
import { FnkBorshSchema } from '../borsh';

export type RustMap<K, V> = { key: K; value: V }[];

export function TMap<
    Sk extends FnkBorshSchema<any>,
    Sv extends FnkBorshSchema<any>
>({ keySchema, valueSchema }: { keySchema: Sk; valueSchema: Sv }) {
    return new MapSchema(keySchema, valueSchema);
}

export class MapSchema<
    Sk extends FnkBorshSchema<any>,
    Sv extends FnkBorshSchema<any>
> implements
        FnkBorshSchema<
            RustMap<InferFnkBorshSchemaInner<Sk>, InferFnkBorshSchemaInner<Sv>>
        >
{
    readonly keySchema: Sk;
    readonly valueSchema: Sv;

    // CONSTRUCTOR ------------------------------------------------------------

    constructor(keySchema: Sk, valueSchema: Sv) {
        this.keySchema = keySchema;
        this.valueSchema = valueSchema;
    }

    // METHODS ----------------------------------------------------------------

    serialize(
        writer: FnkBorshWriter,
        value: RustMap<
            InferFnkBorshSchemaInner<Sk>,
            InferFnkBorshSchemaInner<Sv>
        >
    ) {
        new U32Schema().serialize(writer, value.length);

        for (const item of value) {
            this.keySchema.serialize(writer, item.key);
            this.valueSchema.serialize(writer, item.value);
        }
    }

    deserialize(
        reader: FnkBorshReader
    ): RustMap<InferFnkBorshSchemaInner<Sk>, InferFnkBorshSchemaInner<Sv>> {
        const size = new U32Schema().deserialize(reader);
        const result: RustMap<
            InferFnkBorshSchemaInner<Sk>,
            InferFnkBorshSchemaInner<Sv>
        > = [];

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

export type InferFnkBorshSchemaInner<T> = T extends FnkBorshSchema<infer T2>
    ? T2
    : unknown;
