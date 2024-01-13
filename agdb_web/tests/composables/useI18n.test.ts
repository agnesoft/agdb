import { describe, it, expect, beforeEach, beforeAll } from "vitest";
import useI18n from "@/composables/useI18n";

describe("useI18n", () => {
    const i18n = useI18n();
    beforeAll(async () => {
        await i18n.initI18n();
    });

    beforeEach(() => {
        i18n.setDefaultLocaleCode();
    });

    it("returns the correct default locale code", () => {
        expect(i18n.localeCode.value).toBe("en");
    });

    it("returns the correct locale exists", () => {
        expect(i18n.localeExists("en")).toBe(true);
        expect(i18n.localeExists("fr")).toBe(false);
    });

    it("returns the correct current locale", async () => {
        await i18n.setLocaleCode("cs");
        expect(i18n.currentLocale.value?.iso).toEqual("cs-CZ");
    });

    it("returns the correct fallback locale", () => {
        expect(i18n.fallbackLocale.value?.iso).toEqual("en-US");
    });

    it("returns the correct current page path", () => {
        expect(i18n.currentPagePath.value).toBe("/");
    });

    it("returns the correct locale path for en lang", () => {
        expect(i18n.getLocalePath("/")).toBe("/");
        expect(i18n.getLocalePath("/about")).toBe("/about");
        expect(i18n.getLocalePath("/en")).toBe("/");
        expect(i18n.getLocalePath("/en/about")).toBe("/about");
    });

    it("returns the correct locale path for cs lang", async () => {
        await i18n.setLocaleCode("cs");
        expect(i18n.getLocalePath("/")).toBe("/cs/");
        expect(i18n.getLocalePath("/about")).toBe("/cs/about");
        expect(i18n.getLocalePath("/cs")).toBe("/cs");
        expect(i18n.getLocalePath("/cs/about")).toBe("/cs/about");
    });

    it("does not change the locale code if it is the same", async () => {
        await i18n.setLocaleCode("en");
        expect(i18n.localeCode.value).toBe("en");
    });

    it("does not change the locale code if it does not exist", async () => {
        await i18n.setLocaleCode("fr");
        expect(i18n.localeCode.value).toBe("en");
    });

    it("loads the correct messages for en lang", () => {
        expect(i18n.t("url.about")).toBe("/about");
    });

    it("loads the correct messages for cs lang", async () => {
        await i18n.setLocaleCode("cs");
        expect(i18n.t("url.about")).toBe("/o-agdb");
    });

    it("does not reload the fallback messages on init", () => {
        i18n.initI18n();
        expect(i18n.t("url.about")).toBe("/about");
    });

    it("does not reload the messages for false locale", () => {
        i18n.localeCode.value = "fr";
        i18n.loadMessages();
        expect(i18n.t("url.about")).toBe("/about");
    });

    it("does not reload fallback messages for false fallback locale", () => {
        i18n.fallbackLocaleCode.value = "fr";
        i18n.loadFallbackMessages();
        expect(i18n.t("url.about")).toBe("/about");
    });

    it("sets default locale to en if fallback does not exist", () => {
        i18n.fallbackLocaleCode.value = "fr";
        i18n.setDefaultLocaleCode();
        expect(i18n.localeCode.value).toBe("en");
    });

    it("returns empty string if message does not exist", () => {
        expect(i18n.t("url.false")).toBe("");
    });
});
