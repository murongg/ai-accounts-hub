import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const componentPath = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  "codex-autofill-login-modal.tsx",
);
const source = readFileSync(componentPath, "utf8");

test("stacks field labels above the Codex autofill inputs", () => {
  assert.ok(!source.includes('className="form-control"'));
  assert.ok(source.includes('htmlFor="codex-autofill-email"'));
  assert.ok(source.includes('htmlFor="codex-autofill-password"'));
  assert.ok(source.includes("input input-bordered h-11 w-full rounded-lg"));
});
