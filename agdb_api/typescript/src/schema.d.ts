import type {
  OpenAPIClient,
  Parameters,
  UnknownParamsObject,
  OperationResponse,
  AxiosRequestConfig,
} from 'openapi-client-axios';

declare namespace Components {
    namespace Schemas {
        export interface ChangePassword {
            new_password: string;
            password: string;
        }
        export interface ClusterStatus {
            address: string;
            commit: number; // int64
            leader: boolean;
            status: boolean;
            term: number; // int64
        }
        /**
         * Comparison of database values ([`DbValue`]) used
         * by `key()` condition. Supports
         * the usual set of named comparisons: `==, !=, <, <=, >, =>`
         * plus `contains()`. The comparisons are type
         * strict except for the `contains` comparison
         * which allows vectorized version of the base type. Notably
         * however it does not support the `bytes` and integral types
         * where the "contains" makes little sense (i.e. does 3 contain 1?).
         */
        export type Comparison = /**
         * Comparison of database values ([`DbValue`]) used
         * by `key()` condition. Supports
         * the usual set of named comparisons: `==, !=, <, <=, >, =>`
         * plus `contains()`. The comparisons are type
         * strict except for the `contains` comparison
         * which allows vectorized version of the base type. Notably
         * however it does not support the `bytes` and integral types
         * where the "contains" makes little sense (i.e. does 3 contain 1?).
         */
        {
            Equal: /**
             * Database value is a strongly types value.
             *
             * It is an enum of limited number supported types
             * that are universal across all platforms
             * and programming languages.
             *
             * The value is constructible from large number of
             * raw types or associated types (e.g. i32, &str, etc.).
             * Getting the raw value back as string can be done
             * with `to_string()` but otherwise requires a `match`.
             */
            DbValue;
        } | {
            GreaterThan: /**
             * Database value is a strongly types value.
             *
             * It is an enum of limited number supported types
             * that are universal across all platforms
             * and programming languages.
             *
             * The value is constructible from large number of
             * raw types or associated types (e.g. i32, &str, etc.).
             * Getting the raw value back as string can be done
             * with `to_string()` but otherwise requires a `match`.
             */
            DbValue;
        } | {
            GreaterThanOrEqual: /**
             * Database value is a strongly types value.
             *
             * It is an enum of limited number supported types
             * that are universal across all platforms
             * and programming languages.
             *
             * The value is constructible from large number of
             * raw types or associated types (e.g. i32, &str, etc.).
             * Getting the raw value back as string can be done
             * with `to_string()` but otherwise requires a `match`.
             */
            DbValue;
        } | {
            LessThan: /**
             * Database value is a strongly types value.
             *
             * It is an enum of limited number supported types
             * that are universal across all platforms
             * and programming languages.
             *
             * The value is constructible from large number of
             * raw types or associated types (e.g. i32, &str, etc.).
             * Getting the raw value back as string can be done
             * with `to_string()` but otherwise requires a `match`.
             */
            DbValue;
        } | {
            LessThanOrEqual: /**
             * Database value is a strongly types value.
             *
             * It is an enum of limited number supported types
             * that are universal across all platforms
             * and programming languages.
             *
             * The value is constructible from large number of
             * raw types or associated types (e.g. i32, &str, etc.).
             * Getting the raw value back as string can be done
             * with `to_string()` but otherwise requires a `match`.
             */
            DbValue;
        } | {
            NotEqual: /**
             * Database value is a strongly types value.
             *
             * It is an enum of limited number supported types
             * that are universal across all platforms
             * and programming languages.
             *
             * The value is constructible from large number of
             * raw types or associated types (e.g. i32, &str, etc.).
             * Getting the raw value back as string can be done
             * with `to_string()` but otherwise requires a `match`.
             */
            DbValue;
        } | {
            Contains: /**
             * Database value is a strongly types value.
             *
             * It is an enum of limited number supported types
             * that are universal across all platforms
             * and programming languages.
             *
             * The value is constructible from large number of
             * raw types or associated types (e.g. i32, &str, etc.).
             * Getting the raw value back as string can be done
             * with `to_string()` but otherwise requires a `match`.
             */
            DbValue;
        };
        /**
         * Comparison of unsigned integers (`u64`) used
         * by `distance()` and `edge_count*()` conditions. Supports
         * the usual set of named comparisons: `==, !=, <, <=, >, =>`.
         */
        export type CountComparison = /**
         * Comparison of unsigned integers (`u64`) used
         * by `distance()` and `edge_count*()` conditions. Supports
         * the usual set of named comparisons: `==, !=, <, <=, >, =>`.
         */
        {
            /**
             * property == this
             */
            Equal: number; // int64
        } | {
            /**
             * property > this
             */
            GreaterThan: number; // int64
        } | {
            /**
             * property >= this
             */
            GreaterThanOrEqual: number; // int64
        } | {
            /**
             * property < this
             */
            LessThan: number; // int64
        } | {
            /**
             * property <= this
             */
            LessThanOrEqual: number; // int64
        } | {
            /**
             * property != this
             */
            NotEqual: number; // int64
        };
        export type DbAudit = QueryAudit[];
        /**
         * Database element used in [`QueryResult`]
         * that represents a node or an edge.
         */
        export interface DbElement {
            /**
             * Database id is a wrapper around `i64`.
             * The id is an identifier of a database element
             * both nodes and edges. The positive ids represent nodes,
             * negative ids represent edges. The value of `0` is
             * logically invalid (there cannot be element with id 0) and a default.
             */
            from?: number | null; // int64
            id: /**
             * Database id is a wrapper around `i64`.
             * The id is an identifier of a database element
             * both nodes and edges. The positive ids represent nodes,
             * negative ids represent edges. The value of `0` is
             * logically invalid (there cannot be element with id 0) and a default.
             */
            DbId /* int64 */;
            /**
             * Database id is a wrapper around `i64`.
             * The id is an identifier of a database element
             * both nodes and edges. The positive ids represent nodes,
             * negative ids represent edges. The value of `0` is
             * logically invalid (there cannot be element with id 0) and a default.
             */
            to?: number | null; // int64
            /**
             * List of key-value pairs associated with the element.
             */
            values: /**
             * Database key-value pair (aka property) attached to
             * database elements. It can be constructed from a
             * tuple of types that are convertible to `DbValue`.
             */
            DbKeyValue[];
        }
        /**
         * Database float is a wrapper around `f64` to provide
         * functionality like comparison. The comparison is
         * using `total_cmp` standard library function. See its
         * [docs](https://doc.rust-lang.org/std/primitive.f64.html#method.total_cmp)
         * to understand how it handles NaNs and other edge cases
         * of floating point numbers.
         */
        export type DbF64 = number; // double
        /**
         * Database id is a wrapper around `i64`.
         * The id is an identifier of a database element
         * both nodes and edges. The positive ids represent nodes,
         * negative ids represent edges. The value of `0` is
         * logically invalid (there cannot be element with id 0) and a default.
         */
        export type DbId = number; // int64
        /**
         * Ordering for search queries
         */
        export type DbKeyOrder = /* Ordering for search queries */ {
            Asc: /**
             * Database value is a strongly types value.
             *
             * It is an enum of limited number supported types
             * that are universal across all platforms
             * and programming languages.
             *
             * The value is constructible from large number of
             * raw types or associated types (e.g. i32, &str, etc.).
             * Getting the raw value back as string can be done
             * with `to_string()` but otherwise requires a `match`.
             */
            DbValue;
        } | {
            Desc: /**
             * Database value is a strongly types value.
             *
             * It is an enum of limited number supported types
             * that are universal across all platforms
             * and programming languages.
             *
             * The value is constructible from large number of
             * raw types or associated types (e.g. i32, &str, etc.).
             * Getting the raw value back as string can be done
             * with `to_string()` but otherwise requires a `match`.
             */
            DbValue;
        };
        /**
         * Database key-value pair (aka property) attached to
         * database elements. It can be constructed from a
         * tuple of types that are convertible to `DbValue`.
         */
        export interface DbKeyValue {
            key: /**
             * Database value is a strongly types value.
             *
             * It is an enum of limited number supported types
             * that are universal across all platforms
             * and programming languages.
             *
             * The value is constructible from large number of
             * raw types or associated types (e.g. i32, &str, etc.).
             * Getting the raw value back as string can be done
             * with `to_string()` but otherwise requires a `match`.
             */
            DbValue;
            value: /**
             * Database value is a strongly types value.
             *
             * It is an enum of limited number supported types
             * that are universal across all platforms
             * and programming languages.
             *
             * The value is constructible from large number of
             * raw types or associated types (e.g. i32, &str, etc.).
             * Getting the raw value back as string can be done
             * with `to_string()` but otherwise requires a `match`.
             */
            DbValue;
        }
        export type DbType = "memory" | "mapped" | "file";
        export interface DbTypeParam {
            db_type: DbType;
        }
        export interface DbUser {
            role: DbUserRole;
            user: string;
        }
        export type DbUserRole = "admin" | "write" | "read";
        export interface DbUserRoleParam {
            db_role: DbUserRole;
        }
        /**
         * Database value is a strongly types value.
         *
         * It is an enum of limited number supported types
         * that are universal across all platforms
         * and programming languages.
         *
         * The value is constructible from large number of
         * raw types or associated types (e.g. i32, &str, etc.).
         * Getting the raw value back as string can be done
         * with `to_string()` but otherwise requires a `match`.
         */
        export type DbValue = /**
         * Database value is a strongly types value.
         *
         * It is an enum of limited number supported types
         * that are universal across all platforms
         * and programming languages.
         *
         * The value is constructible from large number of
         * raw types or associated types (e.g. i32, &str, etc.).
         * Getting the raw value back as string can be done
         * with `to_string()` but otherwise requires a `match`.
         */
        {
            /**
             * Byte array, sometimes referred to as blob
             */
            Bytes: string; // binary
        } | {
            /**
             * 64-bit wide signed integer
             */
            I64: number; // int64
        } | {
            /**
             * 64-bit wide unsigned integer
             */
            U64: number; // int64
        } | {
            F64: /**
             * Database float is a wrapper around `f64` to provide
             * functionality like comparison. The comparison is
             * using `total_cmp` standard library function. See its
             * [docs](https://doc.rust-lang.org/std/primitive.f64.html#method.total_cmp)
             * to understand how it handles NaNs and other edge cases
             * of floating point numbers.
             */
            DbF64 /* double */;
        } | {
            /**
             * UTF-8 string
             */
            String: string;
        } | {
            /**
             * List of 64-bit wide signed integers
             */
            VecI64: number /* int64 */[];
        } | {
            /**
             * List of 64-bit wide unsigned integers
             */
            VecU64: number /* int64 */[];
        } | {
            /**
             * List of 64-bit floating point numbers
             */
            VecF64: /**
             * Database float is a wrapper around `f64` to provide
             * functionality like comparison. The comparison is
             * using `total_cmp` standard library function. See its
             * [docs](https://doc.rust-lang.org/std/primitive.f64.html#method.total_cmp)
             * to understand how it handles NaNs and other edge cases
             * of floating point numbers.
             */
            DbF64 /* double */[];
        } | {
            /**
             * List of UTF-8 strings
             */
            VecString: string[];
        };
        /**
         * Query to insert or update aliases of existing nodes.
         * All `ids` must exist. None of the `aliases` can be empty.
         * If there is an existing alias for any of the elements it
         * will be overwritten with a new one.
         *
         * NOTE: Setting `ids` to a search query will result in an error.
         *
         * The result will contain number of aliases inserted/updated but no elements.
         */
        export interface InsertAliasesQuery {
            /**
             * Aliases to be inserted
             */
            aliases: string[];
            ids: /**
             * List of database ids used in queries. It
             * can either represent a list of [`QueryId`]s
             * or a search query. Search query allows query
             * nesting and sourcing the ids dynamically for
             * another query most commonly with the
             * select queries.
             */
            QueryIds;
        }
        /**
         * Query to inserts edges to the database. The `from`
         * and `to` ids must exist in the database. There must be
         * enough `values` for all new edges unless set to `Single`
         * in which case they will be uniformly applied to all new
         * edges. The `each` flag is only useful if `from and `to` are
         * symmetric (same length) but you still want to connect every
         * origin to every destination. By default it would connect only
         * the pairs. For asymmetric inserts `each` is assumed.
         *
         * The result will contain number of edges inserted and elements with
         * their ids but no properties.
         */
        export interface InsertEdgesQuery {
            /**
             * If `true` create an edge between each origin
             * and destination.
             */
            each: boolean;
            from: /**
             * List of database ids used in queries. It
             * can either represent a list of [`QueryId`]s
             * or a search query. Search query allows query
             * nesting and sourcing the ids dynamically for
             * another query most commonly with the
             * select queries.
             */
            QueryIds;
            to: /**
             * List of database ids used in queries. It
             * can either represent a list of [`QueryId`]s
             * or a search query. Search query allows query
             * nesting and sourcing the ids dynamically for
             * another query most commonly with the
             * select queries.
             */
            QueryIds;
            values: /**
             * Helper type distinguishing uniform (`Single`) values
             * and multiple (`Multi`) values in database queries.
             */
            QueryValues;
        }
        export type InsertIndexQuery = /**
         * Database value is a strongly types value.
         *
         * It is an enum of limited number supported types
         * that are universal across all platforms
         * and programming languages.
         *
         * The value is constructible from large number of
         * raw types or associated types (e.g. i32, &str, etc.).
         * Getting the raw value back as string can be done
         * with `to_string()` but otherwise requires a `match`.
         */
        DbValue;
        /**
         * Query to insert nodes to the database. Only one of
         * `count`, `values` or `aliases` need to be given as the
         * implementation will derive the count from the other
         * parameters. If `values` is set to `Single` either `count`
         * or `aliases` must be provided however. If `values` are not
         * set to `Single` there must be enough value for `count/aliases`
         * unless they are not se and the count is derived from `values.
         *
         * The result will contain number of nodes inserted and elements with
         * their ids but no properties.
         */
        export interface InsertNodesQuery {
            /**
             * Aliases of the new nodes.
             */
            aliases: string[];
            /**
             * Number of nodes to be inserted.
             */
            count: number; // int64
            values: /**
             * Helper type distinguishing uniform (`Single`) values
             * and multiple (`Multi`) values in database queries.
             */
            QueryValues;
        }
        /**
         * Query to insert or update key-value pairs (properties)
         * to existing elements in the database. All `ids` must exist
         * in the database. If `values` is set to `Single` the properties
         * will be inserted uniformly to all `ids` otherwise there must be
         * enough `values` for all `ids`.
         *
         * The result will be number of inserted/update values and no elements.
         *
         * NOTE: The result is NOT number of affected elements but individual properties.
         */
        export interface InsertValuesQuery {
            ids: /**
             * List of database ids used in queries. It
             * can either represent a list of [`QueryId`]s
             * or a search query. Search query allows query
             * nesting and sourcing the ids dynamically for
             * another query most commonly with the
             * select queries.
             */
            QueryIds;
            values: /**
             * Helper type distinguishing uniform (`Single`) values
             * and multiple (`Multi`) values in database queries.
             */
            QueryValues;
        }
        export type Queries = /* Convenience enum for serializing/deserializing queries. */ QueryType[];
        export type QueriesResults = /**
         * Universal database result. Successful
         * execution of a query will always yield
         * this type. The `result` field is a numerical
         * representation of the result while the
         * `elements` are the list of `DbElement`s
         * with database ids and properties (key-value pairs).
         */
        QueryResult[];
        export interface QueryAudit {
            query: /* Convenience enum for serializing/deserializing queries. */ QueryType;
            timestamp: number; // int64
            user: string;
        }
        /**
         * Query condition. The condition consists of
         * `data`, logic operator and a modifier.
         */
        export interface QueryCondition {
            data: /* Query condition data */ QueryConditionData;
            logic: /* Logical operator for query conditions */ QueryConditionLogic;
            modifier: /* Query condition modifier */ QueryConditionModifier;
        }
        /**
         * Query condition data
         */
        export type QueryConditionData = /* Query condition data */ {
            Distance: /**
             * Comparison of unsigned integers (`u64`) used
             * by `distance()` and `edge_count*()` conditions. Supports
             * the usual set of named comparisons: `==, !=, <, <=, >, =>`.
             */
            CountComparison;
        } | ("Edge") | {
            EdgeCount: /**
             * Comparison of unsigned integers (`u64`) used
             * by `distance()` and `edge_count*()` conditions. Supports
             * the usual set of named comparisons: `==, !=, <, <=, >, =>`.
             */
            CountComparison;
        } | {
            EdgeCountFrom: /**
             * Comparison of unsigned integers (`u64`) used
             * by `distance()` and `edge_count*()` conditions. Supports
             * the usual set of named comparisons: `==, !=, <, <=, >, =>`.
             */
            CountComparison;
        } | {
            EdgeCountTo: /**
             * Comparison of unsigned integers (`u64`) used
             * by `distance()` and `edge_count*()` conditions. Supports
             * the usual set of named comparisons: `==, !=, <, <=, >, =>`.
             */
            CountComparison;
        } | {
            /**
             * Tests if the current id is in the list of ids.
             */
            Ids: /**
             * Database id used in queries that lets
             * you refer to a database element as numerical
             * id or a string alias.
             */
            QueryId[];
        } | {
            /**
             * Tests if the current element has a property `key`
             * with a value that evaluates true against `comparison`.
             */
            KeyValue: {
                key: /**
                 * Database value is a strongly types value.
                 *
                 * It is an enum of limited number supported types
                 * that are universal across all platforms
                 * and programming languages.
                 *
                 * The value is constructible from large number of
                 * raw types or associated types (e.g. i32, &str, etc.).
                 * Getting the raw value back as string can be done
                 * with `to_string()` but otherwise requires a `match`.
                 */
                DbValue;
                value: /**
                 * Comparison of database values ([`DbValue`]) used
                 * by `key()` condition. Supports
                 * the usual set of named comparisons: `==, !=, <, <=, >, =>`
                 * plus `contains()`. The comparisons are type
                 * strict except for the `contains` comparison
                 * which allows vectorized version of the base type. Notably
                 * however it does not support the `bytes` and integral types
                 * where the "contains" makes little sense (i.e. does 3 contain 1?).
                 */
                Comparison;
            };
        } | {
            /**
             * Test if the current element has **all** of the keys listed.
             */
            Keys: /**
             * Database value is a strongly types value.
             *
             * It is an enum of limited number supported types
             * that are universal across all platforms
             * and programming languages.
             *
             * The value is constructible from large number of
             * raw types or associated types (e.g. i32, &str, etc.).
             * Getting the raw value back as string can be done
             * with `to_string()` but otherwise requires a `match`.
             */
            DbValue[];
        } | ("Node") | {
            /**
             * Nested list of conditions (equivalent to brackets).
             */
            Where: /**
             * Query condition. The condition consists of
             * `data`, logic operator and a modifier.
             */
            QueryCondition[];
        };
        /**
         * Logical operator for query conditions
         */
        export type QueryConditionLogic = "And" | "Or";
        /**
         * Query condition modifier
         */
        export type QueryConditionModifier = "None" | "Beyond" | "Not" | "NotBeyond";
        /**
         * Database id used in queries that lets
         * you refer to a database element as numerical
         * id or a string alias.
         */
        export type QueryId = /**
         * Database id used in queries that lets
         * you refer to a database element as numerical
         * id or a string alias.
         */
        {
            Id: /**
             * Database id is a wrapper around `i64`.
             * The id is an identifier of a database element
             * both nodes and edges. The positive ids represent nodes,
             * negative ids represent edges. The value of `0` is
             * logically invalid (there cannot be element with id 0) and a default.
             */
            DbId /* int64 */;
        } | {
            /**
             * String alias
             */
            Alias: string;
        };
        /**
         * List of database ids used in queries. It
         * can either represent a list of [`QueryId`]s
         * or a search query. Search query allows query
         * nesting and sourcing the ids dynamically for
         * another query most commonly with the
         * select queries.
         */
        export type QueryIds = /**
         * List of database ids used in queries. It
         * can either represent a list of [`QueryId`]s
         * or a search query. Search query allows query
         * nesting and sourcing the ids dynamically for
         * another query most commonly with the
         * select queries.
         */
        {
            /**
             * List of [`QueryId`]s
             */
            Ids: /**
             * Database id used in queries that lets
             * you refer to a database element as numerical
             * id or a string alias.
             */
            QueryId[];
        } | {
            Search: /* Query to search for ids in the database following the graph. */ SearchQuery;
        };
        /**
         * Universal database result. Successful
         * execution of a query will always yield
         * this type. The `result` field is a numerical
         * representation of the result while the
         * `elements` are the list of `DbElement`s
         * with database ids and properties (key-value pairs).
         */
        export interface QueryResult {
            /**
             * List of elements yielded by the query
             * possibly with a list of properties.
             */
            elements: /**
             * Database element used in [`QueryResult`]
             * that represents a node or an edge.
             */
            DbElement[];
            /**
             * Query result
             */
            result: number; // int64
        }
        /**
         * Convenience enum for serializing/deserializing queries.
         */
        export type QueryType = /* Convenience enum for serializing/deserializing queries. */ {
            InsertAlias: /**
             * Query to insert or update aliases of existing nodes.
             * All `ids` must exist. None of the `aliases` can be empty.
             * If there is an existing alias for any of the elements it
             * will be overwritten with a new one.
             *
             * NOTE: Setting `ids` to a search query will result in an error.
             *
             * The result will contain number of aliases inserted/updated but no elements.
             */
            InsertAliasesQuery;
        } | {
            InsertEdges: /**
             * Query to inserts edges to the database. The `from`
             * and `to` ids must exist in the database. There must be
             * enough `values` for all new edges unless set to `Single`
             * in which case they will be uniformly applied to all new
             * edges. The `each` flag is only useful if `from and `to` are
             * symmetric (same length) but you still want to connect every
             * origin to every destination. By default it would connect only
             * the pairs. For asymmetric inserts `each` is assumed.
             *
             * The result will contain number of edges inserted and elements with
             * their ids but no properties.
             */
            InsertEdgesQuery;
        } | {
            InsertIndex: /**
             * Database value is a strongly types value.
             *
             * It is an enum of limited number supported types
             * that are universal across all platforms
             * and programming languages.
             *
             * The value is constructible from large number of
             * raw types or associated types (e.g. i32, &str, etc.).
             * Getting the raw value back as string can be done
             * with `to_string()` but otherwise requires a `match`.
             */
            DbValue;
        } | {
            InsertNodes: /**
             * Query to insert nodes to the database. Only one of
             * `count`, `values` or `aliases` need to be given as the
             * implementation will derive the count from the other
             * parameters. If `values` is set to `Single` either `count`
             * or `aliases` must be provided however. If `values` are not
             * set to `Single` there must be enough value for `count/aliases`
             * unless they are not se and the count is derived from `values.
             *
             * The result will contain number of nodes inserted and elements with
             * their ids but no properties.
             */
            InsertNodesQuery;
        } | {
            InsertValues: /**
             * Query to insert or update key-value pairs (properties)
             * to existing elements in the database. All `ids` must exist
             * in the database. If `values` is set to `Single` the properties
             * will be inserted uniformly to all `ids` otherwise there must be
             * enough `values` for all `ids`.
             *
             * The result will be number of inserted/update values and no elements.
             *
             * NOTE: The result is NOT number of affected elements but individual properties.
             */
            InsertValuesQuery;
        } | {
            Remove: /**
             * List of database ids used in queries. It
             * can either represent a list of [`QueryId`]s
             * or a search query. Search query allows query
             * nesting and sourcing the ids dynamically for
             * another query most commonly with the
             * select queries.
             */
            QueryIds;
        } | {
            RemoveAliases: /**
             * Query to remove aliases from the database. It
             * is not an error if an alias to be removed already
             * does not exist.
             *
             * The result will be a negative number signifying how
             * many aliases have been actually removed.
             */
            RemoveAliasesQuery;
        } | {
            RemoveIndex: /**
             * Database value is a strongly types value.
             *
             * It is an enum of limited number supported types
             * that are universal across all platforms
             * and programming languages.
             *
             * The value is constructible from large number of
             * raw types or associated types (e.g. i32, &str, etc.).
             * Getting the raw value back as string can be done
             * with `to_string()` but otherwise requires a `match`.
             */
            DbValue;
        } | {
            RemoveValues: /**
             * Query to select elements with only certain properties of
             * given ids. All ids must exist in the database and all
             * of them must have the requested properties.
             *
             * The result will be number of elements and the
             * list of elements with the requested properties.
             */
            SelectValuesQuery;
        } | {
            Search: /* Query to search for ids in the database following the graph. */ SearchQuery;
        } | {
            Select: /**
             * List of database ids used in queries. It
             * can either represent a list of [`QueryId`]s
             * or a search query. Search query allows query
             * nesting and sourcing the ids dynamically for
             * another query most commonly with the
             * select queries.
             */
            QueryIds;
        } | {
            SelectAliases: /**
             * List of database ids used in queries. It
             * can either represent a list of [`QueryId`]s
             * or a search query. Search query allows query
             * nesting and sourcing the ids dynamically for
             * another query most commonly with the
             * select queries.
             */
            QueryIds;
        } | {
            SelectAllAliases: /**
             * Query to select all aliases in the database.
             *
             * The result will be number of returned aliases and list
             * of elements with a single property `String("alias")` holding
             * the value `String`.
             */
            SelectAllAliasesQuery;
        } | {
            SelectEdgeCount: /**
             * Query to select number of edges of given node ids.
             * All of the ids must exist in the database. If any
             * of the ids is not a node the result will be 0 (not
             * an error).
             *
             * The result will be number of elements returned and the list
             * of elements with a single property `String("edge_count")` with
             * a value `u64`.
             *
             * NOTE: Self-referential edges are counted twice as if they
             * were coming from another edge. Therefore the edge count
             * might be greater than number of unique db elements.
             */
            SelectEdgeCountQuery;
        } | {
            SelectIndexes: /**
             * Query to select all indexes in the database.
             *
             * The result will be number of returned indexes and single element
             * with index 0 and the properties corresponding to the names of the indexes
             * (keys) with `u64` values representing number of indexed values in each
             * index.
             */
            SelectIndexesQuery;
        } | {
            SelectKeys: /**
             * List of database ids used in queries. It
             * can either represent a list of [`QueryId`]s
             * or a search query. Search query allows query
             * nesting and sourcing the ids dynamically for
             * another query most commonly with the
             * select queries.
             */
            QueryIds;
        } | {
            SelectKeyCount: /**
             * List of database ids used in queries. It
             * can either represent a list of [`QueryId`]s
             * or a search query. Search query allows query
             * nesting and sourcing the ids dynamically for
             * another query most commonly with the
             * select queries.
             */
            QueryIds;
        } | {
            SelectValues: /**
             * Query to select elements with only certain properties of
             * given ids. All ids must exist in the database and all
             * of them must have the requested properties.
             *
             * The result will be number of elements and the
             * list of elements with the requested properties.
             */
            SelectValuesQuery;
        };
        /**
         * Helper type distinguishing uniform (`Single`) values
         * and multiple (`Multi`) values in database queries.
         */
        export type QueryValues = /**
         * Helper type distinguishing uniform (`Single`) values
         * and multiple (`Multi`) values in database queries.
         */
        {
            /**
             * Single list of properties (key-value pairs)
             * to be applied to all elements in a query.
             */
            Single: /**
             * Database key-value pair (aka property) attached to
             * database elements. It can be constructed from a
             * tuple of types that are convertible to `DbValue`.
             */
            DbKeyValue[];
        } | {
            /**
             * List of lists of properties (key-value pairs)
             * to be applied to all elements in a query. There
             * must be as many lists of properties as ids
             * in a query.
             */
            Multi: /**
             * Database key-value pair (aka property) attached to
             * database elements. It can be constructed from a
             * tuple of types that are convertible to `DbValue`.
             */
            DbKeyValue[][];
        };
        /**
         * Query to remove aliases from the database. It
         * is not an error if an alias to be removed already
         * does not exist.
         *
         * The result will be a negative number signifying how
         * many aliases have been actually removed.
         */
        export type RemoveAliasesQuery = string[];
        export type RemoveIndexQuery = /**
         * Database value is a strongly types value.
         *
         * It is an enum of limited number supported types
         * that are universal across all platforms
         * and programming languages.
         *
         * The value is constructible from large number of
         * raw types or associated types (e.g. i32, &str, etc.).
         * Getting the raw value back as string can be done
         * with `to_string()` but otherwise requires a `match`.
         */
        DbValue;
        export type RemoveQuery = /**
         * List of database ids used in queries. It
         * can either represent a list of [`QueryId`]s
         * or a search query. Search query allows query
         * nesting and sourcing the ids dynamically for
         * another query most commonly with the
         * select queries.
         */
        QueryIds;
        export type RemoveValuesQuery = /**
         * Query to select elements with only certain properties of
         * given ids. All ids must exist in the database and all
         * of them must have the requested properties.
         *
         * The result will be number of elements and the
         * list of elements with the requested properties.
         */
        SelectValuesQuery;
        /**
         * Query to search for ids in the database following the graph.
         */
        export interface SearchQuery {
            algorithm: /* Search algorithm to be used */ SearchQueryAlgorithm;
            /**
             * Set of conditions every element must satisfy to be included in the
             * result. Some conditions also influence the search path as well.
             */
            conditions: /**
             * Query condition. The condition consists of
             * `data`, logic operator and a modifier.
             */
            QueryCondition[];
            destination: /**
             * Database id used in queries that lets
             * you refer to a database element as numerical
             * id or a string alias.
             */
            QueryId;
            /**
             * How many elements maximum to return.
             */
            limit: number; // int64
            /**
             * How many elements that would be returned should be
             * skipped in the result.
             */
            offset: number; // int64
            /**
             * Order of the elements in the result. The sorting happens before
             * `offset` and `limit` are applied.
             */
            order_by: /* Ordering for search queries */ DbKeyOrder[];
            origin: /**
             * Database id used in queries that lets
             * you refer to a database element as numerical
             * id or a string alias.
             */
            QueryId;
        }
        /**
         * Search algorithm to be used
         */
        export type SearchQueryAlgorithm = "BreadthFirst" | "DepthFirst" | "Index";
        export type SelectAliasesQuery = /**
         * List of database ids used in queries. It
         * can either represent a list of [`QueryId`]s
         * or a search query. Search query allows query
         * nesting and sourcing the ids dynamically for
         * another query most commonly with the
         * select queries.
         */
        QueryIds;
        /**
         * Query to select all aliases in the database.
         *
         * The result will be number of returned aliases and list
         * of elements with a single property `String("alias")` holding
         * the value `String`.
         */
        export interface SelectAllAliasesQuery {
        }
        /**
         * Query to select number of edges of given node ids.
         * All of the ids must exist in the database. If any
         * of the ids is not a node the result will be 0 (not
         * an error).
         *
         * The result will be number of elements returned and the list
         * of elements with a single property `String("edge_count")` with
         * a value `u64`.
         *
         * NOTE: Self-referential edges are counted twice as if they
         * were coming from another edge. Therefore the edge count
         * might be greater than number of unique db elements.
         */
        export interface SelectEdgeCountQuery {
            /**
             * If set to `true` the query will count outgoing edges
             * from the nodes.
             */
            from: boolean;
            ids: /**
             * List of database ids used in queries. It
             * can either represent a list of [`QueryId`]s
             * or a search query. Search query allows query
             * nesting and sourcing the ids dynamically for
             * another query most commonly with the
             * select queries.
             */
            QueryIds;
            /**
             * If set to `true` the query will count incoming edges
             * to the nodes.
             */
            to: boolean;
        }
        /**
         * Query to select all indexes in the database.
         *
         * The result will be number of returned indexes and single element
         * with index 0 and the properties corresponding to the names of the indexes
         * (keys) with `u64` values representing number of indexed values in each
         * index.
         */
        export interface SelectIndexesQuery {
        }
        export type SelectKeyCountQuery = /**
         * List of database ids used in queries. It
         * can either represent a list of [`QueryId`]s
         * or a search query. Search query allows query
         * nesting and sourcing the ids dynamically for
         * another query most commonly with the
         * select queries.
         */
        QueryIds;
        export type SelectKeysQuery = /**
         * List of database ids used in queries. It
         * can either represent a list of [`QueryId`]s
         * or a search query. Search query allows query
         * nesting and sourcing the ids dynamically for
         * another query most commonly with the
         * select queries.
         */
        QueryIds;
        export type SelectQuery = /**
         * List of database ids used in queries. It
         * can either represent a list of [`QueryId`]s
         * or a search query. Search query allows query
         * nesting and sourcing the ids dynamically for
         * another query most commonly with the
         * select queries.
         */
        QueryIds;
        /**
         * Query to select elements with only certain properties of
         * given ids. All ids must exist in the database and all
         * of them must have the requested properties.
         *
         * The result will be number of elements and the
         * list of elements with the requested properties.
         */
        export interface SelectValuesQuery {
            ids: /**
             * List of database ids used in queries. It
             * can either represent a list of [`QueryId`]s
             * or a search query. Search query allows query
             * nesting and sourcing the ids dynamically for
             * another query most commonly with the
             * select queries.
             */
            QueryIds;
            keys: /**
             * Database value is a strongly types value.
             *
             * It is an enum of limited number supported types
             * that are universal across all platforms
             * and programming languages.
             *
             * The value is constructible from large number of
             * raw types or associated types (e.g. i32, &str, etc.).
             * Getting the raw value back as string can be done
             * with `to_string()` but otherwise requires a `match`.
             */
            DbValue[];
        }
        export interface ServerDatabase {
            backup: number; // int64
            db_type: DbType;
            name: string;
            role: DbUserRole;
            size: number; // int64
        }
        export interface ServerDatabaseRename {
            new_name: string;
        }
        export interface StatusParams {
            cluster?: boolean | null;
        }
        export interface UserCredentials {
            password: string;
        }
        export interface UserLogin {
            password: string;
            username: string;
        }
        export interface UserStatus {
            name: string;
        }
    }
}
declare namespace Paths {
    namespace AdminDbAdd {
        namespace Parameters {
            export type Db = string;
            export type DbType = Components.Schemas.DbType;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        export interface QueryParameters {
            db_type: Parameters.DbType;
        }
        namespace Responses {
            export interface $201 {
            }
            export interface $401 {
            }
            export interface $404 {
            }
            export interface $465 {
            }
        }
    }
    namespace AdminDbAudit {
        namespace Parameters {
            export type Db = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        namespace Responses {
            export type $200 = Components.Schemas.DbAudit;
            export interface $401 {
            }
        }
    }
    namespace AdminDbBackup {
        namespace Parameters {
            export type Db = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        namespace Responses {
            export interface $201 {
            }
            export interface $401 {
            }
            export interface $403 {
            }
            export interface $404 {
            }
        }
    }
    namespace AdminDbCopy {
        namespace Parameters {
            export type Db = string;
            export type NewName = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        export interface QueryParameters {
            new_name: Parameters.NewName;
        }
        namespace Responses {
            export interface $201 {
            }
            export interface $401 {
            }
            export interface $404 {
            }
            export interface $465 {
            }
            export interface $467 {
            }
        }
    }
    namespace AdminDbDelete {
        namespace Parameters {
            export type Db = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        namespace Responses {
            export interface $204 {
            }
            export interface $401 {
            }
            export interface $404 {
            }
        }
    }
    namespace AdminDbExec {
        namespace Parameters {
            export type Db = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        export type RequestBody = Components.Schemas.Queries;
        namespace Responses {
            export type $200 = Components.Schemas.QueriesResults;
            export interface $401 {
            }
            export interface $403 {
            }
            export interface $404 {
            }
        }
    }
    namespace AdminDbList {
        namespace Responses {
            export type $200 = Components.Schemas.ServerDatabase[];
            export interface $401 {
            }
        }
    }
    namespace AdminDbOptimize {
        namespace Parameters {
            export type Db = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        namespace Responses {
            export type $200 = Components.Schemas.ServerDatabase;
            export interface $401 {
            }
        }
    }
    namespace AdminDbRemove {
        namespace Parameters {
            export type Db = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        namespace Responses {
            export interface $204 {
            }
            export interface $401 {
            }
            export interface $404 {
            }
        }
    }
    namespace AdminDbRename {
        namespace Parameters {
            export type Db = string;
            export type NewName = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        export interface QueryParameters {
            new_name: Parameters.NewName;
        }
        namespace Responses {
            export interface $201 {
            }
            export interface $401 {
            }
            export interface $404 {
            }
            export interface $465 {
            }
            export interface $467 {
            }
        }
    }
    namespace AdminDbRestore {
        namespace Parameters {
            export type Db = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        namespace Responses {
            export interface $201 {
            }
            export interface $401 {
            }
            export interface $404 {
            }
        }
    }
    namespace AdminDbUserAdd {
        namespace Parameters {
            export type Db = string;
            export type DbRole = Components.Schemas.DbUserRole;
            export type Owner = string;
            export type Username = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
            username: Parameters.Username;
        }
        export interface QueryParameters {
            db_role: Parameters.DbRole;
        }
        namespace Responses {
            export interface $201 {
            }
            export interface $401 {
            }
            export interface $403 {
            }
            export interface $404 {
            }
        }
    }
    namespace AdminDbUserList {
        namespace Parameters {
            export type Db = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        namespace Responses {
            export type $200 = Components.Schemas.DbUser[];
            export interface $401 {
            }
            export interface $404 {
            }
        }
    }
    namespace AdminDbUserRemove {
        namespace Parameters {
            export type Db = string;
            export type Owner = string;
            export type Username = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
            username: Parameters.Username;
        }
        namespace Responses {
            export interface $204 {
            }
            export interface $401 {
            }
            export interface $403 {
            }
            export interface $404 {
            }
        }
    }
    namespace AdminShutdown {
        namespace Responses {
            export interface $202 {
            }
            export interface $401 {
            }
        }
    }
    namespace AdminUserAdd {
        namespace Parameters {
            export type Username = string;
        }
        export interface PathParameters {
            username: Parameters.Username;
        }
        export type RequestBody = Components.Schemas.UserCredentials;
        namespace Responses {
            export interface $201 {
            }
            export interface $401 {
            }
            export interface $461 {
            }
            export interface $462 {
            }
            export interface $463 {
            }
        }
    }
    namespace AdminUserChangePassword {
        namespace Parameters {
            export type Username = string;
        }
        export interface PathParameters {
            username: Parameters.Username;
        }
        export type RequestBody = Components.Schemas.UserCredentials;
        namespace Responses {
            export interface $201 {
            }
            export interface $401 {
            }
            export interface $461 {
            }
            export interface $464 {
            }
        }
    }
    namespace AdminUserList {
        namespace Responses {
            export type $200 = Components.Schemas.UserStatus[];
            export interface $401 {
            }
        }
    }
    namespace AdminUserRemove {
        namespace Parameters {
            export type Username = string;
        }
        export interface PathParameters {
            username: Parameters.Username;
        }
        namespace Responses {
            export type $204 = Components.Schemas.UserStatus[];
            export interface $401 {
            }
            export interface $404 {
            }
        }
    }
    namespace DbAdd {
        namespace Parameters {
            export type Db = string;
            export type DbType = Components.Schemas.DbType;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        export interface QueryParameters {
            db_type: Parameters.DbType;
        }
        namespace Responses {
            export interface $201 {
            }
            export interface $401 {
            }
            export interface $403 {
            }
            export interface $465 {
            }
            export interface $467 {
            }
        }
    }
    namespace DbAudit {
        namespace Parameters {
            export type Db = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        namespace Responses {
            export type $200 = Components.Schemas.DbAudit;
            export interface $401 {
            }
            export interface $404 {
            }
        }
    }
    namespace DbBackup {
        namespace Parameters {
            export type Db = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        namespace Responses {
            export interface $201 {
            }
            export interface $401 {
            }
            export interface $403 {
            }
            export interface $404 {
            }
        }
    }
    namespace DbCopy {
        namespace Parameters {
            export type Db = string;
            export type NewName = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        export interface QueryParameters {
            new_name: Parameters.NewName;
        }
        namespace Responses {
            export interface $201 {
            }
            export interface $401 {
            }
            export interface $403 {
            }
            export interface $404 {
            }
            export interface $465 {
            }
            export interface $467 {
            }
        }
    }
    namespace DbDelete {
        namespace Parameters {
            export type Db = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        namespace Responses {
            export interface $204 {
            }
            export interface $401 {
            }
            export interface $403 {
            }
            export interface $404 {
            }
        }
    }
    namespace DbExec {
        namespace Parameters {
            export type Db = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        export type RequestBody = Components.Schemas.Queries;
        namespace Responses {
            export type $200 = Components.Schemas.QueriesResults;
            export interface $401 {
            }
            export interface $403 {
            }
            export interface $404 {
            }
        }
    }
    namespace DbList {
        namespace Responses {
            export type $200 = Components.Schemas.ServerDatabase[];
            export interface $401 {
            }
        }
    }
    namespace DbOptimize {
        namespace Parameters {
            export type Db = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        namespace Responses {
            export type $200 = Components.Schemas.ServerDatabase;
            export interface $401 {
            }
            export interface $403 {
            }
        }
    }
    namespace DbRemove {
        namespace Parameters {
            export type Db = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        namespace Responses {
            export interface $204 {
            }
            export interface $401 {
            }
            export interface $403 {
            }
            export interface $404 {
            }
        }
    }
    namespace DbRename {
        namespace Parameters {
            export type Db = string;
            export type NewName = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        export interface QueryParameters {
            new_name: Parameters.NewName;
        }
        namespace Responses {
            export interface $201 {
            }
            export interface $401 {
            }
            export interface $403 {
            }
            export interface $404 {
            }
            export interface $465 {
            }
            export interface $467 {
            }
        }
    }
    namespace DbRestore {
        namespace Parameters {
            export type Db = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        namespace Responses {
            export interface $201 {
            }
            export interface $401 {
            }
            export interface $403 {
            }
            export interface $404 {
            }
        }
    }
    namespace DbUserAdd {
        namespace Parameters {
            export type Db = string;
            export type DbRole = Components.Schemas.DbUserRole;
            export type Owner = string;
            export type Username = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
            username: Parameters.Username;
        }
        export interface QueryParameters {
            db_role: Parameters.DbRole;
        }
        namespace Responses {
            export interface $201 {
            }
            export interface $401 {
            }
            export interface $403 {
            }
            export interface $404 {
            }
        }
    }
    namespace DbUserList {
        namespace Parameters {
            export type Db = string;
            export type Owner = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
        }
        namespace Responses {
            export interface $200 {
            }
            export interface $401 {
            }
            export interface $404 {
            }
        }
    }
    namespace DbUserRemove {
        namespace Parameters {
            export type Db = string;
            export type Owner = string;
            export type Username = string;
        }
        export interface PathParameters {
            owner: Parameters.Owner;
            db: Parameters.Db;
            username: Parameters.Username;
        }
        namespace Responses {
            export interface $204 {
            }
            export interface $401 {
            }
            export interface $403 {
            }
            export interface $404 {
            }
        }
    }
    namespace Status {
        namespace Parameters {
            export type Cluster = boolean;
        }
        export interface PathParameters {
            cluster: Parameters.Cluster;
        }
        namespace Responses {
            export type $200 = Components.Schemas.ClusterStatus[];
        }
    }
    namespace UserChangePassword {
        export type RequestBody = Components.Schemas.ChangePassword;
        namespace Responses {
            export interface $201 {
            }
            export interface $401 {
            }
            export interface $461 {
            }
        }
    }
    namespace UserLogin {
        export type RequestBody = Components.Schemas.UserLogin;
        namespace Responses {
            export type $200 = string;
            export interface $401 {
            }
        }
    }
    namespace UserLogout {
        namespace Responses {
            export interface $201 {
            }
            export interface $401 {
            }
            export interface $404 {
            }
        }
    }
}

export interface OperationMethods {
  /**
   * admin_db_list
   */
  'admin_db_list'(
    parameters?: Parameters<UnknownParamsObject> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.AdminDbList.Responses.$200>
  /**
   * admin_db_add
   */
  'admin_db_add'(
    parameters?: Parameters<Paths.AdminDbAdd.QueryParameters & Paths.AdminDbAdd.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.AdminDbAdd.Responses.$201>
  /**
   * admin_db_audit
   */
  'admin_db_audit'(
    parameters?: Parameters<Paths.AdminDbAudit.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.AdminDbAudit.Responses.$200>
  /**
   * admin_db_backup
   */
  'admin_db_backup'(
    parameters?: Parameters<Paths.AdminDbBackup.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.AdminDbBackup.Responses.$201>
  /**
   * admin_db_copy
   */
  'admin_db_copy'(
    parameters?: Parameters<Paths.AdminDbCopy.QueryParameters & Paths.AdminDbCopy.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.AdminDbCopy.Responses.$201>
  /**
   * admin_db_delete
   */
  'admin_db_delete'(
    parameters?: Parameters<Paths.AdminDbDelete.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.AdminDbDelete.Responses.$204>
  /**
   * admin_db_exec
   */
  'admin_db_exec'(
    parameters?: Parameters<Paths.AdminDbExec.PathParameters> | null,
    data?: Paths.AdminDbExec.RequestBody,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.AdminDbExec.Responses.$200>
  /**
   * admin_db_optimize
   */
  'admin_db_optimize'(
    parameters?: Parameters<Paths.AdminDbOptimize.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.AdminDbOptimize.Responses.$200>
  /**
   * admin_db_remove
   */
  'admin_db_remove'(
    parameters?: Parameters<Paths.AdminDbRemove.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.AdminDbRemove.Responses.$204>
  /**
   * admin_db_rename
   */
  'admin_db_rename'(
    parameters?: Parameters<Paths.AdminDbRename.QueryParameters & Paths.AdminDbRename.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.AdminDbRename.Responses.$201>
  /**
   * admin_db_user_list
   */
  'admin_db_user_list'(
    parameters?: Parameters<Paths.AdminDbUserList.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.AdminDbUserList.Responses.$200>
  /**
   * admin_db_user_add
   */
  'admin_db_user_add'(
    parameters?: Parameters<Paths.AdminDbUserAdd.QueryParameters & Paths.AdminDbUserAdd.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.AdminDbUserAdd.Responses.$201>
  /**
   * admin_db_user_remove
   */
  'admin_db_user_remove'(
    parameters?: Parameters<Paths.AdminDbUserRemove.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.AdminDbUserRemove.Responses.$204>
  /**
   * admin_shutdown
   */
  'admin_shutdown'(
    parameters?: Parameters<UnknownParamsObject> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.AdminShutdown.Responses.$202>
  /**
   * admin_user_list
   */
  'admin_user_list'(
    parameters?: Parameters<UnknownParamsObject> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.AdminUserList.Responses.$200>
  /**
   * admin_user_add
   */
  'admin_user_add'(
    parameters?: Parameters<Paths.AdminUserAdd.PathParameters> | null,
    data?: Paths.AdminUserAdd.RequestBody,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.AdminUserAdd.Responses.$201>
  /**
   * admin_user_change_password
   */
  'admin_user_change_password'(
    parameters?: Parameters<Paths.AdminUserChangePassword.PathParameters> | null,
    data?: Paths.AdminUserChangePassword.RequestBody,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.AdminUserChangePassword.Responses.$201>
  /**
   * admin_user_remove
   */
  'admin_user_remove'(
    parameters?: Parameters<Paths.AdminUserRemove.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.AdminUserRemove.Responses.$204>
  /**
   * admin_db_restore
   */
  'admin_db_restore'(
    parameters?: Parameters<Paths.AdminDbRestore.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.AdminDbRestore.Responses.$201>
  /**
   * db_list
   */
  'db_list'(
    parameters?: Parameters<UnknownParamsObject> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.DbList.Responses.$200>
  /**
   * db_add
   */
  'db_add'(
    parameters?: Parameters<Paths.DbAdd.QueryParameters & Paths.DbAdd.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.DbAdd.Responses.$201>
  /**
   * db_audit
   */
  'db_audit'(
    parameters?: Parameters<Paths.DbAudit.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.DbAudit.Responses.$200>
  /**
   * db_backup
   */
  'db_backup'(
    parameters?: Parameters<Paths.DbBackup.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.DbBackup.Responses.$201>
  /**
   * db_copy
   */
  'db_copy'(
    parameters?: Parameters<Paths.DbCopy.QueryParameters & Paths.DbCopy.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.DbCopy.Responses.$201>
  /**
   * db_delete
   */
  'db_delete'(
    parameters?: Parameters<Paths.DbDelete.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.DbDelete.Responses.$204>
  /**
   * db_exec
   */
  'db_exec'(
    parameters?: Parameters<Paths.DbExec.PathParameters> | null,
    data?: Paths.DbExec.RequestBody,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.DbExec.Responses.$200>
  /**
   * db_optimize
   */
  'db_optimize'(
    parameters?: Parameters<Paths.DbOptimize.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.DbOptimize.Responses.$200>
  /**
   * db_remove
   */
  'db_remove'(
    parameters?: Parameters<Paths.DbRemove.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.DbRemove.Responses.$204>
  /**
   * db_rename
   */
  'db_rename'(
    parameters?: Parameters<Paths.DbRename.QueryParameters & Paths.DbRename.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.DbRename.Responses.$201>
  /**
   * db_restore
   */
  'db_restore'(
    parameters?: Parameters<Paths.DbRestore.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.DbRestore.Responses.$201>
  /**
   * db_user_list
   */
  'db_user_list'(
    parameters?: Parameters<Paths.DbUserList.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.DbUserList.Responses.$200>
  /**
   * db_user_add
   */
  'db_user_add'(
    parameters?: Parameters<Paths.DbUserAdd.QueryParameters & Paths.DbUserAdd.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.DbUserAdd.Responses.$201>
  /**
   * db_user_remove
   */
  'db_user_remove'(
    parameters?: Parameters<Paths.DbUserRemove.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.DbUserRemove.Responses.$204>
  /**
   * status
   */
  'status'(
    parameters?: Parameters<Paths.Status.PathParameters> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.Status.Responses.$200>
  /**
   * user_change_password
   */
  'user_change_password'(
    parameters?: Parameters<UnknownParamsObject> | null,
    data?: Paths.UserChangePassword.RequestBody,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.UserChangePassword.Responses.$201>
  /**
   * user_login
   */
  'user_login'(
    parameters?: Parameters<UnknownParamsObject> | null,
    data?: Paths.UserLogin.RequestBody,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.UserLogin.Responses.$200>
  /**
   * user_logout
   */
  'user_logout'(
    parameters?: Parameters<UnknownParamsObject> | null,
    data?: any,
    config?: AxiosRequestConfig  
  ): OperationResponse<Paths.UserLogout.Responses.$201>
}

export interface PathsDictionary {
  ['/api/v1/admin/db/list']: {
    /**
     * admin_db_list
     */
    'get'(
      parameters?: Parameters<UnknownParamsObject> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.AdminDbList.Responses.$200>
  }
  ['/api/v1/admin/db/{owner}/{db}/add']: {
    /**
     * admin_db_add
     */
    'post'(
      parameters?: Parameters<Paths.AdminDbAdd.QueryParameters & Paths.AdminDbAdd.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.AdminDbAdd.Responses.$201>
  }
  ['/api/v1/admin/db/{owner}/{db}/audit']: {
    /**
     * admin_db_audit
     */
    'get'(
      parameters?: Parameters<Paths.AdminDbAudit.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.AdminDbAudit.Responses.$200>
  }
  ['/api/v1/admin/db/{owner}/{db}/backup']: {
    /**
     * admin_db_backup
     */
    'post'(
      parameters?: Parameters<Paths.AdminDbBackup.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.AdminDbBackup.Responses.$201>
  }
  ['/api/v1/admin/db/{owner}/{db}/copy']: {
    /**
     * admin_db_copy
     */
    'post'(
      parameters?: Parameters<Paths.AdminDbCopy.QueryParameters & Paths.AdminDbCopy.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.AdminDbCopy.Responses.$201>
  }
  ['/api/v1/admin/db/{owner}/{db}/delete']: {
    /**
     * admin_db_delete
     */
    'delete'(
      parameters?: Parameters<Paths.AdminDbDelete.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.AdminDbDelete.Responses.$204>
  }
  ['/api/v1/admin/db/{owner}/{db}/exec']: {
    /**
     * admin_db_exec
     */
    'post'(
      parameters?: Parameters<Paths.AdminDbExec.PathParameters> | null,
      data?: Paths.AdminDbExec.RequestBody,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.AdminDbExec.Responses.$200>
  }
  ['/api/v1/admin/db/{owner}/{db}/optimize']: {
    /**
     * admin_db_optimize
     */
    'post'(
      parameters?: Parameters<Paths.AdminDbOptimize.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.AdminDbOptimize.Responses.$200>
  }
  ['/api/v1/admin/db/{owner}/{db}/remove']: {
    /**
     * admin_db_remove
     */
    'delete'(
      parameters?: Parameters<Paths.AdminDbRemove.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.AdminDbRemove.Responses.$204>
  }
  ['/api/v1/admin/db/{owner}/{db}/rename']: {
    /**
     * admin_db_rename
     */
    'post'(
      parameters?: Parameters<Paths.AdminDbRename.QueryParameters & Paths.AdminDbRename.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.AdminDbRename.Responses.$201>
  }
  ['/api/v1/admin/db/{owner}/{db}/user/list']: {
    /**
     * admin_db_user_list
     */
    'get'(
      parameters?: Parameters<Paths.AdminDbUserList.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.AdminDbUserList.Responses.$200>
  }
  ['/api/v1/admin/db/{owner}/{db}/user/{username}/add']: {
    /**
     * admin_db_user_add
     */
    'put'(
      parameters?: Parameters<Paths.AdminDbUserAdd.QueryParameters & Paths.AdminDbUserAdd.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.AdminDbUserAdd.Responses.$201>
  }
  ['/api/v1/admin/db/{owner}/{db}/user/{username}/remove']: {
    /**
     * admin_db_user_remove
     */
    'delete'(
      parameters?: Parameters<Paths.AdminDbUserRemove.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.AdminDbUserRemove.Responses.$204>
  }
  ['/api/v1/admin/shutdown']: {
    /**
     * admin_shutdown
     */
    'post'(
      parameters?: Parameters<UnknownParamsObject> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.AdminShutdown.Responses.$202>
  }
  ['/api/v1/admin/user/list']: {
    /**
     * admin_user_list
     */
    'get'(
      parameters?: Parameters<UnknownParamsObject> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.AdminUserList.Responses.$200>
  }
  ['/api/v1/admin/user/{username}/add']: {
    /**
     * admin_user_add
     */
    'post'(
      parameters?: Parameters<Paths.AdminUserAdd.PathParameters> | null,
      data?: Paths.AdminUserAdd.RequestBody,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.AdminUserAdd.Responses.$201>
  }
  ['/api/v1/admin/user/{username}/change_password']: {
    /**
     * admin_user_change_password
     */
    'put'(
      parameters?: Parameters<Paths.AdminUserChangePassword.PathParameters> | null,
      data?: Paths.AdminUserChangePassword.RequestBody,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.AdminUserChangePassword.Responses.$201>
  }
  ['/api/v1/admin/user/{username}/remove']: {
    /**
     * admin_user_remove
     */
    'delete'(
      parameters?: Parameters<Paths.AdminUserRemove.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.AdminUserRemove.Responses.$204>
  }
  ['/api/v1/db/admin/{owner}/{db}/restore']: {
    /**
     * admin_db_restore
     */
    'post'(
      parameters?: Parameters<Paths.AdminDbRestore.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.AdminDbRestore.Responses.$201>
  }
  ['/api/v1/db/list']: {
    /**
     * db_list
     */
    'get'(
      parameters?: Parameters<UnknownParamsObject> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.DbList.Responses.$200>
  }
  ['/api/v1/db/{owner}/{db}/add']: {
    /**
     * db_add
     */
    'post'(
      parameters?: Parameters<Paths.DbAdd.QueryParameters & Paths.DbAdd.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.DbAdd.Responses.$201>
  }
  ['/api/v1/db/{owner}/{db}/audit']: {
    /**
     * db_audit
     */
    'get'(
      parameters?: Parameters<Paths.DbAudit.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.DbAudit.Responses.$200>
  }
  ['/api/v1/db/{owner}/{db}/backup']: {
    /**
     * db_backup
     */
    'post'(
      parameters?: Parameters<Paths.DbBackup.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.DbBackup.Responses.$201>
  }
  ['/api/v1/db/{owner}/{db}/copy']: {
    /**
     * db_copy
     */
    'post'(
      parameters?: Parameters<Paths.DbCopy.QueryParameters & Paths.DbCopy.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.DbCopy.Responses.$201>
  }
  ['/api/v1/db/{owner}/{db}/delete']: {
    /**
     * db_delete
     */
    'post'(
      parameters?: Parameters<Paths.DbDelete.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.DbDelete.Responses.$204>
  }
  ['/api/v1/db/{owner}/{db}/exec']: {
    /**
     * db_exec
     */
    'post'(
      parameters?: Parameters<Paths.DbExec.PathParameters> | null,
      data?: Paths.DbExec.RequestBody,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.DbExec.Responses.$200>
  }
  ['/api/v1/db/{owner}/{db}/optimize']: {
    /**
     * db_optimize
     */
    'post'(
      parameters?: Parameters<Paths.DbOptimize.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.DbOptimize.Responses.$200>
  }
  ['/api/v1/db/{owner}/{db}/remove']: {
    /**
     * db_remove
     */
    'post'(
      parameters?: Parameters<Paths.DbRemove.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.DbRemove.Responses.$204>
  }
  ['/api/v1/db/{owner}/{db}/rename']: {
    /**
     * db_rename
     */
    'post'(
      parameters?: Parameters<Paths.DbRename.QueryParameters & Paths.DbRename.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.DbRename.Responses.$201>
  }
  ['/api/v1/db/{owner}/{db}/restore']: {
    /**
     * db_restore
     */
    'post'(
      parameters?: Parameters<Paths.DbRestore.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.DbRestore.Responses.$201>
  }
  ['/api/v1/db/{owner}/{db}/user/list']: {
    /**
     * db_user_list
     */
    'get'(
      parameters?: Parameters<Paths.DbUserList.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.DbUserList.Responses.$200>
  }
  ['/api/v1/db/{owner}/{db}/user/{username}/add']: {
    /**
     * db_user_add
     */
    'post'(
      parameters?: Parameters<Paths.DbUserAdd.QueryParameters & Paths.DbUserAdd.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.DbUserAdd.Responses.$201>
  }
  ['/api/v1/db/{owner}/{db}/user/{username}/remove']: {
    /**
     * db_user_remove
     */
    'post'(
      parameters?: Parameters<Paths.DbUserRemove.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.DbUserRemove.Responses.$204>
  }
  ['/api/v1/status']: {
    /**
     * status
     */
    'get'(
      parameters?: Parameters<Paths.Status.PathParameters> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.Status.Responses.$200>
  }
  ['/api/v1/user/change_password']: {
    /**
     * user_change_password
     */
    'put'(
      parameters?: Parameters<UnknownParamsObject> | null,
      data?: Paths.UserChangePassword.RequestBody,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.UserChangePassword.Responses.$201>
  }
  ['/api/v1/user/login']: {
    /**
     * user_login
     */
    'post'(
      parameters?: Parameters<UnknownParamsObject> | null,
      data?: Paths.UserLogin.RequestBody,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.UserLogin.Responses.$200>
  }
  ['/api/v1/user/logout']: {
    /**
     * user_logout
     */
    'post'(
      parameters?: Parameters<UnknownParamsObject> | null,
      data?: any,
      config?: AxiosRequestConfig  
    ): OperationResponse<Paths.UserLogout.Responses.$201>
  }
}

export type Client = OpenAPIClient<OperationMethods, PathsDictionary>
