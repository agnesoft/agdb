import { useAdmin } from "@agdb-studio/router/src/admin";
import { checkClient, client } from "@agdb-studio/api/src/api";
import type {
  DbAudit,
  DbResource,
  DbKind,
  DbUser,
  DbUserRole,
  ServerDatabase,
} from "@agnesoft/agdb_api";
import type { DbIdentification } from "./types";

const { isAdminView, isAdmin } = useAdmin();

const shouldRunAdminAction: () => boolean = () =>
  isAdminView.value && isAdmin.value;

export const dbAdd = async (params: {
  owner: string;
  db: string;
  db_type: DbKind;
}): Promise<number> => {
  checkClient(client);
  if (shouldRunAdminAction()) {
    return client.value.admin_db_add(params);
  }

  return client.value.db_add(params);
};

export const dbAudit = async (
  params: DbIdentification,
): Promise<[number, DbAudit]> => {
  checkClient(client);
  if (shouldRunAdminAction()) {
    return client.value.admin_db_audit(params);
  }

  return client.value.db_audit(params);
};

export const dbBackup = async (params: DbIdentification): Promise<number> => {
  checkClient(client);
  if (shouldRunAdminAction()) {
    return client.value.admin_db_backup(params);
  }

  return client.value.db_backup(params);
};

export const dbClear = async (
  params: DbIdentification & { resource: DbResource },
): Promise<[number, ServerDatabase]> => {
  checkClient(client);
  if (shouldRunAdminAction()) {
    return client.value.admin_db_clear(params);
  }

  return client.value.db_clear(params);
};

export const dbConvert = async (
  params: DbIdentification & { db_type: DbKind },
): Promise<number> => {
  checkClient(client);
  if (shouldRunAdminAction()) {
    return client.value.admin_db_convert(params);
  }

  return client.value.db_convert(params);
};

export const dbCopy = async (
  params: (DbIdentification & { new_db: string }) & {
    new_db: string;
    new_owner: string;
  },
): Promise<number> => {
  checkClient(client);
  if (shouldRunAdminAction()) {
    return client.value.admin_db_copy(params);
  }
  const { new_owner: _, ...rest } = params;

  return client.value.db_copy(rest);
};

export const dbDelete = async (params: DbIdentification): Promise<number> => {
  checkClient(client);
  if (shouldRunAdminAction()) {
    return client.value.admin_db_delete(params);
  }

  return client.value.db_delete(params);
};

export const dbList = async (): Promise<[number, ServerDatabase[]]> => {
  checkClient(client);
  if (shouldRunAdminAction()) {
    return client.value.admin_db_list();
  }

  return client.value.db_list();
};

export const dbOptimize = async (
  params: DbIdentification,
): Promise<[number, ServerDatabase]> => {
  checkClient(client);
  if (shouldRunAdminAction()) {
    return client.value.admin_db_optimize(params);
  }

  return client.value.db_optimize(params);
};

export const dbRemove = async (params: DbIdentification): Promise<number> => {
  checkClient(client);
  if (shouldRunAdminAction()) {
    return client.value.admin_db_remove(params);
  }

  return client.value.db_remove(params);
};

export const dbRename = async (
  params: (DbIdentification & { new_db: string }) & {
    new_db: string;
    new_owner: string;
  },
): Promise<number> => {
  checkClient(client);
  if (shouldRunAdminAction()) {
    return client.value.admin_db_rename(params);
  }
  const { new_owner: _, ...rest } = params;

  return client.value.db_rename(rest);
};

export const dbRestore = async (params: DbIdentification): Promise<number> => {
  checkClient(client);
  if (shouldRunAdminAction()) {
    return client.value.admin_db_restore(params);
  }

  return client.value.db_restore(params);
};

export const dbRollback = async (params: DbIdentification): Promise<number> => {
  checkClient(client);
  if (shouldRunAdminAction()) {
    return client.value.admin_db_rollback(params);
  }

  return client.value.db_rollback(params);
};

export const dbUserAdd = async (
  params: {
    username: string;
    db_role: DbUserRole;
  } & DbIdentification,
): Promise<number> => {
  checkClient(client);
  if (shouldRunAdminAction()) {
    return client.value.admin_db_user_add(params);
  }

  return client.value.db_user_add(params);
};

export const dbUserList = async (
  params: DbIdentification,
): Promise<[number, DbUser[]]> => {
  checkClient(client);
  if (shouldRunAdminAction()) {
    return client.value.admin_db_user_list(params);
  }

  return client.value.db_user_list(params);
};

export const dbUserRemove = async (
  params: {
    username: string;
  } & DbIdentification,
): Promise<number> => {
  checkClient(client);
  if (shouldRunAdminAction()) {
    return client.value.admin_db_user_remove(params);
  }

  return client.value.db_user_remove(params);
};
