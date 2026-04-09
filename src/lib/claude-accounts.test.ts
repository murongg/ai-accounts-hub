import test from "node:test";
import assert from "node:assert/strict";

import {
  CLAUDE_ACCOUNT_LOGIN_TIMEOUT_MESSAGE,
  withTimeout,
} from "./claude-accounts.ts";

test("withTimeout resolves when the operation finishes before the deadline", async () => {
  const value = await withTimeout(Promise.resolve("ok"), 50, "timeout");
  assert.equal(value, "ok");
});

test("withTimeout rejects with the provided timeout message", async () => {
  const neverSettles = new Promise<never>(() => {});

  await assert.rejects(
    () => withTimeout(neverSettles, 10, CLAUDE_ACCOUNT_LOGIN_TIMEOUT_MESSAGE),
    (error: unknown) => {
      assert.ok(error instanceof Error);
      assert.equal(error.message, CLAUDE_ACCOUNT_LOGIN_TIMEOUT_MESSAGE);
      return true;
    },
  );
});
