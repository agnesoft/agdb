export const resolveServerUrl = (
  currentUrl: string,
  serverAddress: string,
): string => {
  try {
    const current = new URL(
      currentUrl.includes("://") ? currentUrl : `http://${currentUrl}`,
    );
    const target = new URL(
      serverAddress.includes("://")
        ? serverAddress
        : `${current.protocol}//${serverAddress}`,
    );

    const isLocalhost = ["localhost", "127.0.0.1", "::1"].includes(
      current.hostname.toLowerCase(),
    );

    if (isLocalhost) {
      // Keep localhost host, but use the selected server port.
      current.port = target.port;
      return current.toString().replace(/\/$/, "");
    }

    return target.toString().replace(/\/$/, "");
  } catch {
    return serverAddress;
  }
};
