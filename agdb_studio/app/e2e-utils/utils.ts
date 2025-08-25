import { type Page } from "@playwright/test";

export const mockLogin = async (page: Page) => {
  await page.addInitScript(() => {
    window.localStorage.setItem("studio_token", "mocked-token");
  });
};
