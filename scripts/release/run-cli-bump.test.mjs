import test from "node:test";
import assert from "node:assert/strict";
import { buildCliBumpArgs } from "./run-cli-bump.mjs";

test("CLI bump uses the standalone CLI tag namespace by default", () => {
  const args = buildCliBumpArgs(["patch"]);

  assert.deepEqual(args.slice(0, 4), [
    "node_modules/bumpp/bin/bumpp.mjs",
    "patch",
    "--tag",
    "cli-v%s",
  ]);
});
