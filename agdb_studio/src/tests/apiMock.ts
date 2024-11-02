import { vi } from "vitest";

export const get_token = vi.fn();
export const user_status = vi.fn();

export const client = vi.fn().mockResolvedValue({
    login: vi.fn().mockResolvedValue("token"),
    logout: vi.fn().mockResolvedValue(undefined),
    set_token: vi.fn(),
    get_token,
    reset_token: vi.fn(),
    interceptors: {
        request: {
            use: vi.fn(),
        },
        response: {
            use: vi.fn(),
        },
    },
    user_status,
});
vi.mock("agdb_api", () => {
    return {
        AgdbApi: {
            client,
        },
    };
});
