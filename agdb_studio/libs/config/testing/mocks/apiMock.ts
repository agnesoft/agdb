import { vi } from "vitest";

const status = <T>(value: T) => vi.fn().mockResolvedValue(value);

export const user_login = status(200);
export const cluster_user_login = status(200);
export const user_logout = status(201);
export const user_logout_all = status(201);
export const user_logout_others = status(201);
export const user_logout_session = status(201);
export const cluster_user_logout = status(201);
export const cluster_user_logout_all = status(201);
export const cluster_user_logout_others = status(201);
export const cluster_user_logout_session = status(201);
export const user_status = vi.fn();
export const user_change_password = status(201);
export const db_list = vi.fn();
export const db_add = vi.fn();
export const db_backup = status(201);
export const db_restore = status(201);
export const db_rollback = status(201);
export const db_clear = status([201, {}]);
export const db_convert = status(201);
export const db_remove = status(204);
export const db_delete = status(204);
export const db_optimize = status([200, {}]);
export const db_audit = status([200, []]);
export const db_copy = status(201);
export const db_rename = status(201);
export const db_user_list = status([200, []]);
export const db_user_add = status(201);
export const db_user_remove = status(204);
export const db_exec = status([200, []]);
export const db_exec_mut = status([200, []]);
export const admin_db_list = vi.fn();
export const admin_db_add = vi.fn();
export const admin_db_backup = status(201);
export const admin_db_restore = status(201);
export const admin_db_rollback = status(201);
export const admin_db_clear = status([201, {}]);
export const admin_db_convert = status(201);
export const admin_db_remove = status(204);
export const admin_db_delete = status(204);
export const admin_db_optimize = status([200, {}]);
export const admin_db_audit = status([200, []]);
export const admin_db_copy = status(201);
export const admin_db_rename = status(201);
export const admin_db_user_list = status([200, []]);
export const admin_db_user_add = status(201);
export const admin_db_user_remove = status(204);
export const admin_db_exec = status([200, []]);
export const admin_db_exec_mut = status([200, []]);
export const admin_user_list = status([200, []]);
export const admin_user_add = status(201);
export const admin_user_delete = status(204);
export const admin_user_logout = status(201);
export const admin_user_logout_session = status(201);
export const admin_user_logout_all = status(201);
export const cluster_admin_user_logout = status(201);
export const cluster_admin_user_logout_session = status(201);
export const cluster_admin_user_logout_all = status(201);
export const admin_user_change_password = status(201);

/** The client `AgdbApi.new` and `new AgdbApi` make. */
export const api = {
  token: undefined as string | undefined,
  client: { quiet: false },
  address: vi.fn().mockReturnValue("localhost:3000"),
  base_url: vi.fn().mockReturnValue("http://localhost:3000/api/v1"),
  status: status(200),
  user_login,
  cluster_user_login,
  user_logout,
  user_logout_all,
  user_logout_others,
  user_logout_session,
  cluster_user_logout,
  cluster_user_logout_all,
  cluster_user_logout_others,
  cluster_user_logout_session,
  user_status,
  user_change_password,

  db_list,
  db_add,
  db_backup,
  db_restore,
  db_rollback,
  db_clear,
  db_convert,
  db_remove,
  db_delete,
  db_optimize,
  db_audit,
  db_copy,
  db_rename,
  db_user_list,
  db_user_add,
  db_user_remove,
  db_exec,
  db_exec_mut,

  admin_db_list,
  admin_db_add,
  admin_db_backup,
  admin_db_restore,
  admin_db_rollback,
  admin_db_clear,
  admin_db_convert,
  admin_db_remove,
  admin_db_delete,
  admin_db_optimize,
  admin_db_audit,
  admin_db_copy,
  admin_db_rename,
  admin_db_user_list,
  admin_db_user_add,
  admin_db_user_remove,
  admin_db_exec,
  admin_db_exec_mut,

  admin_user_list,
  admin_user_add,
  admin_user_delete,
  admin_user_logout,
  admin_user_logout_session,
  admin_user_logout_all,
  cluster_admin_user_logout,
  cluster_admin_user_logout_session,
  cluster_admin_user_logout_all,
  admin_user_change_password,
};

export const newApi = vi.fn().mockReturnValue(api);

vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  const original = await importOriginal<typeof import("@agnesoft/agdb_api")>();
  const AgdbApi = vi.fn(function () {
    return newApi();
  });
  return {
    ...original,
    AgdbApi: Object.assign(AgdbApi, { new: newApi }),
  };
});
