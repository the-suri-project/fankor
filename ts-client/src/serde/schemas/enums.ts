import { FnkBorshReader } from '../deserializer';
import { FnkBorshWriter } from '../serializer';
import { FnkBorshError, FnkBorshSchema } from '../index';
import { UnwrapSchemaType } from './structs';

export function Enum<S extends ReadonlyArray<EnumVariant>>(
    schema: S
): EnumSchema<S> {
    return new EnumSchema<S>(schema);
}

export class EnumSchema<S extends ReadonlyArray<EnumVariant>>
    implements FnkBorshSchema<FromEnumSchema<S>>
{
    readonly schema: S;

    // CONSTRUCTOR ------------------------------------------------------------

    constructor(schema: S) {
        this.schema = schema;
    }

    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: FromEnumSchema<S>) {
        for (const variant of this.schema) {
            if (variant[1] === value.type) {
                writer.writeByte(variant[0]);
                variant[2].serialize(writer, value.value);
                return;
            }
        }

        throw new FnkBorshError(`Enum variant not found for value: ${value}`);
    }

    deserialize(reader: FnkBorshReader): FromEnumSchema<S> {
        const discriminant = reader.readByte();

        for (const variant of this.schema) {
            if (variant[0] === discriminant) {
                return {
                    type: variant[1],
                    value: variant[2].deserialize(reader),
                };
            }
        }

        throw new FnkBorshError(
            `Enum variant not found for discriminant: ${discriminant}`
        );
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export type EnumVariant = readonly [number, string, FnkBorshSchema<any>];

export type FromEnumSchema<S extends ReadonlyArray<EnumVariant>> = {
    type: string;
    value: FromEnumVariant<S>[number];
};

export type FromEnumVariant<S extends ReadonlyArray<EnumVariant>> = {
    [Index in keyof S]: UnwrapSchemaType<S[Index][2]>;
};
