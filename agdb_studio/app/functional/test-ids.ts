import { createTestIds, type TestId } from "./fixtures/test-id";

export const testIds = createTestIds({
  // Login
  LOGIN_FORM: "login_form",
  INPUT_SERVER: "inputServer",
  INPUT_USERNAME: "inputUsername",
  INPUT_PASSWORD: "inputPassword",
  BUTTON_LOGIN: "buttonLogin",
  ERROR_MESSAGE: "errorMessage",

  // Profile / nav
  PROFILE_DROPDOWN: "profile-dropdown",

  // Database view
  DB_TABLE: "db-table",
  DB_NAME_INPUT: "db-name-input",
  ADD_DB_BUTTON: "add-db-button",
  EMPTY_TABLE_MESSAGE: "empty-table-message",
  REFRESH_BUTTON: "refresh-button",

  // Table
  TABLE_ROW: "table-row",

  // Notifications
  NOTIFICATION_ICON: "notification-icon",
  NOTIFICATION_ITEM: "notification-item",
  NOTIFICATION_NEW_ICON: "notification-new-icon",

  // Modal
  CLOSE_MODAL: "close-modal",

  // Query builder
  QUERY_ROW: "query-row",
  QUERY_ROW_INDEX: "query-row-index",
  QUERY_STEP: "query-step",
  QUERY_HIDDEN_SUMMARY: "query-hidden-summary",
  QUERY_GRAPH: "query-graph",

  // Query results
  RESULTS_CHIP: "results-chip",
  RESULT_SCALAR: "result-scalar",
  RESULTS_TABLE: "results-table",
  RESULTS_TABLE_ROW: "results-table-row",
  RESULTS_ROW_MENU: "results-row-menu",
  RESULTS_JSON: "results-json",
  VIEW_GRAPH: "view-graph",
  VIEW_TABLE: "view-table",
  VIEW_JSON: "view-json",
  ELEMENT_DETAILS_PANEL: "element-details-panel",
  PANEL_VALUE_ROW: "panel-value-row",
  PANEL_VALUE_INPUT: "panel-value-input",
  PANEL_SAVE: "panel-save",

  // Cluster
  CROWN_ICON: "crown-icon",
  ACTIVE_SERVER_CROWN_ICON: "active-server-crown-icon",
});

export const dynamicIds = {
  menuItem: (key: string) => `menu-item-${key}` as unknown as TestId,
  modalButton: (text: string) =>
    `modal-button-${text.replace(/\s+/g, "-").toLowerCase()}` as unknown as TestId,
  tableCell: (key: string) => `table-cell-${key}` as unknown as TestId,
};
