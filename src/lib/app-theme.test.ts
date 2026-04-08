import test from "node:test";
import assert from "node:assert/strict";

import { resolveDaisyTheme } from "./app-theme.ts";

test("maps app theme selections to daisyUI themes", () => {
  assert.equal(resolveDaisyTheme("light", false), "bumblebee");
  assert.equal(resolveDaisyTheme("dark", false), "luxury");
  assert.equal(resolveDaisyTheme("system", false), "bumblebee");
  assert.equal(resolveDaisyTheme("system", true), "luxury");
});
