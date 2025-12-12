import { describe, expect, it, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";

const { useRouterMock, messagesEN, messagesCZ } = vi.hoisted(() => {
    return {
        useRouterMock: vi.fn(),
        messagesEN: {
            test: "testEN",
            test2: {
                test3: "testEN2",
            },
        },
        messagesCZ: {
            test: "testCZ",
        },
    };
});
vi.mock("nextra/hooks", () => ({
    useRouter: useRouterMock,
}));

describe("i18n", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        vi.resetAllMocks();
    });
    it("should not return a fallback translation", async () => {
        // Simulate the default locale import rejecting
        vi.resetModules();
        vi.doMock("../messages/en-US.json", () => {
            throw new Error("error");
        });
        vi.doMock("../messages/cs-CZ.json", () => messagesCZ);

        useRouterMock.mockReturnValue({
            locale: "",
            defaultLocale: "en-US",
        });

        const { useI18n } = await import("@/hooks/i18n");
        const { result } = renderHook(() => useI18n());
        await waitFor(() => {
            expect(result.current.t("test2.test3")).not.toBe("testEN2");
        });
    });

    it("should return a fallback translation", async () => {
        vi.resetModules();
        vi.doMock("../messages/en-US.json", () => messagesEN);
        vi.doMock("../messages/cs-CZ.json", () => messagesCZ);

        useRouterMock.mockReturnValue({
            locale: "",
            defaultLocale: "en-US",
        });

        const { useI18n } = await import("@/hooks/i18n");
        const { result } = renderHook(() => useI18n());
        await waitFor(() => {
            expect(result.current.t("test2.test3")).toBe("testEN2");
        });
    });

    it("should return the default locale", async () => {
        vi.resetModules();
        vi.doMock("../messages/en-US.json", () => messagesEN);
        vi.doMock("../messages/cs-CZ.json", () => messagesCZ);

        useRouterMock.mockReturnValue({
            locale: "cs-CZ",
            defaultLocale: "en-US",
        });

        const { useI18n } = await import("@/hooks/i18n");
        const { result } = renderHook(() => useI18n());
        expect(result.current.locale).toBe("cs-CZ");
    });

    it("should return a translation", async () => {
        vi.resetModules();
        vi.doMock("../messages/en-US.json", () => messagesEN);
        vi.doMock("../messages/cs-CZ.json", () => messagesCZ);

        useRouterMock.mockReturnValue({
            locale: "cs-CZ",
            defaultLocale: "en-US",
        });

        const { useI18n } = await import("@/hooks/i18n");
        const { result } = renderHook(() => useI18n());

        await waitFor(() => {
            expect(result.current.t("test")).toBe("testCZ");
        });
    });
});
