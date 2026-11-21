export const MIN_REFRESH_SECONDS = 10;
/** A day; longer intervals would overflow the browser timer and fire at once. */
export const MAX_REFRESH_SECONDS = 86_400;

/** Between the minimum and maximum; callers treat 0 (off) on their own. */
export const isRefreshInterval = (seconds: number): boolean =>
  seconds >= MIN_REFRESH_SECONDS && seconds <= MAX_REFRESH_SECONDS;
