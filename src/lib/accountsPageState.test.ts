import test from "node:test";
import assert from "node:assert/strict";

import { resolveAccountsPageState } from "./accountsPageState.ts";

test("returns platform placeholder for non-codex tabs", () => {
  const state = resolveAccountsPageState({
    activePlatform: "claude",
    isLoading: false,
    normalizedQuery: "",
    visibleCount: 0,
  });

  assert.ok(state);
  assert.match(state.title, /Claude/);
  assert.ok(state.description.length > 0);
});

test("returns loading copy before accounts are ready", () => {
  const state = resolveAccountsPageState({
    activePlatform: "codex",
    isLoading: true,
    normalizedQuery: "",
    visibleCount: 0,
  });

  assert.ok(state);
  assert.notEqual(state.title.length, 0);
  assert.notEqual(state.description.length, 0);
});

test("returns search empty-state copy when query has no matches", () => {
  const state = resolveAccountsPageState({
    activePlatform: "codex",
    isLoading: false,
    normalizedQuery: "murong",
    visibleCount: 0,
  });

  assert.ok(state);
  assert.notEqual(state.title.length, 0);
  assert.notEqual(state.description.length, 0);
});

test("returns default empty-state copy when there are no accounts", () => {
  const state = resolveAccountsPageState({
    activePlatform: "codex",
    isLoading: false,
    normalizedQuery: "",
    visibleCount: 0,
  });

  assert.ok(state);
  assert.notEqual(state.title.length, 0);
  assert.notEqual(state.description.length, 0);
});

test("returns null when cards should be shown", () => {
  assert.equal(
    resolveAccountsPageState({
      activePlatform: "codex",
      isLoading: false,
      normalizedQuery: "",
      visibleCount: 2,
    }),
    null,
  );
});
