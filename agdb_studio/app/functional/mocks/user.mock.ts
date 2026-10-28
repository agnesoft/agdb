import type { UserStatus } from "@agnesoft/agdb_api/openapi" with {
  "resolution-mode": "import",
};

export const MOCK_SESSIONS = {
  admin: [
    { agent: "Firefox", created: 1698400000, expires_at: 1698403600, session: "sess-admin-1" },
    { agent: "Chrome", created: 1698401000, expires_at: 1698404600, session: "sess-admin-2" },
  ],
  testuser: [
    { agent: "Safari", created: 1698402000, expires_at: 1698405600, session: "sess-test-1" },
  ],
};

export const MOCK_USER_LIST: UserStatus[] = [
  { username: "admin", admin: true, login: true, sessions: MOCK_SESSIONS.admin },
  { username: "testuser", admin: false, login: true, sessions: MOCK_SESSIONS.testuser },
  { username: "inactive", admin: false, login: false, sessions: [] },
];

export const MOCK_DB_USERS = [
  { username: "admin", role: "admin" },
  { username: "reader", role: "read" },
];
