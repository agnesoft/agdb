import { components } from "./schema";

class InsertNodesAliasesBuilder {
    private data: components["schemas"]["InsertNodesQuery"];

    constructor(data: components["schemas"]["InsertNodesQuery"]) {
        return new InsertNodesAliasesBuilder(data);
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
        return new InsertNodesBuilder();
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
        return new InsertNodesCountBuilder(data);
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
        return new InsertNodesValuesBuilder(data);
    }

    query(): components["schemas"]["InsertNodesQuery"] {
        return this.data;
    }
}

class InsertEdgesBuilder {
    constructor() {
        return new InsertEdgesBuilder();
    }

    private query: components["schemas"]["InsertEdgesQuery"];
}

class InsertBuilder {
    constructor() {
        return new InsertBuilder();
    }

    edges(): InsertEdgesBuilder {
        return new InsertEdgesBuilder();
    }

    nodes(): InsertNodesBuilder {
        return new InsertNodesBuilder();
    }
}

export class QueryBuilder {
    constructor() {
        return new QueryBuilder();
    }

    static insert(): InsertBuilder {
        return new InsertBuilder();
    }
}
