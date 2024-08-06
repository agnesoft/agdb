import { describe, expect, it, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { useI18n } from "@/hooks/i18n";

vi.mock("next/router", () => ({
    useRouter: () => ({
        locale: "cs-CZ",
    }),
}));

vi.mock("nextra/constants", () => ({
    DEFAULT_LOCALE: "en-US",
}));

vi.mock("@/messages/cs-CZ.json", () => ({
    test: "testCZ",
}));

vi.mock("@/messages/en-US.json", () => ({
    test: "testEN",
    test2: {
        test3: "testEN2",
    },
}));

describe("i18n", () => {
    it("should return the default locale", () => {
        const { result } = renderHook(() => useI18n());
        expect(result.current.locale).toBe("cs-CZ");
    });

    it("should return a translation", async () => {
        const { result } = renderHook(() => useI18n());

        await waitFor(() => {
            expect(result.current.t("test")).toBe("testCZ");
        });
    });

    it("should return a fallback translation", async () => {
        const { result } = renderHook(() => useI18n());
        await waitFor(() => {
            expect(result.current.t("test2.test3")).toBe("testEN2");
        });
    });
});
