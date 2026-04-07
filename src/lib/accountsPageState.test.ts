import test from "node:test";
import assert from "node:assert/strict";

import { resolveAccountsPageState, type AccountsPageState } from "./accountsPageState.ts";

function expectState(
  state: ReturnType<typeof resolveAccountsPageState>,
): AccountsPageState {
  if (state === null) {
    throw new Error("expected a non-null accounts page state");
  }

  return state;
}

test("returns platform placeholder for non-codex tabs", () => {
  const state = expectState(resolveAccountsPageState({
    activePlatform: "claude",
    isLoading: false,
    normalizedQuery: "",
    visibleCount: 0,
  }));
  assert.match(state.title, /Claude/);
  assert.ok(state.description.length > 0);
});

test("returns loading copy before accounts are ready", () => {
  const state = expectState(resolveAccountsPageState({
    activePlatform: "codex",
    isLoading: true,
    normalizedQuery: "",
    visibleCount: 0,
  }));
  assert.notEqual(state.title.length, 0);
  assert.notEqual(state.description.length, 0);
});

test("returns search empty-state copy when query has no matches", () => {
  const state = expectState(resolveAccountsPageState({
    activePlatform: "codex",
    isLoading: false,
    normalizedQuery: "murong",
    visibleCount: 0,
  }));
  assert.notEqual(state.title.length, 0);
  assert.notEqual(state.description.length, 0);
});

test("returns default empty-state copy when there are no accounts", () => {
  const state = expectState(resolveAccountsPageState({
    activePlatform: "codex",
    isLoading: false,
    normalizedQuery: "",
    visibleCount: 0,
  }));
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
