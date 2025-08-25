import { expect, type Locator, type Page } from "@playwright/test";

export const getSelectorByTestId = (testId: string): string => {
  return `[data-testid="${testId}"]`;
};
export const getLocatorByTestId = (page: Page, testId: string): Locator => {
  return page.locator(getSelectorByTestId(testId));
};

export const isVisible = async (
  page: Page,
  elementOrTestId: Locator | string,
) => {
  if (typeof elementOrTestId === "string") {
    elementOrTestId = getLocatorByTestId(page, elementOrTestId);
  }
  await expect(elementOrTestId).toBeVisible();
};
export const isHidden = async (
  page: Page,
  elementOrTestId: Locator | string,
) => {
  if (typeof elementOrTestId === "string") {
    elementOrTestId = getLocatorByTestId(page, elementOrTestId);
  }
  await expect(elementOrTestId).toBeHidden();
};

export const hasText = async (
  page: Page,
  elementOrTestId: Locator | string,
  text: string,
) => {
  if (typeof elementOrTestId === "string") {
    elementOrTestId = getLocatorByTestId(page, elementOrTestId);
  }
  await expect(elementOrTestId).toHaveText(text);
};

export const containsText = async (
  page: Page,
  elementOrTestId: Locator | string,
  text: string,
) => {
  if (typeof elementOrTestId === "string") {
    elementOrTestId = getLocatorByTestId(page, elementOrTestId);
  }
  await expect(elementOrTestId).toContainText(text);
};

export const hasClass = async (
  page: Page,
  elementOrTestId: Locator | string,
  className: string,
) => {
  if (typeof elementOrTestId === "string") {
    elementOrTestId = getLocatorByTestId(page, elementOrTestId);
  }
  await expect(elementOrTestId).toHaveClass(className);
};

export const hasAttribute = async (
  page: Page,
  elementOrTestId: Locator | string,
  attribute: string,
  value: string,
) => {
  if (typeof elementOrTestId === "string") {
    elementOrTestId = getLocatorByTestId(page, elementOrTestId);
  }
  await expect(elementOrTestId).toHaveAttribute(attribute, value);
};
