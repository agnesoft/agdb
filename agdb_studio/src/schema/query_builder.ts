import type { components } from "./schema";

type QueryId = number | string;

function intoQueryIds(ids: QueryId[]): components["schemas"]["QueryId"][] {
    return ids.map((id) => {
        if (typeof id === "number") {
            return { Id: id };
        } else {
            return { Alias: id };
        }
    });
}

class InsertNodesAliasesBuilder {
    private data: components["schemas"]["InsertNodesQuery"];

    constructor(data: components["schemas"]["InsertNodesQuery"]) {
        this.data = data;
    }

    values_uniform(
        values: components["schemas"]["QueryValues"]["Single"],
    ): InsertNodesValuesBuilder {
        this.data.values.Single = values;
        return new InsertNodesValuesBuilder(this.data);
    }

    values(values: components["schemas"]["QueryValues"]["Multi"]): InsertNodesValuesBuilder {
        this.data.values.Multi = values;
        return new InsertNodesValuesBuilder(this.data);
    }

    query(): components["schemas"]["InsertNodesQuery"] {
        return this.data;
    }
}

class InsertNodesBuilder {
    private data: components["schemas"]["InsertNodesQuery"];

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
        values: components["schemas"]["QueryValues"]["Single"],
    ): InsertNodesValuesBuilder {
        this.data.values.Single = values;
        return new InsertNodesValuesBuilder(this.data);
    }

    values(values: components["schemas"]["QueryValues"]["Multi"]): InsertNodesValuesBuilder {
        this.data.values.Multi = values;
        return new InsertNodesValuesBuilder(this.data);
    }
}

class InsertNodesCountBuilder {
    private data: components["schemas"]["InsertNodesQuery"];

    constructor(data: components["schemas"]["InsertNodesQuery"]) {
        this.data = data;
    }

    values_uniform(
        values: components["schemas"]["QueryValues"]["Single"],
    ): InsertNodesValuesBuilder {
        this.data.values.Single = values;
        return new InsertNodesValuesBuilder(this.data);
    }

    query(): components["schemas"]["InsertNodesQuery"] {
        return this.data;
    }
}

class InsertNodesValuesBuilder {
    private data: components["schemas"]["InsertNodesQuery"];

    constructor(data: components["schemas"]["InsertNodesQuery"]) {
        this.data = data;
    }

    query(): components["schemas"]["InsertNodesQuery"] {
        return this.data;
    }
}

class InsertEdgesValuesBuilder {
    private data: components["schemas"]["InsertEdgesQuery"];

    constructor(query: components["schemas"]["InsertEdgesQuery"]) {
        this.data = query;
    }

    query(): components["schemas"]["InsertEdgesQuery"] {
        return this.data;
    }
}

class InsertEdgesToEachBuilder {
    private data: components["schemas"]["InsertEdgesQuery"];

    constructor(query: components["schemas"]["InsertEdgesQuery"]) {
        this.data = query;
    }

    values(values: components["schemas"]["QueryValues"]["Multi"]): InsertEdgesValuesBuilder {
        this.data.values.Multi = values;
        return new InsertEdgesValuesBuilder(this.data);
    }

    values_uniform(
        values: components["schemas"]["QueryValues"]["Single"],
    ): InsertEdgesValuesBuilder {
        this.data.values.Single = values;
        return new InsertEdgesValuesBuilder(this.data);
    }

    query(): components["schemas"]["InsertEdgesQuery"] {
        return this.data;
    }
}

class InsertEdgesToBuilder {
    private data: components["schemas"]["InsertEdgesQuery"];

    constructor(query: components["schemas"]["InsertEdgesQuery"]) {
        this.data = query;
    }

    each(): InsertEdgesToEachBuilder {
        this.data.each = true;
        return new InsertEdgesToEachBuilder(this.data);
    }

    values(values: components["schemas"]["QueryValues"]["Multi"]): InsertEdgesValuesBuilder {
        this.data.values.Multi = values;
        return new InsertEdgesValuesBuilder(this.data);
    }

    values_uniform(
        values: components["schemas"]["QueryValues"]["Single"],
    ): InsertEdgesValuesBuilder {
        this.data.values.Single = values;
        return new InsertEdgesValuesBuilder(this.data);
    }

    query(): components["schemas"]["InsertEdgesQuery"] {
        return this.data;
    }
}

class InsertEdgesFromBuilder {
    private data: components["schemas"]["InsertEdgesQuery"];

    constructor(query: components["schemas"]["InsertEdgesQuery"]) {
        this.data = query;
    }

    to(ids: QueryId[] | components["schemas"]["SearchQuery"]): InsertEdgesToBuilder {
        if (Array.isArray(ids)) {
            this.data.from.Ids = intoQueryIds(ids);
        } else {
            this.data.from.Search = ids;
        }

        return new InsertEdgesToBuilder(this.data);
    }
}

class InsertEdgesBuilder {
    private data: components["schemas"]["InsertEdgesQuery"];

    constructor() {
        this.data = {
            each: false,
            from: { Ids: [] },
            to: { Ids: [] },
            values: { Single: [] },
        };
    }

    from(ids: QueryId[] | components["schemas"]["SearchQuery"]): InsertEdgesFromBuilder {
        if (Array.isArray(ids)) {
            this.data.from.Ids = intoQueryIds(ids);
        } else {
            this.data.from.Search = ids;
        }

        return new InsertEdgesFromBuilder(this.data);
    }
}

class InsertAliasesIdsBuilder {
    private data: components["schemas"]["InsertAliasesQuery"];

    constructor(data: components["schemas"]["InsertAliasesQuery"]) {
        this.data = data;
    }

    query(): components["schemas"]["InsertAliasesQuery"] {
        return this.data;
    }
}

class InsertAliasesBuilder {
    private data: components["schemas"]["InsertAliasesQuery"];

    constructor(aliases: string[]) {
        this.data = { aliases: aliases, ids: { Ids: [] } };
    }

    ids(ids: QueryId[] | components["schemas"]["SearchQuery"]): InsertAliasesIdsBuilder {
        if (Array.isArray(ids)) {
            this.data.ids.Ids = intoQueryIds(ids);
        } else {
            this.data.ids.Search = ids;
        }
        return new InsertAliasesIdsBuilder(this.data);
    }
}

class InsertValuesIdsBuilder {
    private data: components["schemas"]["InsertValuesQuery"];

    constructor(data: components["schemas"]["InsertValuesQuery"]) {
        this.data = data;
    }

    query(): components["schemas"]["InsertValuesQuery"] {
        return this.data;
    }
}

class InsertValuesBuilder {
    private data: components["schemas"]["InsertValuesQuery"];

    constructor(data: components["schemas"]["InsertValuesQuery"]) {
        this.data = data;
    }

    ids(ids: QueryId[] | components["schemas"]["SearchQuery"]): InsertValuesIdsBuilder {
        if (Array.isArray(ids)) {
            this.data.ids.Ids = intoQueryIds(ids);
        } else {
            this.data.ids.Search = ids;
        }
        return new InsertValuesIdsBuilder(this.data);
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
        let data: components["schemas"]["InsertValuesQuery"] = {
            ids: { Ids: [] },
            values: { Single: [] },
        };
        data.ids.Ids = [];
        data.values = {
            Multi: [],
        };

        for (const elem of elems) {
            for (const key in Object.keys(elem)) {
                if (key === "db_id") {
                    let id = elem[key];
                    if (typeof id === "number") {
                        data.ids.Ids.push({ Id: id });
                    } else {
                        data.ids.Ids.push({ Alias: id });
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

    nodes(): InsertNodesBuilder {
        return new InsertNodesBuilder();
    }

    values(values: components["schemas"]["QueryValues"]["Multi"]): InsertValuesBuilder {
        return new InsertValuesBuilder({
            ids: { Ids: [] },
            values: { Multi: values as components["schemas"]["DbKeyValue"][][] },
        });
    }

    values_uniform(vals: components["schemas"]["QueryValues"]["Single"]): InsertValuesBuilder {
        return new InsertValuesBuilder({
            ids: { Ids: [] },
            values: { Single: vals as components["schemas"]["DbKeyValue"][] },
        });
    }
}

class RemoveIdsBuilder {
    private data: components["schemas"]["RemoveQuery"];

    constructor(data: components["schemas"]["RemoveQuery"]) {
        this.data = data;
    }

    query(): components["schemas"]["RemoveQuery"] {
        return this.data;
    }
}

class RemoveAliasesBuilder {
    private data: components["schemas"]["RemoveAliasesQuery"];

    constructor(data: components["schemas"]["RemoveAliasesQuery"]) {
        this.data = data;
    }

    query(): components["schemas"]["RemoveAliasesQuery"] {
        return this.data;
    }
}

class RemoveValuesIdsBuilder {
    private data: components["schemas"]["RemoveValuesQuery"];

    constructor(data: components["schemas"]["RemoveValuesQuery"]) {
        this.data = data;
    }

    query(): components["schemas"]["RemoveValuesQuery"] {
        return this.data;
    }
}

class RemoveValuesBuilder {
    private data: components["schemas"]["RemoveValuesQuery"];

    constructor(data: components["schemas"]["RemoveValuesQuery"]) {
        this.data = data;
    }

    ids(ids: QueryId[] | components["schemas"]["SearchQuery"]): RemoveValuesIdsBuilder {
        if (Array.isArray(ids)) {
            this.data.ids.Ids = intoQueryIds(ids);
        } else {
            this.data.ids.Search = ids;
        }

        return new RemoveValuesIdsBuilder(this.data);
    }
}

class RemoveBuilder {
    aliases(aliases: string[]): RemoveAliasesBuilder {
        return new RemoveAliasesBuilder(aliases);
    }

    ids(ids: QueryId[] | components["schemas"]["SearchQuery"]): RemoveIdsBuilder {
        if (Array.isArray(ids)) {
            return new RemoveIdsBuilder({ Ids: intoQueryIds(ids) });
        } else {
            return new RemoveIdsBuilder({ Search: ids });
        }
    }

    values(values: components["schemas"]["DbValue"][]): RemoveValuesBuilder {
        return new RemoveValuesBuilder({ ids: { Ids: [] }, keys: values });
    }
}

class SelectAliasesIdsBuilder {
    private data: components["schemas"]["SelectAliasesQuery"];

    constructor(data: components["schemas"]["SelectAliasesQuery"]) {
        this.data = data;
    }

    query(): components["schemas"]["SelectAliasesQuery"] {
        return this.data;
    }
}

class SelectAliasesBuilder {
    private data: components["schemas"]["SelectAliasesQuery"];

    constructor() {
        this.data = {
            Ids: [],
        };
    }

    ids(ids: QueryId[] | components["schemas"]["SearchQuery"]): SelectAliasesIdsBuilder {
        if (Array.isArray(ids)) {
            return new SelectAliasesIdsBuilder({ Ids: intoQueryIds(ids) });
        } else {
            return new SelectAliasesIdsBuilder({ Search: ids });
        }
    }

    query(): components["schemas"]["SelectAllAliasesQuery"] {
        return {};
    }
}

class SelectIdsBuilder {
    private data: components["schemas"]["SelectQuery"];

    constructor(data: components["schemas"]["SelectQuery"]) {
        this.data = data;
    }

    query(): components["schemas"]["SelectQuery"] {
        return this.data;
    }
}

class SelectValuesIdsBuilder {
    private data: components["schemas"]["SelectValuesQuery"];

    constructor(data: components["schemas"]["SelectValuesQuery"]) {
        this.data = data;
    }

    query(): components["schemas"]["SelectValuesQuery"] {
        return this.data;
    }
}

class SelectValuesBuilder {
    private data: components["schemas"]["SelectValuesQuery"];

    constructor(data: components["schemas"]["SelectValuesQuery"]) {
        this.data = data;
    }

    ids(ids: QueryId[] | components["schemas"]["SearchQuery"]): SelectValuesIdsBuilder {
        if (Array.isArray(ids)) {
            this.data.ids.Ids = intoQueryIds(ids);
        } else {
            this.data.ids.Search = ids;
        }

        return new SelectValuesIdsBuilder(this.data);
    }
}

class SelectKeysIdsBuilder {
    private data: components["schemas"]["SelectKeysQuery"];

    constructor(data: components["schemas"]["SelectKeysQuery"]) {
        this.data = data;
    }

    query(): components["schemas"]["SelectKeysQuery"] {
        return this.data;
    }
}

class SelectKeysBuilder {
    private data: components["schemas"]["SelectKeysQuery"];

    constructor(data: components["schemas"]["SelectKeysQuery"]) {
        this.data = data;
    }

    ids(ids: QueryId[] | components["schemas"]["SearchQuery"]): SelectKeysIdsBuilder {
        if (Array.isArray(ids)) {
            this.data.Ids = intoQueryIds(ids);
        } else {
            this.data.Search = ids;
        }

        return new SelectKeysIdsBuilder(this.data);
    }
}

class SelectKeyCountIdsBuilder {
    private data: components["schemas"]["SelectKeyCountQuery"];

    constructor(data: components["schemas"]["SelectKeyCountQuery"]) {
        this.data = data;
    }

    query(): components["schemas"]["SelectKeyCountQuery"] {
        return this.data;
    }
}

class SelectKeyCountBuilder {
    private data: components["schemas"]["SelectKeyCountQuery"];

    constructor(data: components["schemas"]["SelectKeyCountQuery"]) {
        this.data = data;
    }

    ids(ids: QueryId[] | components["schemas"]["SearchQuery"]): SelectKeyCountIdsBuilder {
        if (Array.isArray(ids)) {
            this.data.Ids = intoQueryIds(ids);
        } else {
            this.data.Search = ids;
        }

        return new SelectKeyCountIdsBuilder(this.data);
    }
}

class SelectBuilder {
    aliases(): SelectAliasesBuilder {
        return new SelectAliasesBuilder();
    }

    ids(ids: QueryId[] | components["schemas"]["SearchQuery"]): SelectIdsBuilder {
        if (Array.isArray(ids)) {
            return new SelectIdsBuilder({ Ids: intoQueryIds(ids) });
        } else {
            return new SelectIdsBuilder({ Search: ids });
        }
    }

    keys(): SelectKeysBuilder {
        return new SelectKeysBuilder({ Ids: [] });
    }

    key_count(): SelectKeyCountBuilder {
        return new SelectKeyCountBuilder({ Ids: [] });
    }

    values(values: components["schemas"]["DbValue"][]): SelectValuesBuilder {
        return new SelectValuesBuilder({ ids: { Ids: [] }, keys: values });
    }
}

export class QueryBuilder {
    constructor() {
        return new QueryBuilder();
    }

    static insert(): InsertBuilder {
        return new InsertBuilder();
    }

    static remove(): RemoveBuilder {
        return new RemoveBuilder();
    }

    static select(): SelectBuilder {
        return new SelectBuilder();
    }
}
