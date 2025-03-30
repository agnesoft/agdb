import { shallowMount } from "@vue/test-utils";
import DbTable from "./DbTable.vue";
import { describe, it, expect, vi } from "vitest";

const { databases, getDbName } = vi.hoisted(() => {
    return {
        databases: {
            value: [
                {
                    owner: "test_owner",
                    db: "test_db",
                    db_type: "memory",
                    role: "admin",
                    size: 2568,
                    backup: 0,
                },
                {
                    owner: "test_owner2",
                    db: "test_db2",
                    db_type: "memory",
                    role: "admin",
                    size: 2568,
                    backup: 0,
                },
            ],
        },

        getDbName: vi.fn().mockImplementation((db) => {
            return `${db.owner}/${db.db}`;
        }),
    };
});

vi.mock("@/composables/db/dbStore", () => {
    return {
        useDbStore: () => {
            return {
                databases,
                getDbName,
            };
        },
    };
});

describe("DbTable", () => {
    it("should create table and render databases", () => {
        const wrapper = shallowMount(DbTable);
        expect(wrapper.exists()).toBe(true);
    });

    it("should render message when no databases", () => {
        databases.value = [];
        const wrapper = shallowMount(DbTable);
        expect(wrapper.text()).toContain("No databases found");
    });
});
