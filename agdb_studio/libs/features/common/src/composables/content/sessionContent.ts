import type { Content } from "./types";
import { convertArrayOfStringsToContent } from "./utils";
import { formatTimestamp } from "@agdb-studio/utils/src/format/format";

type SessionData = {
  agent: string;
  created: number;
  expires_at: number;
};

export const buildSessionContent = (sessions: SessionData[]): Content[] => {
  if (sessions.length === 0) {
    return convertArrayOfStringsToContent(["No active sessions."]);
  }

  return [
    {
      table: {
        headers: ["Agent", "Created", "Expires"],
        rows: sessions.map((s) => [
          s.agent || "(none)",
          formatTimestamp(s.created),
          formatTimestamp(s.expires_at),
        ]),
      },
    },
  ];
};
