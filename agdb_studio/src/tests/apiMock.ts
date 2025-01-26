import { vi } from "vitest";

export const get_token = vi.fn();
export const user_status = vi.fn();
export const db_list = vi.fn();
export const db_add = vi.fn();
export const db_backup = vi.fn().mockResolvedValue({});
export const db_restore = vi.fn().mockResolvedValue({});
export const db_clear = vi.fn().mockResolvedValue({});
export const db_convert = vi.fn().mockResolvedValue({});
export const db_remove = vi.fn().mockResolvedValue({});
export const db_delete = vi.fn().mockResolvedValue({});
export const db_optimize = vi.fn().mockResolvedValue({});
export const db_audit = vi.fn().mockResolvedValue({ data: [] });
export const db_copy = vi.fn().mockResolvedValue({});
export const db_rename = vi.fn().mockResolvedValue({});
export const db_user_list = vi.fn().mockResolvedValue({ data: [] });
export const db_user_add = vi.fn().mockResolvedValue({});
export const db_user_remove = vi.fn().mockResolvedValue({});
export const db_exec = vi.fn().mockResolvedValue({});
export const db_exec_mut = vi.fn().mockResolvedValue({});
export const user_change_password = vi.fn().mockResolvedValue(true);
export const admin_db_list = vi.fn();
export const admin_db_add = vi.fn();
export const admin_db_backup = vi.fn().mockResolvedValue({});
export const admin_db_restore = vi.fn().mockResolvedValue({});
export const admin_db_clear = vi.fn().mockResolvedValue({});
export const admin_db_convert = vi.fn().mockResolvedValue({});
export const admin_db_remove = vi.fn().mockResolvedValue({});
export const admin_db_delete = vi.fn().mockResolvedValue({});
export const admin_db_optimize = vi.fn().mockResolvedValue({});
export const admin_db_audit = vi.fn().mockResolvedValue({ data: [] });
export const admin_db_copy = vi.fn().mockResolvedValue({});
export const admin_db_rename = vi.fn().mockResolvedValue({});
export const admin_db_user_list = vi.fn().mockResolvedValue({ data: [] });
export const admin_db_user_add = vi.fn().mockResolvedValue({});
export const admin_db_user_remove = vi.fn().mockResolvedValue({});
export const admin_db_exec = vi.fn().mockResolvedValue({});
export const admin_db_exec_mut = vi.fn().mockResolvedValue({});
export const admin_user_list = vi.fn().mockResolvedValue({ data: [] });
export const admin_user_add = vi.fn().mockResolvedValue({});
export const admin_user_remove = vi.fn().mockResolvedValue({});
export const admin_user_logout = vi.fn().mockResolvedValue({});
export const cluster_admin_user_logout = vi.fn().mockResolvedValue({});
export const admin_user_change_password = vi.fn().mockResolvedValue(true);

export const client = vi.fn().mockResolvedValue({
    login: vi.fn().mockResolvedValue("token"),
    logout: vi.fn().mockResolvedValue(undefined),
    set_token: vi.fn(),
    get_token,
    reset_token: vi.fn(),
    interceptors: {
        request: {
            use: vi.fn(),
        },
        response: {
            use: vi.fn(),
        },
    },
    user_status,
    user_change_password,

    db_list,
    db_add,
    db_backup,
    db_restore,
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
    admin_user_remove,
    admin_user_logout,
    cluster_admin_user_logout,
    admin_user_change_password,
});
vi.mock("agdb_api", () => {
    return {
        AgdbApi: {
            client,
        },
    };
});
