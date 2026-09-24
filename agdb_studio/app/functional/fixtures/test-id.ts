export type TestId = string & { readonly __testId: unique symbol };

export const createTestIds = <T extends Record<string, string>>(
  ids: T,
): {
  readonly [K in keyof T]: TestId;
} => {
  return ids as unknown as { readonly [K in keyof T]: TestId };
};
