// Auth & user
export const OPENAPI_API = "**/api/v1/openapi.json";
export const LOGIN_API = "**/api/v1/user/login*";
export const LOGOUT_API = "**/api/v1/user/logout*";
export const USER_PROFILE_API = "**/api/v1/user/profile*";
export const USER_STATUS_API = "**/api/v1/user/status*";
export const USER_CHANGE_PASSWORD_API = "**/api/v1/user/change_password*";

// Database list (no path params)
export const DB_LIST_API = "**/api/v1/db/list*";
export const ADMIN_DB_LIST_API = "**/api/v1/admin/db/list*";

// Database actions: /api/v1/db/{owner}/{db}/{action}?query_params
export const DB_ADD_API = "**/api/v1/db/*/*/add*";
export const DB_BACKUP_API = "**/api/v1/db/*/*/backup*";
export const DB_RESTORE_API = "**/api/v1/db/*/*/restore*";
export const DB_CLEAR_API = "**/api/v1/db/*/*/clear*";
export const DB_CONVERT_API = "**/api/v1/db/*/*/convert*";
export const DB_REMOVE_API = "**/api/v1/db/*/*/remove*";
export const DB_DELETE_API = "**/api/v1/db/*/*/delete*";
export const DB_OPTIMIZE_API = "**/api/v1/db/*/*/optimize*";
export const DB_AUDIT_API = "**/api/v1/db/*/*/audit*";
export const DB_COPY_API = "**/api/v1/db/*/*/copy*";
export const DB_RENAME_API = "**/api/v1/db/*/*/rename*";
export const DB_ROLLBACK_API = "**/api/v1/db/*/*/rollback*";
export const DB_EXEC_API = "**/api/v1/db/*/*/exec?*";
export const DB_EXEC_MUT_API = "**/api/v1/db/*/*/exec_mut*";

// Database user management: /api/v1/db/{owner}/{db}/user/...
export const DB_USER_LIST_API = "**/api/v1/db/*/*/user/list*";
export const DB_USER_ADD_API = "**/api/v1/db/*/*/user/*/add*";
export const DB_USER_REMOVE_API = "**/api/v1/db/*/*/user/*/remove*";

// Admin database actions: /api/v1/admin/db/{owner}/{db}/{action}?query_params
export const ADMIN_DB_ADD_API = "**/api/v1/admin/db/*/*/add*";
export const ADMIN_DB_BACKUP_API = "**/api/v1/admin/db/*/*/backup*";
export const ADMIN_DB_RESTORE_API = "**/api/v1/admin/db/*/*/restore*";
export const ADMIN_DB_CLEAR_API = "**/api/v1/admin/db/*/*/clear*";
export const ADMIN_DB_CONVERT_API = "**/api/v1/admin/db/*/*/convert*";
export const ADMIN_DB_REMOVE_API = "**/api/v1/admin/db/*/*/remove*";
export const ADMIN_DB_DELETE_API = "**/api/v1/admin/db/*/*/delete*";
export const ADMIN_DB_OPTIMIZE_API = "**/api/v1/admin/db/*/*/optimize*";
export const ADMIN_DB_AUDIT_API = "**/api/v1/admin/db/*/*/audit*";
export const ADMIN_DB_COPY_API = "**/api/v1/admin/db/*/*/copy*";
export const ADMIN_DB_RENAME_API = "**/api/v1/admin/db/*/*/rename*";
export const ADMIN_DB_ROLLBACK_API = "**/api/v1/admin/db/*/*/rollback*";
export const ADMIN_DB_EXEC_API = "**/api/v1/admin/db/*/*/exec?*";
export const ADMIN_DB_EXEC_MUT_API = "**/api/v1/admin/db/*/*/exec_mut*";

// Admin database user management
export const ADMIN_DB_USER_LIST_API = "**/api/v1/admin/db/*/*/user/list*";
export const ADMIN_DB_USER_ADD_API = "**/api/v1/admin/db/*/*/user/*/add*";
export const ADMIN_DB_USER_REMOVE_API = "**/api/v1/admin/db/*/*/user/*/remove*";

// Admin user management: /api/v1/admin/user/{username}/{action} or /api/v1/admin/user/list
export const ADMIN_USER_LIST_API = "**/api/v1/admin/user/list*";
export const ADMIN_USER_ADD_API = "**/api/v1/admin/user/*/add*";
export const ADMIN_USER_DELETE_API = "**/api/v1/admin/user/*/delete*";
export const ADMIN_USER_CHANGE_PASSWORD_API =
  "**/api/v1/admin/user/*/change_password*";
export const ADMIN_USER_LOGOUT_API = "**/api/v1/admin/user/*/logout*";
export const CLUSTER_ADMIN_USER_LOGOUT_API =
  "**/api/v1/cluster/admin/user/*/logout*";

// Cluster
export const CLUSTER_STATUS_API = "**/api/v1/cluster/status*";
