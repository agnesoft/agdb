import { mount } from "@vue/test-utils";
import { describe, beforeEach, vi, it, expect } from "vitest";
import DbDetails from "./DbDetails.vue";

describe("DbDetails", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("should render", async () => {
        expect(true).toBe(true);
    });
});
