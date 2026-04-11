import test from "node:test";
import assert from "node:assert/strict";

import { getI18n } from "./i18n.ts";

test("header copy includes a GitHub aria label in both languages", () => {
  const chinese = getI18n("zh-CN");
  const english = getI18n("en-US");

  assert.equal(chinese.header.openGithubAria, "打开 GitHub 仓库");
  assert.equal(english.header.openGithubAria, "Open GitHub repository");
});
