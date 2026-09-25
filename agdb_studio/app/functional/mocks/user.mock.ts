import type { UserStatus } from "@agnesoft/agdb_api/openapi" with {
  "resolution-mode": "import",
};

export const MOCK_USER_LIST: UserStatus[] = [
  { username: "admin", admin: true, login: true, sessions: [] },
  { username: "testuser", admin: false, login: true, sessions: [] },
  { username: "inactive", admin: false, login: false, sessions: [] },
];

export const MOCK_DB_USERS = [
  { username: "admin", role: "admin" },
  { username: "reader", role: "read" },
];
