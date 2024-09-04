import test_queries from "../../agdb_server/test_queries.json" assert { type: "json" };
import * as fs from "fs";

let tests = `
// GENERATED. DO NOT MODIFY AS ANY CHANGES WILL BE LOST.\n
//query_test_generator.js
\n\n
import { describe, expect, it } from "vitest";import test_queries from "../../../agdb_server/openapi/test_queries.json";
import { QueryBuilder, CountComparison, Comparison, DbKeyOrder } from "../src/index";
\n\n
class T { db_id: undefined = undefined; value1: string = ""; value2: number = 0; }
\n\n
describe("query tests", () => {`;

for (let index in test_queries) {
    let name = test_queries[index][0];
    let builder = test_queries[index][0];
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
    
    tests += `it(\`${name}\`, () => { let query = ${builder};\nexpect(query).toEqual(test_queries[${index}][1]); });\n\n`;
}

tests += `});`;

fs.writeFileSync("tests/query.test.ts", tests);
