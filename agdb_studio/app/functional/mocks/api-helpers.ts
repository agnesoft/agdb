import { type Page } from "@playwright/test";
import {
  ADMIN_DB_LIST_API,
  ADMIN_USER_LIST_API,
  CLUSTER_STATUS_API,
  DB_AUDIT_API,
  DB_EXEC_API,
  DB_LIST_API,
  DB_USER_LIST_API,
  LOGIN_API,
  OPENAPI_API,
  USER_STATUS_API,
} from "../api-paths";
import { MOCK_TOKEN_RESPONSE, MOCK_USER_STATUS } from "./auth.mock";
import { MOCK_DATABASE_LIST } from "./db.mock";
import { MOCK_USER_LIST, MOCK_DB_USERS, MOCK_SESSIONS } from "./user.mock";
import type {
  ServerDatabase,
  UserStatus,
} from "@agnesoft/agdb_api/openapi" with {
  "resolution-mode": "import",
};
import openapi from "../../../../agdb_server/openapi.json";

export async function mockOpenApi(page: Page) {
  await page.route(OPENAPI_API, async (route) => {
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(openapi),
    });
  });
}

export async function mockLoginApi(page: Page) {
  await page.route(LOGIN_API, async (route) => {
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(MOCK_TOKEN_RESPONSE),
    });
  });
}

export async function mockUserStatusApi(page: Page) {
  await page.route(USER_STATUS_API, async (route) => {
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(MOCK_USER_STATUS),
    });
  });
}

export async function mockLogin(page: Page) {
  await page.addInitScript(() => {
    window.localStorage.setItem("studio_token", "mocked-token");
  });
}

export async function mockDatabaseListApi(
  page: Page,
  databaseList: ServerDatabase[] = MOCK_DATABASE_LIST,
) {
  await page.route(DB_LIST_API, async (route) => {
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(databaseList),
    });
  });
  await page.route(CLUSTER_STATUS_API, async (route) => {
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify([]),
    });
  });
}

export async function mockAllApis(page: Page) {
  await mockOpenApi(page);
  await mockLoginApi(page);
  await mockUserStatusApi(page);
}

export async function mockAdminUserStatus(page: Page) {
  await page.route(USER_STATUS_API, async (route) => {
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        username: "admin",
        admin: true,
        login: true,
        sessions: [],
      }),
    });
  });
}

export async function mockUserStatusWithSessions(page: Page) {
  await page.route(USER_STATUS_API, async (route) => {
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        username: "testuser",
        admin: false,
        login: true,
        sessions: MOCK_SESSIONS.admin,
      }),
    });
  });
}

export async function mockAdminDatabaseListApi(
  page: Page,
  databaseList: ServerDatabase[] = MOCK_DATABASE_LIST,
) {
  await page.route(ADMIN_DB_LIST_API, async (route) => {
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(databaseList),
    });
  });
  await page.route(CLUSTER_STATUS_API, async (route) => {
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify([]),
    });
  });
}

export async function mockAdminUserListApi(
  page: Page,
  users: UserStatus[] = MOCK_USER_LIST,
) {
  await page.route(ADMIN_USER_LIST_API, async (route) => {
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(users),
    });
  });
}

export async function mockDbActionApi(page: Page, apiPath: string) {
  await page.route(apiPath, async (route) => {
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({}),
    });
  });
}

export async function mockDbAuditApi(page: Page) {
  await page.route(DB_AUDIT_API, async (route) => {
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify([
        {
          timestamp: 1704067200,
          username: "admin",
          query: { SelectValues: { keys: [], ids: { Ids: [1] } } },
        },
      ]),
    });
  });
}

export async function mockDbUserListApi(
  page: Page,
  users: Array<{ username: string; role: string }> = MOCK_DB_USERS,
) {
  await page.route(DB_USER_LIST_API, async (route) => {
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(users),
    });
  });
}

export async function mockDbExecApi(page: Page) {
  await page.route(DB_EXEC_API, async (route) => {
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ result: [] }),
    });
  });
}

export function enableConsoleLogging(page: Page) {
  page.on("console", (msg) => console.log(msg.text()));
}

// Navigation helpers — wait for the API response that triggers rendering

export async function gotoDbPage(page: Page) {
  await withResponse(page, DB_LIST_API, () => page.goto("/studio/db"));
}

export async function gotoAdminDbPage(page: Page) {
  await withResponse(page, ADMIN_DB_LIST_API, () =>
    page.goto("/studio/admin/db"),
  );
}

export async function gotoAdminUsersPage(page: Page) {
  await withResponse(page, ADMIN_USER_LIST_API, () =>
    page.goto("/studio/admin/users"),
  );
}

export async function gotoLoginPage(page: Page) {
  await page.goto("/studio/login");
  // Login page doesn't fetch data — just wait for the form
}

export async function gotoAndWaitForLoad(page: Page, path: string) {
  await page.goto(path);
  await page.waitForLoadState("load");
  await page.locator("#app > *").first().waitFor({ state: "attached" });
}

export async function withResponse(
  page: Page,
  apiPattern: string,
  action: () => Promise<unknown>,
) {
  const response = page.waitForResponse(apiPattern);
  await action();
  await response;
}
