import { describe, expect, it, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { useI18n } from "@/hooks/i18n";
import { beforeEach } from "node:test";

const { useRounterMock, messagesEnMock } = vi.hoisted(() => {
    return {
        useRounterMock: vi.fn(),
        messagesEnMock: vi.fn(),
    };
});
vi.mock("nextra/hooks", () => ({
    useRouter: useRounterMock,
}));

vi.mock("nextra/constants", () => ({
    DEFAULT_LOCALE: "en-US",
}));

vi.mock("@/messages/cs-CZ.json", () => ({
    test: "testCZ",
}));

const messagesEN = {
    test: "testEN",
    test2: {
        test3: "testEN2",
    },
};

vi.mock("@/messages/en-US.json", messagesEnMock);

describe("i18n", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        vi.resetAllMocks();
    });

    it("should not return a fallback translation", async () => {
        messagesEnMock.mockRejectedValue(new Error("error"));
        useRounterMock.mockReturnValue({
            locale: "",
        });
        const { result } = renderHook(() => useI18n());
        await waitFor(() => {
            expect(result.current.t("test2.test3")).not.toBe("testEN2");
        });
    });

    it("should return a fallback translation", async () => {
        messagesEnMock.mockResolvedValue(messagesEN);
        useRounterMock.mockReturnValue({
            locale: "",
        });
        const { result } = renderHook(() => useI18n());
        await waitFor(() => {
            expect(result.current.t("test2.test3")).toBe("testEN2");
        });
    });

    it("should return the default locale", async () => {
        useRounterMock.mockReturnValue({
            locale: "cs-CZ",
        });

        const { result } = renderHook(() => useI18n());
        expect(result.current.locale).toBe("cs-CZ");
    });

    it("should return a translation", async () => {
        useRounterMock.mockReturnValue({
            locale: "cs-CZ",
        });
        const { result } = renderHook(() => useI18n());

        await waitFor(() => {
            expect(result.current.t("test")).toBe("testCZ");
        });
    });
});
