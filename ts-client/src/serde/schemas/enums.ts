import { FnkBorshReader } from '../deserializer';
import { FnkBorshWriter } from '../serializer';
import { FnkBorshError } from '../errors';
import { FnkBorshSchema } from '../borsh';
import { UnwrapSchemaType } from './structs';

export function TEnum<S extends ReadonlyArray<EnumVariant>>(
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

                if (variant[2]) {
                    variant[2].serialize(writer, (value as any).value);
                }

                return;
            }
        }

        throw new FnkBorshError(`Enum variant not found for value: ${value}`);
    }

    deserialize(reader: FnkBorshReader): FromEnumSchema<S> {
        const discriminant = reader.readByte();

        for (const variant of this.schema) {
            if (variant[0] === discriminant) {
                if (variant[2]) {
                    return {
                        type: variant[1],
                        value: variant[2].deserialize(reader),
                    } as any;
                } else {
                    return {
                        type: variant[1],
                    } as any;
                }
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

export type EnumVariant =
    | readonly [number, string]
    | readonly [number, string, FnkBorshSchema<any>];

export type FromEnumSchema<S extends ReadonlyArray<EnumVariant>> = {
    [Index in keyof S]: FromEnumVariant<S[Index]>;
}[number];

export type FromEnumVariant<S extends EnumVariant> = S[2] extends {}
    ? {
          type: string;
          value: UnwrapSchemaType<S[2]>;
      }
    : {
          type: string;
      };
