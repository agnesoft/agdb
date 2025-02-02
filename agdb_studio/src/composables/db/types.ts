import type { ServerDatabase } from "@agnesoft/agdb_api/openapi";

export type DbIdentification = Pick<ServerDatabase, "owner" | "db">;
