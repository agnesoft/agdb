import type { ServerDatabase } from "@agnesoft/agdb_api/dist/openapi";

export type DbIdentification = Pick<ServerDatabase, "owner" | "db">;
