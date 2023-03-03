// Helper types to correctly derive program types.
export type OptionalFields<
    T,
    Keys extends keyof T | ''
> = ExcludeFunctionProperties<
    Exclude<Keys, ''> extends keyof T
        ? Partial<Pick<T, Exclude<Keys, ''>>> & Omit<T, Exclude<Keys, ''>>
        : T
>;

export type ExcludeFunctionProperties<T> = Pick<
    T,
    {
        [K in keyof T]: T[K] extends Function ? never : K;
    }[keyof T]
>;
