import test from "node:test";
import assert from "node:assert/strict";

import {
  canShowCodexAutofillLogin,
  normalizeCodexAutofillLoginInput,
} from "./codex-autofill-login.ts";

test("shows codex autofill login only for the codex platform", () => {
  assert.equal(canShowCodexAutofillLogin("codex"), true);
  assert.equal(canShowCodexAutofillLogin("claude"), false);
  assert.equal(canShowCodexAutofillLogin("gemini"), false);
  assert.equal(canShowCodexAutofillLogin("unknown"), false);
});

test("normalizes email without trimming password", () => {
  assert.deepEqual(
    normalizeCodexAutofillLoginInput({
      email: "  USER@Example.COM  ",
      password: "  secret password  ",
    }),
    {
      email: "USER@Example.COM",
      password: "  secret password  ",
    },
  );
});
