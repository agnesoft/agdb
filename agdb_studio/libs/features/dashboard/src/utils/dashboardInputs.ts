import { useContentInputs } from "@agdb-studio/common/src/composables/content/inputs";
import { KEY_MODAL } from "@agdb-studio/common/src/composables/modal/constants";
import { validateDashboardName } from "./dashboardDocument";

/** The modal input for a dashboard name. */
export const dashboardNameInput = (value = "", autofocus = true) => ({
  input: {
    key: "name",
    label: "Name (letters, digits, '_', '-', '.')",
    type: "text" as const,
    autofocus,
    required: true,
    value,
    rules: [(v: string) => validateDashboardName(String(v ?? "").trim())],
  },
});

/** The trimmed value of a modal input, e.g. the name from `dashboardNameInput`. */
export const modalInputValue = (key: string): string =>
  String(useContentInputs().getInputValue(KEY_MODAL, key) ?? "").trim();
