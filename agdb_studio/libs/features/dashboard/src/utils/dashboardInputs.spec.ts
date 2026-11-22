import { beforeEach, describe, expect, it } from "vitest";
import { useContentInputs } from "@agdb-studio/common/src/composables/content/inputs";
import { KEY_MODAL } from "@agdb-studio/common/src/composables/modal/constants";
import { dashboardNameInput, modalInputValue } from "./dashboardInputs";

describe("dashboardInputs", () => {
  const { addInput, setInputValue, clearAllInputs } = useContentInputs();

  beforeEach(() => clearAllInputs());

  it("builds a focused name input with the value and a name rule", () => {
    const { input } = dashboardNameInput("ops");
    expect(input).toMatchObject({
      key: "name",
      type: "text",
      autofocus: true,
      required: true,
      value: "ops",
    });
    const [rule] = input.rules;
    expect(rule!(" ops ")).toBeUndefined();
    expect(rule!("bad name")).toBeDefined();
    expect(rule!(undefined as unknown as string)).toBeDefined();
  });

  it("can leave the focus to another input", () => {
    expect(dashboardNameInput("", false).input).toMatchObject({
      value: "",
      autofocus: false,
    });
  });

  it("reads a trimmed modal input, empty when missing", () => {
    addInput(KEY_MODAL, dashboardNameInput().input);
    setInputValue(KEY_MODAL, "name", "  ops ");
    expect(modalInputValue("name")).toBe("ops");
    expect(modalInputValue("missing")).toBe("");
  });
});
