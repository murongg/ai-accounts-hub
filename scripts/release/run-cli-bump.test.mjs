import test from "node:test";
import assert from "node:assert/strict";
import { buildCliBumpArgs } from "./run-cli-bump.mjs";

test("CLI bump uses the standalone CLI tag namespace by default", () => {
  const args = buildCliBumpArgs(["patch"]);

  assert.deepEqual(args.slice(0, 6), [
    "node_modules/bumpp/bin/bumpp.mjs",
    "patch",
    "--commit",
    "chore: release cli-v%s",
    "--tag",
    "cli-v%s",
  ]);
});
