// The `DbType` impls that `#[derive(DbType)]` and `#[derive(DbElement)]` make for the types of
// agdb/tests/query_builder_test.rs, which the generated tests pass to the builder.

import type { DbType } from "../src/query_builder.ts";

export const T_DbType: DbType = {
    db_keys: () => [{ String: "value1" }, { String: "value2" }],
    db_element_id: () => undefined,
};

export const E_DbType: DbType = {
    db_keys: () => [{ String: "value" }],
    db_element_id: () => ({ String: "E" }),
};
