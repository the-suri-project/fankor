export type Either<L, R> = EitherLeft<L> | EitherRight<R>;

export interface EitherLeft<T> {
    type: 'Left';
    value: T;
}

export interface EitherRight<T> {
    type: 'Right';
    value: T;
}