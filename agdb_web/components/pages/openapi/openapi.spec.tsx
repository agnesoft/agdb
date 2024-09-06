import { expect, describe, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";

import { OpenApi } from "./openapi";

vi.mock("../../../../agdb_server/openapi.json", () => ({
    openapi: "3.0.0",
    info: {
        title: "AGDB API",
        version: "1.0.0",
    },
    paths: {},
}));

describe("openapi", () => {
    it("should render the openapi", () => {
        render(<OpenApi />);
        expect(screen.getByText("openapi.json")).toBeDefined();
        expect(screen.getByText("AGDB API")).toBeDefined();
    });
});
