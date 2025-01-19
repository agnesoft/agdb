import { vi, describe, it, beforeEach, expect } from "vitest";
import { useUserActions } from "./userActions";

describe("userActions.ts", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });
    it("returns the user actions", () => {
        const { actions } = useUserActions();
        expect(actions.value.length).toBe(2);
    });
});
