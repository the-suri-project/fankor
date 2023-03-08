import { FnkBorshReader } from '../deserializer';
import { FnkBorshWriter } from '../serializer';
import { FnkBorshError } from '../errors';
import { FnkBorshSchema } from '../borsh';
import { FromEnumSchema } from './enums';

export function TAccountEnum<S extends ReadonlyArray<AccountEnumVariant>>(
    schema: S
): AccountEnumSchema<S> {
    return new AccountEnumSchema<S>(schema);
}

export class AccountEnumSchema<S extends ReadonlyArray<AccountEnumVariant>>
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
                variant[2].serialize(writer, (value as any).value);

                return;
            }
        }

        throw new FnkBorshError(`Enum variant not found for value: ${value}`);
    }

    deserialize(reader: FnkBorshReader): FromEnumSchema<S> {
        const discriminant = reader.peekByte();

        for (const variant of this.schema) {
            if (variant[0] === discriminant) {
                return {
                    type: variant[1],
                    value: variant[2].deserialize(reader),
                } as any;
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

export type AccountEnumVariant = readonly [number, string, FnkBorshSchema<any>];
