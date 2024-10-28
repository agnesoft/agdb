import { vi } from "vitest";
import { AgdbApi } from "agdb_api";

const client = vi.fn().mockResolvedValue({
    login: vi.fn().mockResolvedValue("token"),
    logout: vi.fn().mockResolvedValue(undefined),
    set_token: vi.fn(),
    get_token: vi.fn(),
    reset_token: vi.fn(),
    interceptors: {
        request: {
            use: vi.fn(),
        },
        response: {
            use: vi.fn(),
        },
    },
}) as unknown as AgdbApi.AgdbApiClient;
vi.mock("agdb_api", () => {
    return {
        AgdbApi: {
            client,
        },
    };
});
