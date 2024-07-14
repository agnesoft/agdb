import type { Components } from "./schema";

type BuilderQueryId = number | string | Components.Schemas.QueryId;
type BuilderQueryIds =
    | BuilderQueryId
    | BuilderQueryId[]
    | Components.Schemas.QueryType
    | Components.Schemas.QueryResult
    | Components.Schemas.QueryIds;
type NativeValue = boolean | string | number | string[] | number[];
type BuilderDbValue = NativeValue | Components.Schemas.DbValue;
type BuilderDbKeyValue = BuilderDbValue[] | Components.Schemas.DbKeyValue;

function intoQueryIds(ids: BuilderQueryIds): Components.Schemas.QueryIds {
    if (Array.isArray(ids)) {
        return { Ids: ids.map((id) => intoQueryId(id)) };
    }

    if (
        typeof ids === "string" ||
        typeof ids === "number" ||
        "Id" in ids ||
        "Alias" in ids
    ) {
        return { Ids: [intoQueryId(ids)] };
    }

    if ("Search" in ids) {
        return ids;
    }

    if ("result" in ids) {
        return {
            Ids: ids.elements.map((elem) => {
                return { Id: elem.id };
            }),
        };
    }
}

function intoQueryId(id: BuilderQueryId): Components.Schemas.QueryId {
    if (typeof id === "number") {
        return { Id: id };
    }

    if (typeof id === "string") {
        return { Alias: id };
    }

    return id;
}

function intoAliases(aliases: string | string[]): string[] {
    if (typeof aliases === "string") {
        return [aliases];
    }

    return aliases;
}

function intoDbKeyOrder(
    keys: Components.Schemas.DbKeyOrder | Components.Schemas.DbKeyOrder[],
): Components.Schemas.DbKeyOrder[] {
    if (Array.isArray(keys)) {
        return keys;
    }

    return [keys];
}

export function convertToNativeValue(
    value: Components.Schemas.DbValue,
): NativeValue {
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

export function convertTo<T>(result: Components.Schemas.QueryResult): T | T[] {
    let res: T[] = [];

    for (let e of result.elements) {
        let obj: T = {} as T;

        for (let kv of e.values) {
            obj["db_id"] = e.id;
            obj[convertToNativeValue(kv.key) as string] = convertToNativeValue(
                kv.value,
            );
        }

        res.push(obj);
    }

    if (res.length === 1) {
        return res[0];
    }

    return res;
}

export function convertToDbValue(value: any): Components.Schemas.DbValue {
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
        if (value.every((item) => typeof item === "number")) {
            if (value.every(Number.isInteger)) {
                return { VecI64: value.map(Number) };
            }

            return { VecF64: value };
        }

        return { VecString: value };
    }

    if (value === null || value === undefined) {
        return undefined;
    }

    return value;
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

export class CountComparison {
    static Equal(value: number): Components.Schemas.CountComparison {
        return { Equal: value };
    }

    static GreaterThan(value: number): Components.Schemas.CountComparison {
        return { GreaterThan: value };
    }

    static GreaterThanOrEqual(
        value: number,
    ): Components.Schemas.CountComparison {
        return { GreaterThanOrEqual: value };
    }

    static LessThan(value: number): Components.Schemas.CountComparison {
        return { LessThan: value };
    }

    static LessThanOrEqual(value: number): Components.Schemas.CountComparison {
        return { LessThanOrEqual: value };
    }

    static NotEqual(value: number): Components.Schemas.CountComparison {
        return { NotEqual: value };
    }
}

export class Comparison {
    static Equal(value: BuilderDbValue): Components.Schemas.Comparison {
        return { Equal: convertToDbValue(value) };
    }

    static GreaterThan(value: BuilderDbValue): Components.Schemas.Comparison {
        return { GreaterThan: convertToDbValue(value) };
    }

    static GreaterThanOrEqual(
        value: BuilderDbValue,
    ): Components.Schemas.Comparison {
        return { GreaterThanOrEqual: convertToDbValue(value) };
    }

    static LessThan(value: BuilderDbValue): Components.Schemas.Comparison {
        return { LessThan: convertToDbValue(value) };
    }

    static LessThanOrEqual(
        value: BuilderDbValue,
    ): Components.Schemas.Comparison {
        return { LessThanOrEqual: convertToDbValue(value) };
    }

    static NotEqual(value: BuilderDbValue): Components.Schemas.Comparison {
        return { NotEqual: convertToDbValue(value) };
    }

    static Contains(value: BuilderDbValue): Components.Schemas.Comparison {
        return { Contains: convertToDbValue(value) };
    }
}

export class DbKeyOrder {
    static Asc(value: BuilderDbValue): Components.Schemas.DbKeyOrder {
        return { Asc: convertToDbValue(value) };
    }

    static Desc(value: BuilderDbValue): Components.Schemas.DbKeyOrder {
        return { Desc: convertToDbValue(value) };
    }
}

class InsertNodesAliasesBuilder {
    private data: Components.Schemas.InsertNodesQuery;

    constructor(data: Components.Schemas.InsertNodesQuery) {
        this.data = data;
    }

    values_uniform(values: BuilderDbKeyValue[]): InsertNodesValuesBuilder {
        this.data.values = { Single: values.map(convertToDbKeyValue) };
        return new InsertNodesValuesBuilder(this.data);
    }

    values(values: BuilderDbKeyValue[][]): InsertNodesValuesBuilder {
        this.data.values = {
            Multi: values.map((item) => item.map(convertToDbKeyValue)),
        };
        return new InsertNodesValuesBuilder(this.data);
    }

    query(): Components.Schemas.QueryType {
        return { InsertNodes: this.data };
    }
}

class InsertNodesBuilder {
    private data: Components.Schemas.InsertNodesQuery;

    constructor() {
        this.data = {
            count: 0,
            aliases: [],
            ids: { Ids: [] },
            values: {
                Single: [],
            },
        };
    }

    aliases(aliases: string | string[]): InsertNodesAliasesBuilder {
        this.data.aliases = intoAliases(aliases);
        return new InsertNodesAliasesBuilder(this.data);
    }

    count(count: number): InsertNodesCountBuilder {
        this.data.count = count;
        return new InsertNodesCountBuilder(this.data);
    }

    ids(ids: BuilderQueryIds) {
        this.data.ids = intoQueryIds(ids);
        return new InsertNodesIdsBuilder(this.data);
    }

    values(values: BuilderDbKeyValue[][]): InsertNodesValuesBuilder {
        this.data.values = {
            Multi: values.map((item) => item.map(convertToDbKeyValue)),
        };
        return new InsertNodesValuesBuilder(this.data);
    }
}

class InsertNodesIdsBuilder {
    private data: Components.Schemas.InsertNodesQuery;

    constructor(data: Components.Schemas.InsertNodesQuery) {
        this.data = data;
    }

    aliases(aliases: string | string[]): InsertNodesAliasesBuilder {
        this.data.aliases = intoAliases(aliases);
        return new InsertNodesAliasesBuilder(this.data);
    }

    count(count: number): InsertNodesCountBuilder {
        this.data.count = count;
        return new InsertNodesCountBuilder(this.data);
    }

    values_uniform(values: BuilderDbKeyValue[]): InsertNodesValuesBuilder {
        this.data.values = { Single: values.map(convertToDbKeyValue) };
        return new InsertNodesValuesBuilder(this.data);
    }

    values(values: BuilderDbKeyValue[][]): InsertNodesValuesBuilder {
        this.data.values = {
            Multi: values.map((item) => item.map(convertToDbKeyValue)),
        };
        return new InsertNodesValuesBuilder(this.data);
    }
}

class InsertNodesCountBuilder {
    private data: Components.Schemas.InsertNodesQuery;

    constructor(data: Components.Schemas.InsertNodesQuery) {
        this.data = data;
    }

    values_uniform(values: BuilderDbKeyValue[]): InsertNodesValuesBuilder {
        this.data.values = { Single: values.map(convertToDbKeyValue) };
        return new InsertNodesValuesBuilder(this.data);
    }

    query(): Components.Schemas.QueryType {
        return { InsertNodes: this.data };
    }
}

class InsertNodesValuesBuilder {
    private data: Components.Schemas.InsertNodesQuery;

    constructor(data: Components.Schemas.InsertNodesQuery) {
        this.data = data;
    }

    query(): Components.Schemas.QueryType {
        return { InsertNodes: this.data };
    }
}

class InsertEdgesValuesBuilder {
    private data: Components.Schemas.InsertEdgesQuery;

    constructor(query: Components.Schemas.InsertEdgesQuery) {
        this.data = query;
    }

    query(): Components.Schemas.QueryType {
        return { InsertEdges: this.data };
    }
}

class InsertEdgesToEachBuilder {
    private data: Components.Schemas.InsertEdgesQuery;

    constructor(query: Components.Schemas.InsertEdgesQuery) {
        this.data = query;
    }

    values(values: BuilderDbKeyValue[][]): InsertEdgesValuesBuilder {
        this.data.values = {
            Multi: values.map((item) => item.map(convertToDbKeyValue)),
        };
        return new InsertEdgesValuesBuilder(this.data);
    }

    values_uniform(values: BuilderDbKeyValue[]): InsertEdgesValuesBuilder {
        this.data.values = { Single: values.map(convertToDbKeyValue) };
        return new InsertEdgesValuesBuilder(this.data);
    }

    query(): Components.Schemas.QueryType {
        return { InsertEdges: this.data };
    }
}

class InsertEdgesToBuilder {
    private data: Components.Schemas.InsertEdgesQuery;

    constructor(query: Components.Schemas.InsertEdgesQuery) {
        this.data = query;
    }

    each(): InsertEdgesToEachBuilder {
        this.data.each = true;
        return new InsertEdgesToEachBuilder(this.data);
    }

    values(values: BuilderDbKeyValue[][]): InsertEdgesValuesBuilder {
        this.data.values = {
            Multi: values.map((item) => item.map(convertToDbKeyValue)),
        };
        return new InsertEdgesValuesBuilder(this.data);
    }

    values_uniform(values: BuilderDbKeyValue[]): InsertEdgesValuesBuilder {
        this.data.values = { Single: values.map(convertToDbKeyValue) };
        return new InsertEdgesValuesBuilder(this.data);
    }

    query(): Components.Schemas.QueryType {
        return { InsertEdges: this.data };
    }
}

class InsertEdgesFromBuilder {
    private data: Components.Schemas.InsertEdgesQuery;

    constructor(query: Components.Schemas.InsertEdgesQuery) {
        this.data = query;
    }

    to(ids: BuilderQueryIds): InsertEdgesToBuilder {
        this.data.to = intoQueryIds(ids);
        return new InsertEdgesToBuilder(this.data);
    }
}

class InsertEdgesBuilder {
    private data: Components.Schemas.InsertEdgesQuery;

    constructor() {
        this.data = {
            each: false,
            from: { Ids: [] },
            to: { Ids: [] },
            ids: { Ids: [] },
            values: { Single: [] },
        };
    }

    from(ids: BuilderQueryIds): InsertEdgesFromBuilder {
        this.data.from = intoQueryIds(ids);
        return new InsertEdgesFromBuilder(this.data);
    }

    ids(ids: BuilderQueryIds) {
        this.data.ids = intoQueryIds(ids);
        return new InsertEdgesIdsBuilder(this.data);
    }
}

class InsertEdgesIdsBuilder {
    private data: Components.Schemas.InsertEdgesQuery;

    constructor(data: Components.Schemas.InsertEdgesQuery) {
        this.data = data;
    }

    from(ids: BuilderQueryIds): InsertEdgesFromBuilder {
        this.data.from = intoQueryIds(ids);
        return new InsertEdgesFromBuilder(this.data);
    }
}

class InsertAliasesIdsBuilder {
    private data: Components.Schemas.InsertAliasesQuery;

    constructor(data: Components.Schemas.InsertAliasesQuery) {
        this.data = data;
    }

    query(): Components.Schemas.QueryType {
        return { InsertAlias: this.data };
    }
}

class InsertAliasesBuilder {
    private data: Components.Schemas.InsertAliasesQuery;

    constructor(aliases: string[]) {
        this.data = { aliases: aliases, ids: { Ids: [] } };
    }

    ids(ids: BuilderQueryIds): InsertAliasesIdsBuilder {
        this.data.ids = intoQueryIds(ids);
        return new InsertAliasesIdsBuilder(this.data);
    }
}

class InsertValuesIdsBuilder {
    private data: Components.Schemas.InsertValuesQuery;

    constructor(data: Components.Schemas.InsertValuesQuery) {
        this.data = data;
    }

    query(): Components.Schemas.QueryType {
        return { InsertValues: this.data };
    }
}

class InsertValuesBuilder {
    private data: Components.Schemas.InsertValuesQuery;

    constructor(data: Components.Schemas.InsertValuesQuery) {
        this.data = data;
    }

    ids(ids: BuilderQueryIds): InsertValuesIdsBuilder {
        this.data.ids = intoQueryIds(ids);
        return new InsertValuesIdsBuilder(this.data);
    }
}

class InsertIndexBuilder {
    private key: Components.Schemas.DbValue;

    constructor(key: Components.Schemas.DbValue) {
        this.key = key;
    }

    query(): Components.Schemas.QueryType {
        return { InsertIndex: this.key };
    }
}

class InsertBuilder {
    aliases(names: string | string[]): InsertAliasesBuilder {
        if (typeof names === "string") {
            return new InsertAliasesBuilder([names]);
        }

        return new InsertAliasesBuilder(names);
    }

    element(elem: any): InsertValuesIdsBuilder {
        return this.elements([elem]);
    }

    elements(elems: any[]): InsertValuesIdsBuilder {
        let data = {
            ids: { Ids: [] },
            values: { Multi: [] },
        };

        let multiItem: Components.Schemas.DbKeyValue[] = [];
        for (const elem of elems) {
            multiItem = [];
            for (const key of Object.keys(elem)) {
                if (key === "db_id") {
                    let id = elem[key];
                    if (typeof id === "number") {
                        data.ids.Ids.push({ Id: id });
                    } else if (typeof id === "string") {
                        data.ids.Ids.push({ Alias: id });
                    } else if (id === null || id === undefined) {
                        data.ids.Ids.push({ Id: 0 });
                    } else if ("Id" in id || "Alias" in id) {
                        data.ids.Ids.push(id);
                    } else {
                        throw "Invalid db_id type";
                    }
                } else {
                    let keyValue = convertToDbValue(key);
                    let valueValue = convertToDbValue(elem[key]);
                    if (keyValue !== undefined && valueValue !== undefined) {
                        multiItem.push({
                            key: keyValue,
                            value: valueValue,
                        });
                    }
                }
            }
            data.values.Multi.push(multiItem);
        }
        return new InsertValuesIdsBuilder(data);
    }

    edges(): InsertEdgesBuilder {
        return new InsertEdgesBuilder();
    }

    index(key: BuilderDbValue): InsertIndexBuilder {
        return new InsertIndexBuilder(convertToDbValue(key));
    }

    nodes(): InsertNodesBuilder {
        return new InsertNodesBuilder();
    }

    values(values: BuilderDbKeyValue[][]): InsertValuesBuilder {
        return new InsertValuesBuilder({
            ids: { Ids: [] },
            values: {
                Multi: values.map((item) => item.map(convertToDbKeyValue)),
            },
        });
    }

    values_uniform(vals: BuilderDbKeyValue[]): InsertValuesBuilder {
        return new InsertValuesBuilder({
            ids: { Ids: [] },
            values: { Single: vals.map(convertToDbKeyValue) },
        });
    }
}

class RemoveIdsBuilder {
    private data: Components.Schemas.RemoveQuery;

    constructor(data: Components.Schemas.RemoveQuery) {
        this.data = data;
    }

    query(): Components.Schemas.QueryType {
        return { Remove: this.data };
    }
}

class RemoveAliasesBuilder {
    private data: Components.Schemas.RemoveAliasesQuery;

    constructor(data: Components.Schemas.RemoveAliasesQuery) {
        this.data = data;
    }

    query(): Components.Schemas.QueryType {
        return { RemoveAliases: this.data };
    }
}

class RemoveValuesIdsBuilder {
    private data: Components.Schemas.RemoveValuesQuery;

    constructor(data: Components.Schemas.RemoveValuesQuery) {
        this.data = data;
    }

    query(): Components.Schemas.QueryType {
        return { RemoveValues: this.data };
    }
}

class RemoveValuesBuilder {
    private data: Components.Schemas.RemoveValuesQuery;

    constructor(data: Components.Schemas.RemoveValuesQuery) {
        this.data = data;
    }

    ids(ids: BuilderQueryIds): RemoveValuesIdsBuilder {
        this.data.ids = intoQueryIds(ids);
        return new RemoveValuesIdsBuilder(this.data);
    }
}

class RemoveIndexBuilder {
    private key: Components.Schemas.DbValue;

    constructor(key: Components.Schemas.DbValue) {
        this.key = key;
    }

    query(): Components.Schemas.QueryType {
        return { RemoveIndex: this.key };
    }
}

class RemoveBuilder {
    aliases(aliases: string | string[]): RemoveAliasesBuilder {
        if (typeof aliases === "string") {
            return new RemoveAliasesBuilder([aliases]);
        }

        return new RemoveAliasesBuilder(aliases);
    }

    ids(ids: BuilderQueryIds): RemoveIdsBuilder {
        if (Array.isArray(ids)) {
            return new RemoveIdsBuilder(intoQueryIds(ids));
        } else {
            return new RemoveIdsBuilder(intoQueryIds(ids));
        }
    }

    index(key: BuilderDbValue): RemoveIndexBuilder {
        return new RemoveIndexBuilder(convertToDbValue(key));
    }

    values(values: BuilderDbValue[]): RemoveValuesBuilder {
        return new RemoveValuesBuilder({
            ids: { Ids: [] },
            keys: values.map(convertToDbValue),
        });
    }
}

class SelectAliasesIdsBuilder {
    private data: Components.Schemas.SelectAliasesQuery;

    constructor(data: Components.Schemas.SelectAliasesQuery) {
        this.data = data;
    }

    query(): Components.Schemas.QueryType {
        return { SelectAliases: this.data };
    }
}

class SelectAliasesBuilder {
    private data: Components.Schemas.SelectAliasesQuery;

    constructor() {
        this.data = {
            Ids: [],
        };
    }

    ids(ids: BuilderQueryIds): SelectAliasesIdsBuilder {
        if (Array.isArray(ids)) {
            return new SelectAliasesIdsBuilder(intoQueryIds(ids));
        } else {
            return new SelectAliasesIdsBuilder(intoQueryIds(ids));
        }
    }

    query(): Components.Schemas.QueryType {
        return { SelectAllAliases: {} };
    }
}

class SelectIdsBuilder {
    private data: Components.Schemas.SelectQuery;

    constructor(data: Components.Schemas.SelectQuery) {
        this.data = data;
    }

    query(): Components.Schemas.QueryType {
        return { Select: this.data };
    }
}

class SelectValuesIdsBuilder {
    private data: Components.Schemas.SelectValuesQuery;

    constructor(data: Components.Schemas.SelectValuesQuery) {
        this.data = data;
    }

    query(): Components.Schemas.QueryType {
        return { SelectValues: this.data };
    }
}

class SelectValuesBuilder {
    private data: Components.Schemas.SelectValuesQuery;

    constructor(data: Components.Schemas.SelectValuesQuery) {
        this.data = data;
    }

    ids(ids: BuilderQueryIds): SelectValuesIdsBuilder {
        this.data.ids = intoQueryIds(ids);
        return new SelectValuesIdsBuilder(this.data);
    }
}

class SelectKeysIdsBuilder {
    private data: Components.Schemas.SelectKeysQuery;

    constructor(data: Components.Schemas.SelectKeysQuery) {
        this.data = data;
    }

    query(): Components.Schemas.QueryType {
        return { SelectKeys: this.data };
    }
}

class SelectKeysBuilder {
    private data: Components.Schemas.SelectKeysQuery;

    constructor(data: Components.Schemas.SelectKeysQuery) {
        this.data = data;
    }

    ids(ids: BuilderQueryIds): SelectKeysIdsBuilder {
        this.data = intoQueryIds(ids);
        return new SelectKeysIdsBuilder(this.data);
    }
}

class SelectKeyCountIdsBuilder {
    private data: Components.Schemas.SelectKeyCountQuery;

    constructor(data: Components.Schemas.SelectKeyCountQuery) {
        this.data = data;
    }

    query(): Components.Schemas.QueryType {
        return { SelectKeyCount: this.data };
    }
}

class SelectEdgeCountIdsBuilder {
    private data: Components.Schemas.SelectEdgeCountQuery;

    constructor(data: Components.Schemas.SelectEdgeCountQuery) {
        this.data = data;
    }

    query(): Components.Schemas.QueryType {
        return { SelectEdgeCount: this.data };
    }
}

class SelectEdgeCountBuilder {
    private data: Components.Schemas.SelectEdgeCountQuery;

    constructor(data: Components.Schemas.SelectEdgeCountQuery) {
        this.data = data;
    }

    ids(ids: BuilderQueryIds): SelectEdgeCountIdsBuilder {
        this.data.ids = intoQueryIds(ids);
        return new SelectEdgeCountIdsBuilder(this.data);
    }
}

class SelectKeyCountBuilder {
    private data: Components.Schemas.SelectKeyCountQuery;

    constructor(data: Components.Schemas.SelectKeyCountQuery) {
        this.data = data;
    }

    ids(ids: BuilderQueryIds): SelectKeyCountIdsBuilder {
        this.data = intoQueryIds(ids);
        return new SelectKeyCountIdsBuilder(this.data);
    }
}

class SelectIndexesBuilder {
    query(): Components.Schemas.QueryType {
        return { SelectIndexes: {} };
    }
}

class SelectNodeCountBuilder {
    query(): Components.Schemas.QueryType {
        return { SelectNodeCount: {} };
    }
}

class SelectBuilder {
    aliases(): SelectAliasesBuilder {
        return new SelectAliasesBuilder();
    }

    edge_count(): SelectEdgeCountBuilder {
        return new SelectEdgeCountBuilder({
            ids: { Ids: [] },
            from: true,
            to: true,
        });
    }

    edge_count_from(): SelectEdgeCountBuilder {
        return new SelectEdgeCountBuilder({
            ids: { Ids: [] },
            from: true,
            to: false,
        });
    }

    edge_count_to(): SelectEdgeCountBuilder {
        return new SelectEdgeCountBuilder({
            ids: { Ids: [] },
            from: false,
            to: true,
        });
    }

    ids(ids: BuilderQueryIds): SelectIdsBuilder {
        return new SelectIdsBuilder(intoQueryIds(ids));
    }

    indexes(): SelectIndexesBuilder {
        return new SelectIndexesBuilder();
    }

    keys(): SelectKeysBuilder {
        return new SelectKeysBuilder({ Ids: [] });
    }

    key_count(): SelectKeyCountBuilder {
        return new SelectKeyCountBuilder({ Ids: [] });
    }

    node_count(): SelectNodeCountBuilder {
        return new SelectNodeCountBuilder();
    }

    values(values: BuilderDbValue[]): SelectValuesBuilder {
        return new SelectValuesBuilder({
            ids: { Ids: [] },
            keys: values.map(convertToDbValue),
        });
    }
}

function collapse_conditions(
    conditions: Components.Schemas.QueryCondition[][],
): boolean {
    if (conditions.length > 1) {
        let last = conditions.pop();
        let current = conditions[conditions.length - 1];
        let last_condition = current[current.length - 1];
        last_condition.data = { Where: last };
        return true;
    }

    return false;
}

function push_condition(
    builder: SearchWhereBuilder,
    condition: Components.Schemas.QueryCondition,
): SearchWhereLogicBuilder {
    builder.conditions[builder.conditions.length - 1].push(condition);
    builder.modifier = "None";
    builder.logic = "And";
    return new SearchWhereLogicBuilder(builder);
}

class SearchWhereLogicBuilder {
    private data: SearchWhereBuilder;

    constructor(data: SearchWhereBuilder) {
        this.data = data;
    }

    end_where(): SearchWhereLogicBuilder {
        collapse_conditions(this.data.conditions);
        return this;
    }

    and(): SearchWhereBuilder {
        this.data.logic = "And";
        return this.data;
    }

    or(): SearchWhereBuilder {
        this.data.logic = "Or";
        return this.data;
    }

    query(): Components.Schemas.QueryType {
        do {} while (collapse_conditions(this.data.conditions));
        this.data.data.conditions = this.data.conditions[0];
        return { Search: this.data.data };
    }
}

class SearchWhereKeyBuilder {
    private data: SearchWhereBuilder;
    private key: Components.Schemas.DbValue;

    constructor(key: Components.Schemas.DbValue, data: SearchWhereBuilder) {
        this.key = key;
        this.data = data;
    }

    value(value: Components.Schemas.Comparison): SearchWhereLogicBuilder {
        return push_condition(this.data, {
            data: { KeyValue: { key: this.key, value: value } },
            logic: this.data.logic,
            modifier: this.data.modifier,
        });
    }
}

class SearchWhereBuilder {
    data: Components.Schemas.SearchQuery;
    modifier: Components.Schemas.QueryConditionModifier;
    logic: Components.Schemas.QueryConditionLogic;
    conditions: Components.Schemas.QueryCondition[][];

    constructor(data: Components.Schemas.SearchQuery) {
        this.data = data;
        this.logic = "And";
        this.modifier = "None";
        this.conditions = [[]];
    }

    beyond(): SearchWhereBuilder {
        this.modifier = "Beyond";
        return this;
    }

    distance(
        distance: Components.Schemas.CountComparison,
    ): SearchWhereLogicBuilder {
        return push_condition(this, {
            data: { Distance: distance },
            logic: this.logic,
            modifier: this.modifier,
        });
    }

    edge(): SearchWhereLogicBuilder {
        return push_condition(this, {
            data: "Edge",
            logic: this.logic,
            modifier: this.modifier,
        });
    }

    edge_count(
        count: Components.Schemas.CountComparison,
    ): SearchWhereLogicBuilder {
        return push_condition(this, {
            data: { EdgeCount: count },
            logic: this.logic,
            modifier: this.modifier,
        });
    }

    edge_count_from(
        count: Components.Schemas.CountComparison,
    ): SearchWhereLogicBuilder {
        return push_condition(this, {
            data: { EdgeCountFrom: count },
            logic: this.logic,
            modifier: this.modifier,
        });
    }

    edge_count_to(
        count: Components.Schemas.CountComparison,
    ): SearchWhereLogicBuilder {
        return push_condition(this, {
            data: { EdgeCountTo: count },
            logic: this.logic,
            modifier: this.modifier,
        });
    }

    ids(ids: BuilderQueryId | BuilderQueryId[]): SearchWhereLogicBuilder {
        let inner_ids = Array.isArray(ids)
            ? ids.map((id) => intoQueryId(id))
            : [intoQueryId(ids)];
        return push_condition(this, {
            data: { Ids: inner_ids },
            logic: this.logic,
            modifier: this.modifier,
        });
    }

    key(key: BuilderDbValue): SearchWhereKeyBuilder {
        return new SearchWhereKeyBuilder(convertToDbValue(key), this);
    }

    keys(keys: BuilderDbValue[]): SearchWhereLogicBuilder {
        return push_condition(this, {
            data: { Keys: keys.map(convertToDbValue) },
            logic: this.logic,
            modifier: this.modifier,
        });
    }

    node(): SearchWhereLogicBuilder {
        return push_condition(this, {
            data: "Node",
            logic: this.logic,
            modifier: this.modifier,
        });
    }

    not(): SearchWhereBuilder {
        this.modifier = "Not";
        return this;
    }

    not_beyond(): SearchWhereBuilder {
        this.modifier = "NotBeyond";
        return this;
    }

    where(): SearchWhereBuilder {
        this.conditions[this.conditions.length - 1].push({
            data: { Where: [] },
            logic: this.logic,
            modifier: this.modifier,
        });
        this.logic = "And";
        this.modifier = "None";
        this.conditions.push([]);
        return this;
    }
}

class SearchLimitBuilder {
    private data: Components.Schemas.SearchQuery;

    constructor(data: Components.Schemas.SearchQuery) {
        this.data = data;
    }

    where(): SearchWhereBuilder {
        return new SearchWhereBuilder(this.data);
    }

    query(): Components.Schemas.QueryType {
        return { Search: this.data };
    }
}

class SearchOffsetBuilder {
    private data: Components.Schemas.SearchQuery;

    constructor(data: Components.Schemas.SearchQuery) {
        this.data = data;
    }

    limit(limit: number): SearchLimitBuilder {
        this.data.limit = limit;
        return new SearchLimitBuilder(this.data);
    }

    where(): SearchWhereBuilder {
        return new SearchWhereBuilder(this.data);
    }

    query(): Components.Schemas.QueryType {
        return { Search: this.data };
    }
}

class SearchOrderBy {
    private data: Components.Schemas.SearchQuery;

    constructor(data: Components.Schemas.SearchQuery) {
        this.data = data;
    }

    limit(limit: number): SearchLimitBuilder {
        this.data.limit = limit;
        return new SearchLimitBuilder(this.data);
    }

    offset(offset: number): SearchOffsetBuilder {
        this.data.offset = offset;
        return new SearchOffsetBuilder(this.data);
    }

    where(): SearchWhereBuilder {
        return new SearchWhereBuilder(this.data);
    }

    query(): Components.Schemas.QueryType {
        return { Search: this.data };
    }
}

class SearchFromBuilder {
    private data: Components.Schemas.SearchQuery;

    constructor(data: Components.Schemas.SearchQuery) {
        this.data = data;
    }

    limit(limit: number): SearchLimitBuilder {
        this.data.limit = limit;
        return new SearchLimitBuilder(this.data);
    }

    offset(offset: number): SearchOffsetBuilder {
        this.data.offset = offset;
        return new SearchOffsetBuilder(this.data);
    }

    order_by(
        keys: Components.Schemas.DbKeyOrder | Components.Schemas.DbKeyOrder[],
    ): SearchOrderBy {
        this.data.order_by = intoDbKeyOrder(keys);
        return new SearchOrderBy(this.data);
    }

    to(id: BuilderQueryId): SearchToBuilder {
        this.data.destination = intoQueryId(id);
        return new SearchToBuilder(this.data);
    }

    where(): SearchWhereBuilder {
        return new SearchWhereBuilder(this.data);
    }

    query(): Components.Schemas.QueryType {
        return { Search: this.data };
    }
}

class SearchToBuilder {
    private data: Components.Schemas.SearchQuery;

    constructor(data: Components.Schemas.SearchQuery) {
        this.data = data;
    }

    limit(limit: number): SearchLimitBuilder {
        this.data.limit = limit;
        return new SearchLimitBuilder(this.data);
    }

    offset(offset: number): SearchOffsetBuilder {
        this.data.offset = offset;
        return new SearchOffsetBuilder(this.data);
    }

    order_by(
        keys: Components.Schemas.DbKeyOrder | Components.Schemas.DbKeyOrder[],
    ): SearchOrderBy {
        this.data.order_by = intoDbKeyOrder(keys);
        return new SearchOrderBy(this.data);
    }

    where(): SearchWhereBuilder {
        return new SearchWhereBuilder(this.data);
    }

    query(): Components.Schemas.QueryType {
        return { Search: this.data };
    }
}

class SearchAlgorithmBuilder {
    private data: Components.Schemas.SearchQuery;

    constructor(data: Components.Schemas.SearchQuery) {
        this.data = data;
    }

    from(id: BuilderQueryId): SearchFromBuilder {
        this.data.origin = intoQueryId(id);
        return new SearchFromBuilder(this.data);
    }

    to(id: BuilderQueryId): SearchToBuilder {
        this.data.destination = intoQueryId(id);
        return new SearchToBuilder(this.data);
    }
}

class SearchIndexValueBuilder {
    private data: Components.Schemas.SearchQuery;

    constructor(data: Components.Schemas.SearchQuery) {
        this.data = data;
    }

    query(): Components.Schemas.QueryType {
        return { Search: this.data };
    }
}

class SearchIndexBuilder {
    private key: Components.Schemas.DbValue;

    constructor(key: Components.Schemas.DbValue) {
        this.key = key;
    }

    value(value: BuilderDbValue): SearchIndexValueBuilder {
        let data = SearchBuilder.new_data();
        data.algorithm = "Index";
        data.conditions.push({
            data: {
                KeyValue: {
                    key: this.key,
                    value: { Equal: convertToDbValue(value) },
                },
            },
            logic: "And",
            modifier: "None",
        });
        return new SearchIndexValueBuilder(data);
    }
}

class SearchBuilder {
    static new_data(): Components.Schemas.SearchQuery {
        return {
            algorithm: "BreadthFirst",
            conditions: [],
            origin: { Id: 0 },
            destination: { Id: 0 },
            limit: 0,
            offset: 0,
            order_by: [],
        };
    }

    breadth_first(): SearchAlgorithmBuilder {
        let data = SearchBuilder.new_data();
        data.algorithm = "BreadthFirst";
        return new SearchAlgorithmBuilder(data);
    }

    depth_first(): SearchAlgorithmBuilder {
        let data = SearchBuilder.new_data();
        data.algorithm = "DepthFirst";
        return new SearchAlgorithmBuilder(data);
    }

    elements(): SearchToBuilder {
        let data = SearchBuilder.new_data();
        data.algorithm = "Elements";
        return new SearchToBuilder(data);
    }

    from(id: BuilderQueryId): SearchFromBuilder {
        let data = SearchBuilder.new_data();
        data.origin = intoQueryId(id);
        return new SearchFromBuilder(data);
    }

    index(key: BuilderDbValue): SearchIndexBuilder {
        return new SearchIndexBuilder(convertToDbValue(key));
    }

    to(id: BuilderQueryId): SearchToBuilder {
        let data = SearchBuilder.new_data();
        data.destination = intoQueryId(id);
        return new SearchToBuilder(data);
    }
}

export class QueryBuilder {
    static insert(): InsertBuilder {
        return new InsertBuilder();
    }

    static remove(): RemoveBuilder {
        return new RemoveBuilder();
    }

    static search(): SearchBuilder {
        return new SearchBuilder();
    }

    static select(): SelectBuilder {
        return new SelectBuilder();
    }
}
