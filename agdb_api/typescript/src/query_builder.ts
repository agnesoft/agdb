import type { Components } from "./schema";

type QueryId = number | string;

function intoQueryIds(
    ids:
        | QueryId[]
        | Components.Schemas.QueryType
        | Components.Schemas.QueryResult,
): Components.Schemas.QueryIds {
    if ("Search" in ids) {
        return ids;
    } else if (Array.isArray(ids)) {
        return { Ids: ids.map((id) => intoQueryId(id)) };
    } else if ("result" in ids) {
        return {
            Ids: ids.elements.map((elem) => {
                return { Id: elem.id };
            }),
        };
    } else {
        throw new Error("invalid search query");
    }
}

function intoQueryId(id: QueryId): Components.Schemas.QueryId {
    if (typeof id === "number") {
        return { Id: id };
    } else {
        return { Alias: id };
    }
}

class InsertNodesAliasesBuilder {
    private data: Components.Schemas.InsertNodesQuery;

    constructor(data: Components.Schemas.InsertNodesQuery) {
        this.data = data;
    }

    values_uniform(
        values: Components.Schemas.DbKeyValue[],
    ): InsertNodesValuesBuilder {
        this.data.values = { Single: values };
        return new InsertNodesValuesBuilder(this.data);
    }

    values(
        values: Components.Schemas.DbKeyValue[][],
    ): InsertNodesValuesBuilder {
        this.data.values = { Multi: values };
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
            values: {
                Single: [],
            },
        };
    }

    aliases(aliases: string[]): InsertNodesAliasesBuilder {
        this.data.aliases = aliases;
        return new InsertNodesAliasesBuilder(this.data);
    }

    count(count: number): InsertNodesCountBuilder {
        this.data.count = count;
        return new InsertNodesCountBuilder(this.data);
    }

    values_uniform(
        values: Components.Schemas.DbKeyValue[],
    ): InsertNodesValuesBuilder {
        this.data.values = { Single: values };
        return new InsertNodesValuesBuilder(this.data);
    }

    values(
        values: Components.Schemas.DbKeyValue[][],
    ): InsertNodesValuesBuilder {
        this.data.values = { Multi: values };
        return new InsertNodesValuesBuilder(this.data);
    }
}

class InsertNodesCountBuilder {
    private data: Components.Schemas.InsertNodesQuery;

    constructor(data: Components.Schemas.InsertNodesQuery) {
        this.data = data;
    }

    values_uniform(
        values: Components.Schemas.DbKeyValue[],
    ): InsertNodesValuesBuilder {
        this.data.values = { Single: values };
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

    values(
        values: Components.Schemas.DbKeyValue[][],
    ): InsertEdgesValuesBuilder {
        this.data.values = { Multi: values };
        return new InsertEdgesValuesBuilder(this.data);
    }

    values_uniform(
        values: Components.Schemas.DbKeyValue[],
    ): InsertEdgesValuesBuilder {
        this.data.values = { Single: values };
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

    values(
        values: Components.Schemas.DbKeyValue[][],
    ): InsertEdgesValuesBuilder {
        this.data.values = { Multi: values };
        return new InsertEdgesValuesBuilder(this.data);
    }

    values_uniform(
        values: Components.Schemas.DbKeyValue[],
    ): InsertEdgesValuesBuilder {
        this.data.values = { Single: values };
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

    to(
        ids:
            | QueryId[]
            | Components.Schemas.QueryType
            | Components.Schemas.QueryResult,
    ): InsertEdgesToBuilder {
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
            values: { Single: [] },
        };
    }

    from(
        ids:
            | QueryId[]
            | Components.Schemas.QueryType
            | Components.Schemas.QueryResult,
    ): InsertEdgesFromBuilder {
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

    ids(
        ids:
            | QueryId[]
            | Components.Schemas.QueryType
            | Components.Schemas.QueryResult,
    ): InsertAliasesIdsBuilder {
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

    ids(
        ids:
            | QueryId[]
            | Components.Schemas.QueryType
            | Components.Schemas.QueryResult,
    ): InsertValuesIdsBuilder {
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
    aliases(names: string[]): InsertAliasesBuilder {
        return new InsertAliasesBuilder(names);
    }

    element(elem: any): InsertValuesIdsBuilder {
        return this.elements([elem]);
    }

    elements(elems: any[]): InsertValuesIdsBuilder {
        let data: Components.Schemas.InsertValuesQuery = {
            ids: { Ids: [] },
            values: { Single: [] },
        };
        data.ids = { Ids: [] };
        data.values = {
            Multi: [],
        };

        for (const elem of elems) {
            for (const key of Object.keys(elem)) {
                if (key === "db_id") {
                    let id = elem[key];
                    if (typeof id === "number") {
                        data.ids.Ids.push({ Id: id });
                    } else {
                        throw new Error("invalid db_id");
                    }
                } else {
                    data.values.Multi.push([
                        {
                            key: { String: key },
                            value: elem[key],
                        },
                    ]);
                }
            }
        }

        return new InsertValuesIdsBuilder(data);
    }

    edges(): InsertEdgesBuilder {
        return new InsertEdgesBuilder();
    }

    index(key: Components.Schemas.DbValue): InsertIndexBuilder {
        return new InsertIndexBuilder(key);
    }

    nodes(): InsertNodesBuilder {
        return new InsertNodesBuilder();
    }

    values(values: Components.Schemas.DbKeyValue[][]): InsertValuesBuilder {
        return new InsertValuesBuilder({
            ids: { Ids: [] },
            values: { Multi: values as Components.Schemas.DbKeyValue[][] },
        });
    }

    values_uniform(vals: Components.Schemas.DbKeyValue[]): InsertValuesBuilder {
        return new InsertValuesBuilder({
            ids: { Ids: [] },
            values: { Single: vals as Components.Schemas.DbKeyValue[] },
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

    ids(
        ids:
            | QueryId[]
            | Components.Schemas.QueryType
            | Components.Schemas.QueryResult,
    ): RemoveValuesIdsBuilder {
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
    aliases(aliases: string[]): RemoveAliasesBuilder {
        return new RemoveAliasesBuilder(aliases);
    }

    ids(
        ids:
            | QueryId[]
            | Components.Schemas.QueryType
            | Components.Schemas.QueryResult,
    ): RemoveIdsBuilder {
        if (Array.isArray(ids)) {
            return new RemoveIdsBuilder(intoQueryIds(ids));
        } else {
            return new RemoveIdsBuilder(intoQueryIds(ids));
        }
    }

    index(key: Components.Schemas.DbValue): RemoveIndexBuilder {
        return new RemoveIndexBuilder(key);
    }

    values(values: Components.Schemas.DbValue[]): RemoveValuesBuilder {
        return new RemoveValuesBuilder({ ids: { Ids: [] }, keys: values });
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

    ids(
        ids:
            | QueryId[]
            | Components.Schemas.QueryType
            | Components.Schemas.QueryResult,
    ): SelectAliasesIdsBuilder {
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

    ids(
        ids:
            | QueryId[]
            | Components.Schemas.QueryType
            | Components.Schemas.QueryResult,
    ): SelectValuesIdsBuilder {
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

    ids(
        ids:
            | QueryId[]
            | Components.Schemas.QueryType
            | Components.Schemas.QueryResult,
    ): SelectKeysIdsBuilder {
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

    ids(
        ids:
            | QueryId[]
            | Components.Schemas.QueryType
            | Components.Schemas.QueryResult,
    ): SelectEdgeCountIdsBuilder {
        this.data.ids = intoQueryIds(ids);
        return new SelectEdgeCountIdsBuilder(this.data);
    }
}

class SelectKeyCountBuilder {
    private data: Components.Schemas.SelectKeyCountQuery;

    constructor(data: Components.Schemas.SelectKeyCountQuery) {
        this.data = data;
    }

    ids(
        ids:
            | QueryId[]
            | Components.Schemas.QueryType
            | Components.Schemas.QueryResult,
    ): SelectKeyCountIdsBuilder {
        this.data = intoQueryIds(ids);
        return new SelectKeyCountIdsBuilder(this.data);
    }
}

class SelectIndexesBuilder {
    query(): Components.Schemas.QueryType {
        return { SelectIndexes: {} };
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

    ids(
        ids:
            | QueryId[]
            | Components.Schemas.QueryType
            | Components.Schemas.QueryResult,
    ): SelectIdsBuilder {
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

    values(values: Components.Schemas.DbValue[]): SelectValuesBuilder {
        return new SelectValuesBuilder({ ids: { Ids: [] }, keys: values });
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
        while (collapse_conditions(this.data.conditions)) {
            /* intentionally empty */
        }
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

    ids(ids: QueryId[]): SearchWhereLogicBuilder {
        return push_condition(this, {
            data: { Ids: ids.map((id) => intoQueryId(id)) },
            logic: this.logic,
            modifier: this.modifier,
        });
    }

    key(key: Components.Schemas.DbValue): SearchWhereKeyBuilder {
        return new SearchWhereKeyBuilder(key, this);
    }

    keys(keys: Components.Schemas.DbValue[]): SearchWhereLogicBuilder {
        return push_condition(this, {
            data: { Keys: keys },
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

    order_by(keys: Components.Schemas.DbKeyOrder[]): SearchOrderBy {
        this.data.order_by = keys;
        return new SearchOrderBy(this.data);
    }

    to(id: QueryId): SearchToBuilder {
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

    order_by(keys: Components.Schemas.DbKeyOrder[]): SearchOrderBy {
        this.data.order_by = keys;
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

    from(id: QueryId): SearchFromBuilder {
        this.data.origin = intoQueryId(id);
        return new SearchFromBuilder(this.data);
    }

    to(id: QueryId): SearchToBuilder {
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

    value(value: Components.Schemas.DbValue): SearchIndexValueBuilder {
        let data = SearchBuilder.new_data();
        data.algorithm = "Index";
        data.conditions.push({
            data: { KeyValue: { key: this.key, value: { Equal: value } } },
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

    from(id: QueryId): SearchFromBuilder {
        let data = SearchBuilder.new_data();
        data.origin = intoQueryId(id);
        return new SearchFromBuilder(data);
    }

    index(key: Components.Schemas.DbValue): SearchIndexBuilder {
        return new SearchIndexBuilder(key);
    }

    to(id: QueryId): SearchToBuilder {
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
