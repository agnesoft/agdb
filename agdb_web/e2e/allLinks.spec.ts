import { test, expect, Page } from "@playwright/test";

const validatedLinks: string[] = [];

const validateLinks = async (page: Page) => {
    const links = await page
        .locator("a")
        .evaluateAll((els) => els.map((el) => el.getAttribute("href")));

    for (const href of links) {
        if (
            href &&
            !validatedLinks.includes(href) &&
            !href.startsWith("mailto") &&
            !href.startsWith("tel") &&
            !href.startsWith("javascript") &&
            !href.startsWith("#")
        ) {
            const sourcePageTitle = await page.title();
            const error = `${sourcePageTitle} -> ${href}`;

            await page.goto(href);

            const pageTitle = await page.title();
            expect(pageTitle.length, error).toBeGreaterThan(0);
            expect(pageTitle, error).not.toContain("404");

            validatedLinks.push(href);

            if (!href.startsWith("http")) {
                await validateLinks(page);
            }
        }
    }
};

test("should validate all links", async ({ page }) => {
    test.setTimeout(300000);

    await page.goto("http://localhost:5001/");

    await validateLinks(page);
});
