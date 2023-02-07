import { FnkBorshReader } from '../deserializer';
import { FnkBorshWriter } from '../serializer';
import { FnkBorshError } from '../errors';
import { FnkBorshSchema } from '../borsh';
import { InferFnkBorshSchemaInner } from './maps';

export type RustOption<T> = { type: 'Some'; value: T } | { type: 'None' };

export function Option<S extends FnkBorshSchema<any>>(schema: S) {
    return new OptionSchema(schema);
}

export class OptionSchema<S extends FnkBorshSchema<any>>
    implements FnkBorshSchema<RustOption<InferFnkBorshSchemaInner<S>>>
{
    readonly schema: S;

    // CONSTRUCTOR ------------------------------------------------------------

    constructor(schema: S) {
        this.schema = schema;
    }

    // METHODS ----------------------------------------------------------------

    serialize(
        writer: FnkBorshWriter,
        value: RustOption<InferFnkBorshSchemaInner<S>>
    ) {
        if (value.type === 'None') {
            writer.writeByte(0);
        } else {
            writer.writeByte(1);
            this.schema.serialize(writer, value.value);
        }
    }

    deserialize(
        reader: FnkBorshReader
    ): RustOption<InferFnkBorshSchemaInner<S>> {
        const discriminant = reader.readByte();

        if (discriminant === 0) {
            return { type: 'None' };
        } else if (discriminant === 1) {
            return {
                type: 'Some',
                value: this.schema.deserialize(reader),
            };
        } else {
            throw new FnkBorshError(
                `Invalid discriminant ${discriminant} for Option`
            );
        }
    }
}
