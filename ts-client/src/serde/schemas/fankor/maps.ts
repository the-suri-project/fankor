import { FnkBorshWriter } from '../../serializer';
import { FnkBorshReader } from '../../deserializer';
import { FnkUIntSchema } from './unsigned';
import { InferFnkBorshSchemaInner, RustMap } from '../maps';
import { FnkBorshSchema } from '../../borsh';

export function FnkMap<
    Sk extends FnkBorshSchema<any>,
    Sv extends FnkBorshSchema<any>
>({ keySchema, valueSchema }: { keySchema: Sk; valueSchema: Sv }) {
    return new FnkMapSchema(keySchema, valueSchema);
}

export class FnkMapSchema<
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
        new FnkUIntSchema().serialize(writer, value.length);

        for (const item of value) {
            this.keySchema.serialize(writer, item.key);
            this.valueSchema.serialize(writer, item.value);
        }
    }

    deserialize(
        reader: FnkBorshReader
    ): RustMap<InferFnkBorshSchemaInner<Sk>, InferFnkBorshSchemaInner<Sv>> {
        const size = new FnkUIntSchema().deserialize(reader).toNumber();
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
