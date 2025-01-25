import type { ServerDatabase } from "agdb_api/dist/openapi";

export type DbIdentification = Pick<ServerDatabase, "owner" | "db">;
