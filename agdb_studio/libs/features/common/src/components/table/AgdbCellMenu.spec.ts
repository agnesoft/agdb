import { mount } from "@vue/test-utils";
import AgdbCellMenu from "./AgdbCellMenu.vue";
import { describe, beforeEach, vi, it, expect } from "vitest";
import {
  INJECT_KEY_ROW,
  INJECT_KEY_TABLE_NAME,
} from "@/composables/table/constants";
import useModal from "@/composables/modal/modal";
import { convertArrayOfStringsToContent } from "@/composables/content/utils";
import DropdownContent from "../dropdown/DropdownContent.vue";
import type { TRow } from "@/composables/table/types";
import type { Action } from "@/composables/content/types";
const { fetchData } = vi.hoisted(() => {
  return {
    fetchData: vi.fn(),
  };
});
const { modalIsVisible, onConfirm, modal, closeModal } = useModal();

const dbActions: Action<TRow>[] = [
  {
    key: "audit",
    label: "Audit",
    action: vi.fn(),
  },
  {
    key: "backup",
    label: "Backup",

    action: vi.fn(),
  },
];

vi.mock("@/composables/table/tableConfig", () => {
  return {
    fetchData,
  };
});
describe("AgdbCellMenu", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    closeModal();
  });
  it("should open and close on click", async () => {
    const wrapper = mount(AgdbCellMenu, {
      props: {
        actions: dbActions as unknown as Action<TRow>[],
      },
      global: {
        provide: {
          [INJECT_KEY_ROW]: {
            value: {
              role: "admin",
              owner: "admin",
              db: "test",
              db_type: "memory",
              size: 2656,
              backup: 0,
            },
          },
          [INJECT_KEY_TABLE_NAME]: "databases",
        },
      },
    });
    const trigger = wrapper.find(".trigger");
    const dropdown = wrapper.findComponent(DropdownContent);
    expect(dropdown.isVisible()).toBe(false);
    trigger.trigger("click");
    await wrapper.vm.$nextTick();
    expect(dropdown.isVisible()).toBe(true);
    trigger.trigger("click");
    await wrapper.vm.$nextTick();
    expect(dropdown.isVisible()).toBe(false);
  });
  it("should call action on click when no confirmation required", async () => {
    const wrapper = mount(AgdbCellMenu, {
      props: {
        actions: dbActions as unknown as Action<TRow>[],
      },
      global: {
        provide: {
          [INJECT_KEY_ROW]: {
            value: {
              role: "admin",
              owner: "admin",
              db: "test",
              db_type: "memory",
              size: 2656,
              backup: 0,
            },
          },
          [INJECT_KEY_TABLE_NAME]: "databases",
        },
      },
    });

    const trigger = wrapper.find(".trigger");
    const dropdown = wrapper.findComponent(DropdownContent);
    expect(dropdown.isVisible()).toBe(false);
    trigger.trigger("click");
    await wrapper.vm.$nextTick();
    expect(dropdown.isVisible()).toBe(true);
    const action = dropdown.find(".menu-item[data-key=audit]");
    await action.trigger("click");
    await wrapper.vm.$nextTick();
    expect(dropdown.isVisible()).toBe(false);
    expect(fetchData).toHaveBeenCalledOnce();
  });
  it("should open the modal on click when confirmation is required", async () => {
    const deleteAction = vi.fn();
    const question = "Are you sure you want to delete this database?";
    const header = "Delete Database";
    const deleteConfirmation = convertArrayOfStringsToContent([
      question,
      "This will permanently delete all data.",
    ]);
    const wrapper = mount(AgdbCellMenu, {
      props: {
        actions: [
          {
            key: "delete",
            label: "Delete",
            action: deleteAction,
            confirmation: deleteConfirmation,
            confirmationHeader: header,
          },
        ],
      },
      global: {
        provide: {
          [INJECT_KEY_ROW]: {
            value: {
              role: "admin",
              owner: "admin",
              db: "test",
              db_type: "memory",
              size: 2656,
              backup: 0,
            },
          },
          [INJECT_KEY_TABLE_NAME]: "databases",
        },
      },
    });

    const trigger = wrapper.find(".trigger");
    const dropdown = wrapper.findComponent(DropdownContent);
    expect(dropdown.isVisible()).toBe(false);
    trigger.trigger("click");
    await wrapper.vm.$nextTick();
    expect(dropdown.isVisible()).toBe(true);
    const action = dropdown.find(".menu-item[data-key=delete]");
    await action.trigger("click");
    await wrapper.vm.$nextTick();
    expect(dropdown.isVisible()).toBe(false);
    expect(modalIsVisible.value).toBe(true);
    onConfirm.value?.();
    expect(deleteAction).toHaveBeenCalledOnce();
    expect(modal.content[0].paragraph?.at(0)?.text).toBe(question);
    expect(modal.header).toBe(header);
  });
  it("should create confirmation content from function", async () => {
    const deleteAction = vi.fn();
    const question = "Are you sure you want to delete this database?";
    const header = "Delete Database";
    const deleteConfirmation = vi
      .fn()
      .mockReturnValue(
        convertArrayOfStringsToContent([
          question,
          "This will permanently delete all data.",
        ]),
      );
    const wrapper = mount(AgdbCellMenu, {
      props: {
        actions: [
          {
            key: "delete",
            label: "Delete",
            action: deleteAction,
            confirmation: deleteConfirmation,
            confirmationHeader: header,
          },
        ],
      },
      global: {
        provide: {
          [INJECT_KEY_ROW]: {
            value: {
              role: "admin",
              owner: "admin",
              db: "test",
              db_type: "memory",
              size: 2656,
              backup: 0,
            },
          },
          [INJECT_KEY_TABLE_NAME]: "databases",
        },
      },
    });

    const trigger = wrapper.find(".trigger");
    const dropdown = wrapper.findComponent(DropdownContent);
    expect(dropdown.isVisible()).toBe(false);
    trigger.trigger("click");
    await wrapper.vm.$nextTick();
    expect(dropdown.isVisible()).toBe(true);
    const action = dropdown.find(".menu-item[data-key=delete]");
    await action.trigger("click");
    await wrapper.vm.$nextTick();
    expect(dropdown.isVisible()).toBe(false);
    expect(modalIsVisible.value).toBe(true);
    onConfirm.value?.();
    expect(deleteAction).toHaveBeenCalledOnce();
    expect(modal.content[0].paragraph?.at(0)?.text).toBe(question);
    expect(modal.header).toBe(header);
  });

  it("should handle if row is not provided", async () => {
    const deleteAction = vi.fn();
    const header = "Delete Database";
    const wrapper = mount(AgdbCellMenu, {
      props: {
        actions: [
          {
            key: "delete",
            label: "Delete",
            action: deleteAction,
            confirmationHeader: header,
          },
        ],
      },
      global: {
        provide: {
          [INJECT_KEY_TABLE_NAME]: "databases",
        },
      },
    });

    const trigger = wrapper.find(".trigger");
    const dropdown = wrapper.findComponent(DropdownContent);
    expect(dropdown.isVisible()).toBe(false);
    trigger.trigger("click");
    await wrapper.vm.$nextTick();
    expect(dropdown.isVisible()).toBe(true);
    const action = dropdown.find(".menu-item");
    expect(action.exists()).toBe(false);
  });
  it("should not close the dropdown if item has no action", async () => {
    const wrapper = mount(AgdbCellMenu, {
      props: {
        actions: dbActions as unknown as Action<TRow>[],
      },
      global: {
        provide: {
          [INJECT_KEY_ROW]: {
            value: {
              role: "admin",
              owner: "admin",
              db: "test",
              db_type: "memory",
              size: 2656,
              backup: 0,
            },
          },
          [INJECT_KEY_TABLE_NAME]: "databases",
        },
      },
    });

    const trigger = wrapper.find(".trigger");
    const dropdown = wrapper.findComponent(DropdownContent);
    expect(dropdown.isVisible()).toBe(false);
    trigger.trigger("click");
    await wrapper.vm.$nextTick();
    expect(dropdown.isVisible()).toBe(true);
    const action = dropdown.find(".menu-item[data-key=convert]");
    await action.trigger("click");
    await wrapper.vm.$nextTick();
    expect(dropdown.isVisible()).toBe(true);
  });

  it("should use header function if provided", async () => {
    const deleteAction = vi.fn();
    const question = "Are you sure you want to delete this database?";
    const header = vi.fn().mockReturnValue("Test Header");
    const deleteConfirmation = convertArrayOfStringsToContent([
      question,
      "This will permanently delete all data.",
    ]);
    const wrapper = mount(AgdbCellMenu, {
      props: {
        actions: [
          {
            key: "delete",
            label: "Delete",
            action: deleteAction,
            confirmation: deleteConfirmation,
            confirmationHeader: header,
          },
        ],
      },
      global: {
        provide: {
          [INJECT_KEY_ROW]: {
            value: {
              role: "admin",
              owner: "admin",
              db: "test",
              db_type: "memory",
              size: 2656,
              backup: 0,
            },
          },
          [INJECT_KEY_TABLE_NAME]: "databases",
        },
      },
    });
    const trigger = wrapper.find(".trigger");
    const dropdown = wrapper.findComponent(DropdownContent);
    expect(dropdown.isVisible()).toBe(false);
    trigger.trigger("click");
    await wrapper.vm.$nextTick();
    expect(dropdown.isVisible()).toBe(true);
    const action = dropdown.find(".menu-item[data-key=delete]");
    await action.trigger("click");
    await wrapper.vm.$nextTick();
    expect(dropdown.isVisible()).toBe(false);
    expect(header).toHaveBeenCalled();
    expect(modal.content[0].paragraph?.at(0)?.text).toBe(question);
    expect(modal.header).toBe("Test Header");
  });
  it("should set the header to the default if no header function is provided", async () => {
    const deleteAction = vi.fn();
    const question = "Are you sure you want to delete this database?";
    const deleteConfirmation = convertArrayOfStringsToContent([
      question,
      "This will permanently delete all data.",
    ]);
    const wrapper = mount(AgdbCellMenu, {
      props: {
        actions: [
          {
            key: "delete",
            label: "Delete",
            action: deleteAction,
            confirmation: deleteConfirmation,
          },
        ],
      },
      global: {
        provide: {
          [INJECT_KEY_ROW]: {
            value: {
              role: "admin",
              owner: "admin",
              db: "test",
              db_type: "memory",
              size: 2656,
              backup: 0,
            },
          },
          [INJECT_KEY_TABLE_NAME]: "databases",
        },
      },
    });
    const trigger = wrapper.find(".trigger");
    const dropdown = wrapper.findComponent(DropdownContent);
    expect(dropdown.isVisible()).toBe(false);
    trigger.trigger("click");
    await wrapper.vm.$nextTick();
    expect(dropdown.isVisible()).toBe(true);
    const action = dropdown.find(".menu-item[data-key=delete]");
    await action.trigger("click");
    await wrapper.vm.$nextTick();
    expect(dropdown.isVisible()).toBe(false);
    expect(modal.content[0].paragraph?.at(0)?.text).toBe(question);
    expect(modal.header).toBe("Confirm action");
  });
});
