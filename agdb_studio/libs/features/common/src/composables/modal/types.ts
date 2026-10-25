import type { Content } from "../content/types";

export type Modal = {
  header: string;
  content: Content[];
  wide?: boolean;
};

export type Button = {
  className: string;
  text: string;
  action: () => void | Promise<void>;
  type?: "button" | "submit" | "reset";
};
