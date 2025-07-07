import { useAuth } from "@agdb-studio/auth/src/auth";
import useModal from "@agdb-studio/common/src/composables/modal/modal";
import { useContentInputs } from "@agdb-studio/common/src/composables/content/inputs";
import { convertArrayOfStringsToContent } from "@agdb-studio/common/src/composables/content/utils";
import { client } from "@agdb-studio/api/src/api";
import { KEY_MODAL } from "@agdb-studio/common/src/composables/modal/constants";
import { computed } from "vue";
import { useAdmin } from "./admin";
import { getRouter } from "@agdb-studio/router/src/router";
import type { Action } from "@agdb-studio/common/src/composables/content/types";

const { logout } = useAuth();
const { openModal } = useModal();
const { getInputValue } = useContentInputs();

export const USER_VIEW_KEY = "user-view";
export const ADMIN_VIEW_KEY = "admin";
export const CHANGE_PASSWORD_KEY = "change-password";
export const LOGOUT_KEY = "logout";

const adminActions: Action<undefined>[] = [
  {
    key: USER_VIEW_KEY,
    label: "User View",
    action: () => {
      getRouter().push({ name: "home" });
      return true;
    },
  },
];

const toAdminView: Action<undefined>[] = [
  {
    key: ADMIN_VIEW_KEY,
    label: "Admin View",
    action: () => {
      getRouter().push({ name: "admin" });
      return true;
    },
  },
];

const accountActions: Action<undefined>[] = [
  {
    key: CHANGE_PASSWORD_KEY,
    label: "Change password",
    action: () => {
      openModal({
        header: "Change password",
        content: [
          {
            input: {
              type: "password",
              label: "Current password",
              key: "currentPassword",
              required: true,
              autofocus: true,
            },
          },
          {
            input: {
              type: "password",
              label: "New password",
              key: "newPassword",
              required: true,
              rules: [
                (value: string) => {
                  if (value.length < 8) {
                    return "Password must be at least 8 characters long";
                  }
                  return undefined;
                },
              ],
            },
          },
          {
            input: {
              type: "password",
              label: "Confirm new password",
              key: "confirmNewPassword",
              required: true,
              rules: [
                (value: string) => {
                  if (
                    value !== getInputValue<string>(KEY_MODAL, "newPassword")
                  ) {
                    return "Passwords do not match";
                  }
                  return undefined;
                },
              ],
            },
          },
        ],
        onConfirm: async () => {
          const currentPassword = getInputValue<string>(
            KEY_MODAL,
            "currentPassword",
          );
          const newPassword = getInputValue<string>(KEY_MODAL, "newPassword");
          const confirmNewPassword = getInputValue(
            KEY_MODAL,
            "confirmNewPassword",
          );
          if (newPassword !== confirmNewPassword) {
            return false;
          }
          const response = await client.value?.user_change_password(null, {
            password: currentPassword,
            new_password: newPassword,
          });
          return response !== undefined;
        },
      });
      return true;
    },
  },
  {
    key: LOGOUT_KEY,
    label: "Logout",
    action: () => {
      openModal({
        header: "Logout",
        content: [
          ...convertArrayOfStringsToContent([
            "Are you sure you want to logout?",
          ]),
          {
            input: {
              type: "checkbox",
              label: "Logout from all nodes in the cluster",
              key: "cluster",
              value: false,
            },
          },
        ],
        onConfirm: () => {
          const cluster = !!getInputValue<string>(KEY_MODAL, "cluster");

          logout(cluster);
          return true;
        },
      });
      return true;
    },
  },
];

export const useUserActions = () => {
  const { isAdmin, isAdminView } = useAdmin();
  const actions = computed<Action<undefined>[]>(() => [
    ...(isAdmin.value ? (!isAdminView.value ? toAdminView : adminActions) : []),
    ...accountActions,
  ]);

  return {
    actions,
  };
};
