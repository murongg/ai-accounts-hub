import test from "node:test";
import assert from "node:assert/strict";

import {
  CLAUDE_ACCOUNT_LOGIN_TIMEOUT_MESSAGE,
  refreshClaudeUsageNow,
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

test("refreshClaudeUsageNow invokes the Claude usage refresh command", async () => {
  const previousWindow = globalThis.window;
  const calls: Array<{ command: string; payload: unknown }> = [];

  (globalThis as { window?: unknown }).window = {
    __TAURI_INTERNALS__: {
      invoke(command: string, payload: unknown) {
        calls.push({ command, payload });
        return Promise.resolve();
      },
    },
  };

  try {
    await refreshClaudeUsageNow();
  } finally {
    (globalThis as { window?: unknown }).window = previousWindow;
  }

  assert.deepEqual(calls, [{ command: "refresh_claude_usage_now", payload: {} }]);
});
