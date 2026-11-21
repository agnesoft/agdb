export const MIN_REFRESH_SECONDS = 10;
/** A day; longer intervals would overflow the browser timer and fire at once. */
export const MAX_REFRESH_SECONDS = 86_400;

/** Between the minimum and maximum; callers treat 0 (off) on their own. */
export const isRefreshInterval = (seconds: number): boolean =>
  seconds >= MIN_REFRESH_SECONDS && seconds <= MAX_REFRESH_SECONDS;

/** The auto refresh intervals offered in the widget editor; 0 is off. */
export const REFRESH_CHOICES = [0, 10, 30, 60, 300, 900, 3600] as const;

/** A refresh interval for people: "Off", "30 s", "5 min", "1 h", "1 h 30 min". */
export const formatRefresh = (seconds: number): string => {
  if (!seconds) return "Off";
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const rest = seconds % 60;
  const parts = [
    hours ? `${hours} h` : "",
    minutes ? `${minutes} min` : "",
    rest ? `${rest} s` : "",
  ];
  return parts.filter(Boolean).join(" ");
};
