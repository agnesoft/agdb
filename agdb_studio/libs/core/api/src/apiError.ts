import { AgdbApiError } from "@agnesoft/agdb_api";

/** The text of an error: for a failed request the description the server sent. */
export const errorMessage = (error: unknown): string => {
  if (error instanceof AgdbApiError) {
    return error.description || error.to_string();
  }
  return error instanceof Error ? error.message : String(error);
};
