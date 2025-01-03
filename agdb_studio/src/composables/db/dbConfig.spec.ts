import { describe, it, expect, beforeEach, vi } from "vitest";
import {
    dbActions,
    dbColumns,
    getConfirmationHeaderFn,
    type DbActionProps,
} from "./dbConfig";
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
import useModal from "../modal/modal";

const { addInput, setInputValue, clearAllInputs } = useContentInputs();

const { modalIsVisible, modal } = useModal();

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

        it("should print the empty audit log", async () => {
            const action = dbActions.find((action) => action.key === "audit");
            const params = { db: "test_db", owner: "test_owner" };
            await action?.action({ params });
            expect(db_audit).toHaveBeenCalledWith(params);

            expect(modalIsVisible.value).toBe(true);
            expect(modal.header).toBe("Audit log of test_owner/test_db");
            expect(modal.content).toHaveLength(1);
        });
        it("should print the audit log", async () => {
            const action = dbActions.find((action) => action.key === "audit");
            const params = { db: "test_db", owner: "test_owner" };
            db_audit.mockResolvedValueOnce({
                data: [
                    {
                        timestamp: "123",
                        user: "test_user",
                        query: "test_query",
                    },
                    {
                        timestamp: "456",
                        user: "test_user2",
                        query: "test_query2",
                    },
                ],
            });
            await action?.action({ params });
            expect(db_audit).toHaveBeenCalledWith(params);

            expect(modalIsVisible.value).toBe(true);
            expect(modal.header).toBe("Audit log of test_owner/test_db");
            expect(modal.content).toHaveLength(2);
            expect(modal.content[0].paragraph?.at(0)?.text).toBe(
                "123 | test_user | test_query",
            );
            expect(modal.content[1].paragraph?.at(0)?.text).toBe(
                "456 | test_user2 | test_query2",
            );
        });
    });
    describe("getConfirmationHeaderFn", () => {
        it("should return correct header", () => {
            const header = getConfirmationHeaderFn({
                params: { db: "test_db", owner: "test_owner" },
            } as unknown as DbActionProps);
            expect(header).toBe("Confirm action for test_owner/test_db");
        });
    });
});
