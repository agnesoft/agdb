import { expect, describe, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";

import { OpenApi } from "./openapi";
import { beforeEach } from "node:test";

const mocks = vi.hoisted(() => {
    return {
        openapiFile: {
            openapi: "3.0.0",
            info: {
                title: "AGDB API",
                version: "1.0.0",
            },
            paths: {},
        },
    };
});
vi.mock("../../../../agdb_server/openapi.json", () => ({
    default: mocks.openapiFile,
}));
vi.mock("next/router", () => ({
    useRouter: () => ({
        pathname: "/",
        locale: "en-US",
    }),
}));
describe("openapi", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("should render the openapi", () => {
        render(<OpenApi />);
        expect(screen.getByText("openapi.json")).toBeDefined();
        expect(screen.getByText('"AGDB API"')).toBeDefined();
    });
});
