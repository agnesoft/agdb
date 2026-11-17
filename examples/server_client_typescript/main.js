var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype);
    return g.next = verb(0), g["throw"] = verb(1), g["return"] = verb(2), typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
import { QueryBuilder, Comparison, AgdbApi } from "agdb_api";
function main() {
    return __awaiter(this, void 0, void 0, function () {
        var client, queries, results, _i, _a, _b, key, value;
        return __generator(this, function (_c) {
            switch (_c.label) {
                case 0: return [4 /*yield*/, AgdbApi.client("http://localhost:3000")];
                case 1:
                    client = _c.sent();
                    // Creates a user using default admin credentials.
                    client.login({ username: "admin", password: "admin" });
                    return [4 /*yield*/, client.admin_user_add("user1", { password: "password123" })];
                case 2:
                    _c.sent();
                    // Creates a database using the newly created user.
                    client.login({ username: "user1", password: "password123" });
                    return [4 /*yield*/, client.db_add({
                            owner: "user1",
                            db: "db1",
                            db_type: "memory",
                        })];
                case 3:
                    _c.sent();
                    queries = [
                        QueryBuilder.insert().nodes().aliases("users").query(),
                        QueryBuilder.insert()
                            .nodes()
                            .values([
                            [
                                ["username", "user1"],
                                ["password", "password123"],
                            ],
                            [
                                ["username", "user1"],
                                ["password", "password456"],
                            ],
                        ])
                            .query(),
                    ];
                    return [4 /*yield*/, client.db_exec_mut({ owner: "user1", db: "db1" }, queries)];
                case 4:
                    results = (_c.sent()).data;
                    // Prepare the second batch using the result of the previous batch.
                    queries = [
                        QueryBuilder.insert().edges().from("users").to(results[1]).query(),
                        QueryBuilder.select()
                            .search()
                            .from("users")
                            .where()
                            .key("username")
                            .value(Comparison.Equal("user1"))
                            .query(),
                    ];
                    return [4 /*yield*/, client.db_exec_mut({ owner: "user1", db: "db1" }, queries)];
                case 5:
                    // Execute the second batch of queries.
                    results = (_c.sent())
                        .data;
                    // Print the result of the second query.
                    console.log("User (id: ".concat(results[1].elements[0].id, ")"));
                    for (_i = 0, _a = results[1].elements[0].values; _i < _a.length; _i++) {
                        _b = _a[_i], key = _b.key, value = _b.value;
                        console.log("".concat(key["String"], ": ").concat(value["String"]));
                    }
                    return [2 /*return*/];
            }
        });
    });
}
main();
//# sourceMappingURL=main.js.map