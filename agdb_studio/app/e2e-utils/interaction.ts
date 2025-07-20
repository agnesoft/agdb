import { Page } from "@playwright/test";

export const getTestIdSelector = (testId: string): string => {
  return `[data-testid="${testId}"]`;
};

export const click = async (page: Page, testId: string): Promise<void> => {
  const selector = getTestIdSelector(testId);
  await page.click(selector);
};

export const fillInput = async (
  page: Page,
  testId: string,
  value: string,
): Promise<void> => {
  const selector = getTestIdSelector(testId);
  await page.fill(selector, value);
};
