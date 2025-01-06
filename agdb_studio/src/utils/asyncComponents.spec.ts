import { describe, it, expect } from "vitest";
import { getAsyncComponent } from "./asyncComponents";

describe("asyncComponents", () => {
    it("should return component", () => {
        const component = getAsyncComponent("DbDetails");
        expect(component).toBeDefined();
    });
});
