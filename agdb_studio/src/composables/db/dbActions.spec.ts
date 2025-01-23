import { describe, beforeEach, vi, it, expect } from "vitest";

const { username, admin } = vi.hoisted(() => {
    return {
        username: { value: "test_user" },
        admin: { value: false },
    };
});
vi.mock("../user/account", () => {
    return {
        useAccount: () => {
            return {
                username,
                admin,
            };
        },
    };
});
