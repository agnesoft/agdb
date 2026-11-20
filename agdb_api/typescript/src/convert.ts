import type { Components } from "./openapi.d.ts";

type NativeValue = boolean | string | number | string[] | number[];
type BuilderDbValue = NativeValue | Components.Schemas.DbValue;
type BuilderDbKeyValue = BuilderDbValue[] | Components.Schemas.DbKeyValue;

export function convertToNativeValue(
    value: Components.Schemas.DbValue,
): NativeValue | undefined {
    if ("Bytes" in value) {
        return value.Bytes;
    }

    if ("String" in value) {
        if (value.String === "true") {
            return true;
        }

        if (value.String === "false") {
            return false;
        }

        return value.String;
    }

    if ("I64" in value) {
        return value.I64;
    }

    if ("U64" in value) {
        return value.U64;
    }

    if ("F64" in value) {
        return value.F64;
    }

    if ("VecString" in value) {
        return value.VecString;
    }

    if ("VecI64" in value) {
        return value.VecI64;
    }

    if ("VecU64" in value) {
        return value.VecU64;
    }

    if ("VecF64" in value) {
        return value.VecF64;
    }
}

export function convertTo<T extends Record<string, unknown>>(
    result: Components.Schemas.QueryResult,
): T | T[] {
    const res: T[] = [];

    for (const e of result.elements) {
        const obj: T = {} as T;
        const record = obj as Record<string, unknown>;

        for (const kv of e.values) {
            record["db_id"] = e.id;
            record[convertToNativeValue(kv.key) as string] =
                convertToNativeValue(kv.value);
        }

        res.push(obj);
    }

    if (res.length === 1) {
        return res[0];
    }

    return res;
}

export function convertToDbValue(
    value:
        | string
        | boolean
        | number
        | number[]
        | string[]
        | Components.Schemas.DbValue
        | object,
): Components.Schemas.DbValue {
    if (typeof value === "string") {
        return { String: value };
    }

    if (typeof value === "boolean") {
        return { String: value ? "true" : "false" };
    }

    if (typeof value === "number") {
        if (Number.isInteger(value)) {
            return { I64: value };
        }

        return { F64: value };
    }

    if (Array.isArray(value)) {
        if (value.every((item): item is number => typeof item === "number")) {
            if (value.every(Number.isInteger)) {
                return { VecI64: value.map(Number) };
            }

            return { VecF64: value.map(Number) };
        }

        return { VecString: value as string[] };
    }

    return value as Components.Schemas.DbValue;
}

export function convertToDbKeyValue(
    key_value: BuilderDbKeyValue,
): Components.Schemas.DbKeyValue {
    if (Array.isArray(key_value)) {
        return {
            key: convertToDbValue(key_value[0]),
            value: convertToDbValue(key_value[1]),
        };
    }

    return key_value;
}
