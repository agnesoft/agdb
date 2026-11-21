import type { ServerDatabase } from "@agnesoft/agdb_api";

export type DbIdentification = Pick<ServerDatabase, "owner" | "db">;
