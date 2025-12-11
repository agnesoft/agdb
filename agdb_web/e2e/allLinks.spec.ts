import { test, expect, type Page } from "@playwright/test";

const validatedLinks: string[] = [];
const externalLinks: Map<Promise<Response>, string> = new Map();

const validateLinks = async (page: Page) => {
  const links = await page
    .locator("a")
    .evaluateAll((els) => els.map((el) => el.getAttribute("href")));
  const sourcePageTitle = await page.title();

  for (const href of links) {
    if (
      href &&
      !validatedLinks.includes(href) &&
      !href.includes("reddit.com") &&
      !href.startsWith("mailto") &&
      !href.startsWith("tel") &&
      !href.startsWith("javascript") &&
      !href.startsWith("#")
    ) {
      validatedLinks.push(href);

      if (!href.startsWith("http")) {
        await page.goto(href);
        await validateLinks(page);
      } else {
        externalLinks.set(fetch(href), `${sourcePageTitle} -> ${href}`);
      }
    }
  }

  for (const [f, error] of externalLinks) {
    const r = await f;
    expect(r.status, error).not.toBe(404);
  }
};

test("should validate all links", async ({ page }) => {
  test.setTimeout(300000);
  await page.goto("http://localhost:3000/");
  await validateLinks(page);
});
