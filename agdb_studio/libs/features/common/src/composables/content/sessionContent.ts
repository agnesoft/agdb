import { CiLogout } from "@kalimahapps/vue-icons";
import type { Content } from "./types";
import { convertArrayOfStringsToContent } from "./utils";
import {
  formatAgent,
  formatTimestamp,
} from "@agdb-studio/utils/src/format/format";

type SessionData = {
  agent: string;
  created: number;
  expires_at: number;
};

export const buildSessionContent = (
  sessions: SessionData[],
  onLogout?: (rowIndex: number) => void,
): Content[] => {
  if (sessions.length === 0) {
    return convertArrayOfStringsToContent(["No active sessions."]);
  }

  return [
    {
      table: {
        headers: ["Agent", "Created", "Expires"],
        rows: sessions.map((s) => [
          formatAgent(s.agent),
          formatTimestamp(s.created),
          formatTimestamp(s.expires_at),
        ]),
        ...(onLogout
          ? {
              rowActions: [
                {
                  label: "Logout",
                  icon: CiLogout,
                  className: "session-logout",
                  onClick: onLogout,
                },
              ],
            }
          : {}),
      },
    },
  ];
};
