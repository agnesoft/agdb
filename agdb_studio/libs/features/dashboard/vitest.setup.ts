import { afterEach, vi } from "vitest";

// Admin mode needs the app router; specs switch it on through `useAdmin()`.
vi.mock("@agdb-studio/router/src/admin", async () => {
  const { ref } = await import("vue");
  const isAdmin = ref(false);
  const isAdminView = ref(false);
  return { useAdmin: () => ({ isAdmin, isAdminView }) };
});

afterEach(async () => {
  const { useAdmin } = await import("@agdb-studio/router/src/admin");
  const { isAdmin, isAdminView } = useAdmin();
  // The mock's refs are writable; the real ones are computed.
  (isAdmin as { value: boolean }).value = false;
  (isAdminView as { value: boolean }).value = false;
});
