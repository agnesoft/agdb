import { describe, it, expect, beforeEach, vi } from "vitest";
import { dbActions, dbColumns } from "./dbConfig";
import {
    db_backup,
    db_restore,
    db_clear,
    db_convert,
    db_remove,
    db_delete,
    db_optimize,
    db_audit,
    db_copy,
    db_rename,
} from "@/tests/apiMock";
import { useContentInputs } from "../content/inputs";
import { KEY_MODAL } from "../modal/constants";
import { ref } from "vue";

const { addInput, setInputValue, clearAllInputs } = useContentInputs();

describe("dbConfig", () => {
    describe("dbColumns", () => {
        it("should have correct db columns", () => {
            const columnKeys = dbColumns.map((column) => column.key);
            expect(columnKeys).toContain("owner");
            expect(columnKeys).toContain("db");
            expect(columnKeys).toContain("db_type");
            expect(columnKeys).toContain("size");
            expect(columnKeys).toContain("backup");
        });
    });
    describe("dbActions", () => {
        beforeEach(() => {
            clearAllInputs();
            vi.clearAllMocks();
        });
        it("should have correct db actions", () => {
            const actionKeys = dbActions.map((action) => action.key);
            expect(actionKeys).toContain("backup");
            expect(actionKeys).toContain("optimize");
            expect(actionKeys).toContain("audit");
            expect(actionKeys).toContain("remove");
            expect(actionKeys).toContain("delete");
            expect(actionKeys).toContain("restore");
        });
        it.each([
            ["backup", db_backup],
            ["restore", db_restore],
            ["remove", db_remove],
            ["delete", db_delete],
            ["optimize", db_optimize],
            ["audit", db_audit],
        ])("should run correct db actions for %s", (key, api) => {
            const action = dbActions.find((action) => action.key === key);
            const params = { db: "test_db" };
            action?.action({ params });
            expect(api).toHaveBeenCalledWith(params);
        });

        it.each([
            ["clear", "all", "resource", db_clear],
            ["clear", "db", "resource", db_clear],
            ["clear", "audit", "resource", db_clear],
            ["clear", "backup", "resource", db_clear],
            ["convert", "file", "db_type", db_convert],
            ["convert", "memory", "db_type", db_convert],
            ["convert", "mapped", "db_type", db_convert],
        ])(
            "should run correct db actions for %s with %s",
            (key, value, valueKey, api) => {
                const action = dbActions.find((action) => action.key === key);
                const subaction = action?.actions?.find(
                    (action) => action.key === value,
                );
                const params = { db: "test_db" };
                subaction?.action({ params });
                expect(api).toHaveBeenCalledWith({
                    ...params,
                    [valueKey]: value,
                });
            },
        );

        it.each([
            ["copy", db_copy],
            ["rename", db_rename],
        ])("should run correct db actions for %s", (key, api) => {
            const newName = "new_test_db";
            addInput(KEY_MODAL, "new_db", ref());
            setInputValue(KEY_MODAL, "new_db", newName);
            const action = dbActions.find((action) => action.key === key);
            const params = { db: "test_db" };
            action?.action({ params });
            expect(api).toHaveBeenCalledWith({
                ...params,
                new_db: newName,
            });
            clearAllInputs();
        });
        it.each([
            ["copy", db_copy],
            ["rename", db_rename],
        ])("should not run correct db actions for %s", (key, api) => {
            addInput(KEY_MODAL, "new_db", ref());
            const action = dbActions.find((action) => action.key === key);
            const params = { db: "test_db" };
            action?.action({ params }).catch(() => {});
            expect(api).not.toHaveBeenCalled();
            clearAllInputs();
        });
    });
});
