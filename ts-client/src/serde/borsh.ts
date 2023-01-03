import { FnkBorshReadSchema } from './deserializer';
import { FnkBorshWriteSchema } from './serializer';

export type FnkBorshSchema<T> = FnkBorshReadSchema<T> & FnkBorshWriteSchema<T>;
