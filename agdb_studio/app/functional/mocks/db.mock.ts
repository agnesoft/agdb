import type { ServerDatabase } from "@agnesoft/agdb_api/openapi" with {
  "resolution-mode": "import",
};

export const MOCK_DATABASE_LIST: ServerDatabase[] = [
  {
    db: "users",
    owner: "admin",
    db_type: "memory",
    role: "admin",
    size: 2568,
    backup: 0,
    created: 0,
  },
  {
    db: "orders",
    owner: "admin",
    db_type: "memory",
    role: "admin",
    size: 2568,
    backup: 1754213481,
    created: 0,
  },
  {
    db: "products",
    owner: "admin",
    db_type: "memory",
    role: "admin",
    size: 2568,
    backup: 0,
    created: 0,
  },
];
