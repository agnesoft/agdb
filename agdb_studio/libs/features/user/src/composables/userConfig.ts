import type { UserStatus } from "@agnesoft/agdb_api/openapi";
import { useContentInputs } from "@agdb-studio/common/src/composables/content/inputs";
import { client } from "@agdb-studio/api/src/api";
import { KEY_MODAL } from "@agdb-studio/common/src/composables/modal/constants";
import { addNotification } from "@agdb-studio/notification/src/composables/notificationStore";
import { convertArrayOfStringsToContent } from "@agdb-studio/common/src/composables/content/utils";
import useModal from "@agdb-studio/common/src/composables/modal/modal";
import type {
  Column,
  TRow,
} from "@agdb-studio/common/src/composables/table/types";
import type {
  Action,
  Content,
} from "@agdb-studio/common/src/composables/content/types";
import { buildSessionContent } from "@agdb-studio/common/src/composables/content/sessionContent";
import { PhFillCrownSimple } from "@kalimahapps/vue-icons";
import { useUserStore } from "./userStore";
import {
  formatAgent,
  formatTimestamp,
} from "@agdb-studio/utils/src/format/format";

const { getInputValue } = useContentInputs();
const { openModal } = useModal();

const userActions: Action<UserStatus>[] = [
  {
    key: "view_sessions",
    label: "Sessions",
    action: async ({ params }) => {
      const { users, fetchUsers } = useUserStore();
      await fetchUsers();
      const openSessions = (): void => {
        const user = users.value.find((u) => u.username === params.username);
        const sessions = user?.sessions ?? [];
        openModal({
          header: `Sessions for ${params.username}`,
          content: buildSessionContent(sessions, async (rowIndex) => {
            const session = sessions[rowIndex];
            if (!session) return;
            await client.value?.admin_user_logout({
              username: params.username,
              session: session.session,
            });
            await fetchUsers().catch(() => {});
            openSessions();
          }),
          wide: true,
        });
      };
      openSessions();
      return true;
    },
  },
  {
    key: "change_password",
    label: "Change Password",
    action: async ({ params }) => {
      const password = getInputValue<string>(KEY_MODAL, "password")?.toString();
      return client.value
        ?.admin_user_change_password(
          {
            username: params.username,
          },
          { password },
        )
        .then(() => {
          addNotification({
            type: "success",
            title: "Password Changed",
            message: `Password for ${params.username} changed successfully.`,
          });
        });
    },
    confirmation: [
      ...convertArrayOfStringsToContent([
        "Insert the new password for the user.",
      ]),
      {
        input: {
          type: "text",
          label: "Password",
          key: "password",
          required: true,
          autofocus: true,
        },
      },
    ],
    confirmationHeader: ({ params }) =>
      `Change password for ${params.username}`,
  },
  {
    key: "logout",
    label: "Logout",
    action: async ({ params }) => {
      const { users, fetchUsers } = useUserStore();
      await fetchUsers();
      const user = users.value.find((u) => u.username === params.username);
      const sessions = user?.sessions ?? [];

      const content: Content[] = [
        ...convertArrayOfStringsToContent(["Do you want to log out the user?"]),
      ];

      if (sessions.length > 1) {
        content.push({
          input: {
            type: "select",
            label: "Session",
            key: "session",
            options: [
              { value: "", label: "All sessions" },
              ...sessions.map((s) => ({
                value: s.session,
                label: `${formatAgent(s.agent)} — ${formatTimestamp(s.created)}`,
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
        header: `Logout ${params.username}`,
        content,
        onConfirm: async () => {
          const cluster = !!getInputValue<string>(KEY_MODAL, "cluster");
          const sessionId = getInputValue<string>(
            KEY_MODAL,
            "session",
          )?.toString();
          const logoutParams = {
            username: params.username,
            ...(sessionId ? { session: sessionId } : {}),
          };
          const call = cluster
            ? client.value?.cluster_admin_user_logout(logoutParams)
            : client.value?.admin_user_logout(logoutParams);
          await call;
          addNotification({
            type: "success",
            title: "User Logged Out",
            message: sessionId
              ? `Session for ${params.username} logged out successfully${cluster ? " from all nodes in the cluster" : ""}.`
              : `User ${params.username} logged out successfully${cluster ? " from all nodes in the cluster" : ""}.`,
          });
          await fetchUsers().catch(() => {});
          return true;
        },
      });
      return true;
    },
  },
  {
    key: "delete",
    label: "Delete",
    action: async ({ params }) => {
      return client.value
        ?.admin_user_delete({ username: params.username })
        .then(() => {
          addNotification({
            type: "success",
            title: "User Deleted",
            message: `User ${params.username} deleted successfully.`,
          });
        });
    },
    confirmation: [
      ...convertArrayOfStringsToContent(["Do you want to delete the user?"]),
    ],
    confirmationHeader: ({ params }) => `Delete ${params.username}`,
  },
];

const userColumns: Column<TRow>[] = [
  {
    key: "username",
    title: "Username",
    iconResolver: (row: TRow) => (row.admin ? PhFillCrownSimple : undefined),
    iconTitleResolver: (row: TRow) => (row.admin ? "Admin user" : undefined),
    iconClassResolver: (row: TRow) => (row.admin ? "crown-icon" : ""),
  },
  {
    key: "sessionCount",
    title: "Sessions",
  },
  {
    key: "actions",
    title: "Actions",
    actions: userActions as unknown as Action<TRow>[],
  },
];

const logoutAllUsers = (): void => {
  const { fetchUsers } = useUserStore();
  openModal({
    header: "Logout All Users",
    content: [
      ...convertArrayOfStringsToContent(["Do you want to log out all users?"]),
      {
        input: {
          type: "checkbox",
          label: "Logout from all nodes in the cluster",
          key: "cluster",
          value: true,
        },
      },
    ],
    onConfirm: async () => {
      const cluster = !!getInputValue<string>(KEY_MODAL, "cluster");
      const call = cluster
        ? client.value?.cluster_admin_user_logout_all()
        : client.value?.admin_user_logout_all();
      await call;
      addNotification({
        type: "success",
        title: "All Users Logged Out",
        message: `All users have been logged out successfully${cluster ? " from all nodes in the cluster" : ""}.`,
      });
      await fetchUsers().catch(() => {});
      return true;
    },
  });
};

export { userActions, userColumns, logoutAllUsers };
