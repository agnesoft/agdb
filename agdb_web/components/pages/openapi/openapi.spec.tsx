import { expect, describe, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";

import { OpenApi } from "./openapi";
import { beforeEach } from "node:test";

const jsonMock = {
    openapi: "3.0.0",
    info: {
        title: "AGDB API",
        version: "1.0.0",
    },
    paths: {},
};

vi.mock("nextra/hooks", () => ({
    useRouter: () => ({
        pathname: "/",
        locale: "en-US",
    }),
}));
describe("openapi", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("should render the openapi code on click", async () => {
        const file = await import("../../../../agdb_server/openapi.json");

        //@ts-expect-error mocking the openapi
        file.default = jsonMock;

        render(<OpenApi />);
        expect(screen.getByText("openapi.json")).toBeDefined();

        const showButton = screen.getByText("Show code");
        fireEvent.click(showButton);

        expect(screen.getByText("Hide code")).toBeDefined();
    });
});
