import { expect, type Locator, test } from "@playwright/test";
import { type TestId } from "./test-id";

type ElementAssertions = {
  isVisible: () => Promise<void>;
  isHidden: () => Promise<void>;
  isChecked: () => Promise<void>;
  isEnabled: () => Promise<void>;
  isDisabled: () => Promise<void>;
  hasText: (text: string) => Promise<void>;
  containsText: (text: string) => Promise<void>;
  hasValue: (value: string) => Promise<void>;
  hasAttribute: (name: string, value: string | RegExp) => Promise<void>;
  hasClass: (className: string | RegExp) => Promise<void>;
  hasStyle: (style: string, value: string) => Promise<void>;
  not: {
    isVisible: () => Promise<void>;
    isHidden: () => Promise<void>;
    isChecked: () => Promise<void>;
    isEnabled: () => Promise<void>;
    isDisabled: () => Promise<void>;
    hasText: (text: string) => Promise<void>;
    containsText: (text: string) => Promise<void>;
    hasValue: (value: string) => Promise<void>;
    hasAttribute: (name: string, value: string | RegExp) => Promise<void>;
    hasClass: (className: string | RegExp) => Promise<void>;
  };
};

export type ElementAPI = Locator & ElementAssertions;

export const createElementAPI = (
  id: TestId,
  getLocator: (id: TestId) => Locator,
): ElementAPI => {
  const locator = getLocator(id);
  const step = <T>(msg: string, fn: () => Promise<T>) => test.step(msg, fn);

  const assertions: ElementAssertions = {
    isVisible: () =>
      step(`Element "${id}" is visible`, () => expect(locator).toBeVisible()),

    isHidden: () =>
      step(`Element "${id}" is hidden`, () => expect(locator).toBeHidden()),

    isChecked: () =>
      step(`Element "${id}" is checked`, () => expect(locator).toBeChecked()),

    isEnabled: () =>
      step(`Element "${id}" is enabled`, () => expect(locator).toBeEnabled()),

    isDisabled: () =>
      step(`Element "${id}" is disabled`, () =>
        expect(locator).toBeDisabled(),
      ),

    hasText: (text: string) =>
      step(`Element "${id}" has text "${text}"`, () =>
        expect(locator).toHaveText(text),
      ),

    containsText: (text: string) =>
      step(`Element "${id}" contains text "${text}"`, () =>
        expect(locator).toContainText(text),
      ),

    hasValue: (value: string) =>
      step(`Element "${id}" has value "${value}"`, () =>
        expect(locator).toHaveValue(value),
      ),

    hasAttribute: (name: string, value: string | RegExp) =>
      step(`Element "${id}" has attribute "${name}"`, () =>
        expect(locator).toHaveAttribute(name, value),
      ),

    hasClass: (className: string | RegExp) =>
      step(`Element "${id}" has class "${className}"`, () =>
        expect(locator).toHaveClass(className),
      ),

    hasStyle: (style: string, value: string) =>
      step(`Element "${id}" has style "${style}: ${value}"`, () =>
        expect(locator).toHaveCSS(style, value),
      ),

    not: {
      isVisible: () =>
        step(`Element "${id}" is not visible`, () =>
          expect(locator).not.toBeVisible(),
        ),

      isHidden: () =>
        step(`Element "${id}" is not hidden`, () =>
          expect(locator).not.toBeHidden(),
        ),

      isChecked: () =>
        step(`Element "${id}" is not checked`, () =>
          expect(locator).not.toBeChecked(),
        ),

      isEnabled: () =>
        step(`Element "${id}" is not enabled`, () =>
          expect(locator).not.toBeEnabled(),
        ),

      isDisabled: () =>
        step(`Element "${id}" is not disabled`, () =>
          expect(locator).not.toBeDisabled(),
        ),

      hasText: (text: string) =>
        step(`Element "${id}" does not have text "${text}"`, () =>
          expect(locator).not.toHaveText(text),
        ),

      containsText: (text: string) =>
        step(`Element "${id}" does not contain text "${text}"`, () =>
          expect(locator).not.toContainText(text),
        ),

      hasValue: (value: string) =>
        step(`Element "${id}" does not have value "${value}"`, () =>
          expect(locator).not.toHaveValue(value),
        ),

      hasAttribute: (name: string, value: string | RegExp) =>
        step(`Element "${id}" does not have attribute "${name}"`, () =>
          expect(locator).not.toHaveAttribute(name, value),
        ),

      hasClass: (className: string | RegExp) =>
        step(`Element "${id}" does not have class "${className}"`, () =>
          expect(locator).not.toHaveClass(className),
        ),
    },
  };

  return Object.assign(locator, assertions);
};
