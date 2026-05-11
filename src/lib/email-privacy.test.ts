import test from "node:test";
import assert from "node:assert/strict";

import { displayEmailAddress, maskEmailAddress } from "./email-privacy.ts";

test("maskEmailAddress masks the middle of the local part and keeps the domain visible", () => {
  assert.equal(maskEmailAddress("murong@example.com"), "mu***g@example.com");
  assert.equal(maskEmailAddress("me@example.com"), "m***@example.com");
  assert.equal(maskEmailAddress("x@example.com"), "***@example.com");
});

test("displayEmailAddress returns the raw email unless email privacy mode is enabled", () => {
  assert.equal(displayEmailAddress("murong@example.com", false), "murong@example.com");
  assert.equal(displayEmailAddress("murong@example.com", true), "mu***g@example.com");
});
