import { FnkBorshReader } from '../deserializer';
import { FnkBorshWriter } from '../serializer';
import { FnkBorshSchema } from '../borsh';

export function Struct<S extends ReadonlyArray<StructField>>(
    schema: S
): StructSchema<S> {
    return new StructSchema<S>(schema);
}

export class StructSchema<S extends ReadonlyArray<StructField>>
    implements FnkBorshSchema<FromStructSchema<S>>
{
    readonly schema: S;

    // CONSTRUCTOR ------------------------------------------------------------

    constructor(schema: S) {
        this.schema = schema;
    }

    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: FromStructSchema<S>) {
        for (const field of this.schema) {
            const fieldValue = (value as any)[field[0]];
            field[1].serialize(writer, fieldValue);
        }
    }

    deserialize(reader: FnkBorshReader): FromStructSchema<S> {
        const result: Record<string, any> = {};

        for (const field of this.schema) {
            result[field[0]] = field[1].deserialize(reader);
        }

        return result as any;
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export type StructField = readonly [string, FnkBorshSchema<any>];

export type FromStructSchema<S extends ReadonlyArray<StructField>> = IsZero<
    S['length']
> extends true
    ? {}
    : UnionToIntersection<ToObjectsArray<S>[number]>;

type ToObject<T> = T extends readonly [infer K, infer Ty]
    ? K extends PropertyKey
        ? { [P in K]: UnwrapSchemaType<Ty> }
        : never
    : never;

type ToObjectsArray<T> = {
    [I in keyof T]: ToObject<T[I]>;
};

type UnionToIntersection<U> = (U extends any ? (k: U) => void : never) extends (
    k: infer I
) => void
    ? I
    : never;

type IsZero<N extends number> = N extends 0 ? true : false;

export type UnwrapSchemaType<S> = S extends FnkBorshSchema<infer T> ? T : never;
