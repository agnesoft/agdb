import test_queries from "../../agdb_server/openapi/test_queries.json" assert { type: "json" };
import * as fs from "fs";

let tests = `import { describe, expect, it } from "vitest";import test_queries from "../../../agdb_server/openapi/test_queries.json";import { QueryBuilder, CountComparison, Comparison, DbKeyOrder } from "../src/index";\n\nclass T { value1: string = ""; value2: number = 0; }\n\ndescribe("openapi test", () => {`;

for (let query in test_queries) {
    let builder = query;
    builder = builder.replace(/&/g, "");
    builder = builder.replace(/T::default\(\)/g, "new T()");
    builder = builder.replace(/::/g, ".");
    builder = builder.replace(/vec!/g, "");
    builder = builder.replace(/where_/g, "where");
    builder = builder.replace(/\.into\(\)/g, "");
    builder = builder.replace(/\("k","v"\)/g, "[\"k\", \"v\"]");
    builder = builder.replace(/\("k",1\)/g, "[\"k\", 1]");
    builder = builder.replace(/\("k",2\)/g, "[\"k\", 2]");
    builder = builder.replace(/\(1,10\)/g, "[1, 10]");
    
    tests += `it(\`${query}\`, () => { let query = \`${query}\`;\nlet builder = ${builder};\nexpect(builder).toEqual(test_queries[query]); });\n\n`;
}

tests += `});`;

fs.writeFileSync("tests/query.test.ts", tests);
