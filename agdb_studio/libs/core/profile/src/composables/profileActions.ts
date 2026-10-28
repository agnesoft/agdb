import { client } from "@agdb-studio/api/src/api";
import { useAccount } from "@agdb-studio/auth/src/account";
import { useAuth } from "@agdb-studio/auth/src/auth";
import { useContentInputs } from "@agdb-studio/common/src/composables/content/inputs";
import { buildSessionContent } from "@agdb-studio/common/src/composables/content/sessionContent";
import type {
  Action,
  Content,
} from "@agdb-studio/common/src/composables/content/types";
import { convertArrayOfStringsToContent } from "@agdb-studio/common/src/composables/content/utils";
import { KEY_MODAL } from "@agdb-studio/common/src/composables/modal/constants";
import useModal from "@agdb-studio/common/src/composables/modal/modal";
import { getRouter } from "@agdb-studio/router/src/router";
import { formatTimestamp } from "@agdb-studio/utils/src/format/format";
import { computed } from "vue";
import { useAdmin } from "./admin";

const { logoutSession } = useAuth();
const { sessions, fetchUserStatus } = useAccount();
const { openModal } = useModal();
const { getInputValue } = useContentInputs();

export const USER_VIEW_KEY = "user-view";
export const ADMIN_VIEW_KEY = "admin";
export const SESSIONS_KEY = "sessions";
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
    key: SESSIONS_KEY,
    label: "Sessions",
    action: async () => {
      await fetchUserStatus();
      openModal({
        header: "Active Sessions",
        content: buildSessionContent(sessions.value),
        wide: true,
      });
      return true;
    },
  },
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
    action: async () => {
      await fetchUserStatus();
      const sessionList = sessions.value;
      const content: Content[] = [
        ...convertArrayOfStringsToContent(["Are you sure you want to logout?"]),
      ];

      if (sessionList.length > 1) {
        content.push({
          input: {
            type: "select",
            label: "Session",
            key: "session",
            options: [
              { value: "", label: "Current session" },
              { value: "all", label: "All sessions" },
              { value: "others", label: "Other sessions" },
              ...sessionList.map((s) => ({
                value: s.session,
                label: `${s.agent || "(none)"} — ${formatTimestamp(s.created)}`,
              })),
            ],
            value: "",
          },
        });
      }

      content.push({
        input: {
          type: "checkbox",
          label: "Logout from all nodes in the cluster",
          key: "cluster",
          value: true,
        },
      });

      openModal({
        header: "Logout",
        content,
        onConfirm: async () => {
          const cluster = !!getInputValue<string>(KEY_MODAL, "cluster");
          const sessionId =
            getInputValue<string>(KEY_MODAL, "session")?.toString() || "";

          if (!sessionId || sessionId === "all") {
            await logoutSession(cluster, sessionId || undefined);
            return true;
          }

          await logoutSession(cluster, sessionId);
          await fetchUserStatus();
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
