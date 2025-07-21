import { test as setup } from "@playwright/test";
import { LOGIN_API, OPENAPI_API, USER_STATUS_API } from "./apiPaths";
import openapi from "../../../agdb_server/openapi.json";

setup.beforeEach("setup api", async ({ page }) => {
  // Mock the OpenAPI schema
  await page.route(OPENAPI_API, async (route) => {
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(openapi),
    });
  });

  // Mock the login API
  await page.route(LOGIN_API, async (route) => {
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ token: "mocked-token" }),
    });
  });

  // Mock the user status API
  await page.route(USER_STATUS_API, async (route) => {
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ username: "testuser", admin: false, login: true }),
    });
  });
});
export const test = setup;
