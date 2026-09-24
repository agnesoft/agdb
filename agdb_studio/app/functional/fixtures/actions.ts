import { type Locator, test } from "@playwright/test";
import { type TestId } from "./test-id";

export const createActions = (locator: (id: TestId) => Locator) => {
  return {
    click: (testId: TestId) => {
      return test.step(`click ${testId}`, () => locator(testId).click());
    },

    fill: (testId: TestId, value: string) => {
      return test.step(`fill ${testId} with "${value}"`, () =>
        locator(testId).fill(value));
    },

    type: (testId: TestId, value: string, options?: { delay?: number }) => {
      return test.step(`type "${value}" into ${testId}`, () =>
        locator(testId).pressSequentially(value, options));
    },

    press: (testId: TestId, key: string) => {
      return test.step(`press ${key} on ${testId}`, () =>
        locator(testId).press(key));
    },

    select: (testId: TestId, value: string) => {
      return test.step(`select "${value}" on ${testId}`, () =>
        locator(testId).selectOption(value));
    },

    check: (testId: TestId) => {
      return test.step(`check ${testId}`, () => locator(testId).check());
    },

    uncheck: (testId: TestId) => {
      return test.step(`uncheck ${testId}`, () => locator(testId).uncheck());
    },

    hover: (testId: TestId) => {
      return test.step(`hover ${testId}`, () => locator(testId).hover());
    },

    clear: (testId: TestId) => {
      return test.step(`clear ${testId}`, () => locator(testId).clear());
    },

    focus: (testId: TestId) => {
      return test.step(`focus ${testId}`, () => locator(testId).focus());
    },

    blur: (testId: TestId) => {
      return test.step(`blur ${testId}`, () => locator(testId).blur());
    },
  };
};
