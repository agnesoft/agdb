const format = (level: string, message: string, prefix?: string) =>
  `${prefix ? `[${prefix}] ` : ""}[${level}] ${message}`;

export const loggerInfo = (msg: string, prefix?: string) =>
  console.log(format("INFO", msg, prefix));
export const loggerWarn = (msg: string, prefix?: string) =>
  console.warn(format("WARN", msg, prefix));
export const loggerError = (msg: string, prefix?: string) =>
  console.error(format("ERROR", msg, prefix));
export const loggerDebug = (msg: string, prefix?: string) =>
  console.debug(format("DEBUG", msg, prefix));

type Message = string | number | boolean | null | undefined;

export const createLogger = (prefix = "") => {
  return {
    info: (...msg: Message[]) => loggerInfo(msg.join(" "), prefix),
    warn: (...msg: Message[]) => loggerWarn(msg.join(" "), prefix),
    error: (...msg: Message[]) => loggerError(msg.join(" "), prefix),
    debug: (...msg: Message[]) => loggerDebug(msg.join(" "), prefix),
  };
};
