import { FnkBorshReader } from '../deserializer';
import { FnkBorshWriter } from '../serializer';
import { FnkBorshError } from '../errors';
import { FnkBorshSchema } from '../index';

export type RustOption<T> = { type: 'some'; value: T } | { type: 'none' };

export function Option<T, S extends FnkBorshSchema<T>>(schema: S) {
    return new OptionSchema(schema);
}

export class OptionSchema<T, S extends FnkBorshSchema<T>>
    implements FnkBorshSchema<RustOption<T>>
{
    readonly schema: S;

    // CONSTRUCTOR ------------------------------------------------------------

    constructor(schema: S) {
        this.schema = schema;
    }

    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: RustOption<T>) {
        if (value.type === 'none') {
            writer.writeByte(0);
        } else {
            writer.writeByte(1);
            this.schema.serialize(writer, value.value);
        }
    }

    deserialize(reader: FnkBorshReader): RustOption<T> {
        const discriminant = reader.readByte();

        if (discriminant === 0) {
            return { type: 'none' };
        } else if (discriminant === 1) {
            return {
                type: 'some',
                value: this.schema.deserialize(reader),
            };
        } else {
            throw new FnkBorshError(
                `Invalid discriminant ${discriminant} for Option`
            );
        }
    }
}
