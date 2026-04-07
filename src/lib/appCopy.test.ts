import test from "node:test";
import assert from "node:assert/strict";

import { getAppCopy, resolveDaisyTheme } from "./appCopy.ts";

test("returns Chinese settings copy by default", () => {
  const copy = getAppCopy("zh-CN");

  assert.equal(copy.settings.language.options[0]?.value, "zh-CN");
  assert.equal(copy.settings.sync.options.length > 0, true);
  assert.equal(copy.header.searchPlaceholder.length > 0, true);
});

test("returns English copy for the settings experience", () => {
  const copy = getAppCopy("en-US");

  assert.equal(copy.settings.language.options[1]?.value, "en-US");
  assert.equal(copy.settings.sync.options.length > 0, true);
  assert.equal(copy.accounts.addAccount.length > 0, true);
});

test("maps app theme selections to default daisyUI themes", () => {
  assert.equal(resolveDaisyTheme("light", false), "bumblebee");
  assert.equal(resolveDaisyTheme("dark", false), "luxury");
  assert.equal(resolveDaisyTheme("system", false), "bumblebee");
  assert.equal(resolveDaisyTheme("system", true), "luxury");
});

test("includes a follow-system theme label in both locales", () => {
  assert.equal(getAppCopy("zh-CN").settings.theme.system.length > 0, true);
  assert.equal(getAppCopy("en-US").settings.theme.system.length > 0, true);
  assert.notEqual(getAppCopy("zh-CN").settings.theme.system, getAppCopy("en-US").settings.theme.system);
});

test("includes updater copy in both locales", () => {
  assert.equal(getAppCopy("zh-CN").settings.update.title.length > 0, true);
  assert.equal(getAppCopy("zh-CN").settings.update.check.length > 0, true);
  assert.equal(getAppCopy("en-US").settings.update.title.length > 0, true);
  assert.equal(getAppCopy("en-US").settings.update.install.length > 0, true);
});
