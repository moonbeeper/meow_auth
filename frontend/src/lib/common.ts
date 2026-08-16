import { formatDistance } from "date-fns";

/** A successful result */
export type Ok<T> = {
    result: T;
    error: null;
};

/** A sad fail result */
export type Err<E> = {
    result: null;
    error: E;
};

export type Result<T, E = Error> = Ok<T> | Err<E>;

/** Practically a wrapper for try catch (makes it prettier not seeing try catches everywhere) */
export const tryCatch = async <T, E = Error>(fun: () => Promise<T>): Promise<Result<T, E>> => {
    try {
        return {
            result: await fun(),
            error: null
        };
    } catch (e) {
        return {
            result: null,
            error: e as E
        };
    }
};

/** Get the human readable date (67 days ago) from something like "2026-08-13T23:30:17.390975Z" */
export const actionDate = (date: string): string => {
    const d = new Date(date);
    return formatDistance(d, new Date(), { addSuffix: true });
};
