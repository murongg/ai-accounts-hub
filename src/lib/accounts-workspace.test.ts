import test from "node:test";
import assert from "node:assert/strict";

import {
  createLatestRequestGate,
  resolveAccountsPageState,
  type AccountsPageState,
} from "./accounts-workspace.ts";

function expectState(
  state: ReturnType<typeof resolveAccountsPageState>,
): AccountsPageState {
  if (state === null) {
    throw new Error("expected a non-null accounts page state");
  }

  return state;
}

function createDeferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });

  return { promise, resolve, reject };
}

test("returns the default empty-state copy for Claude when no managed accounts exist", () => {
  const state = expectState(resolveAccountsPageState({
    activePlatform: "claude",
    isLoading: false,
    normalizedQuery: "",
    visibleCount: 0,
  }));
  assert.doesNotMatch(state.title, /Claude 即将接入|Claude is coming soon/);
  assert.notEqual(state.title.length, 0);
});

test("returns the default empty-state copy for Gemini when no managed accounts exist", () => {
  const state = expectState(resolveAccountsPageState({
    activePlatform: "gemini",
    isLoading: false,
    normalizedQuery: "",
    visibleCount: 0,
  }));
  assert.doesNotMatch(state.title, /Gemini 即将接入|Gemini is coming soon/);
  assert.notEqual(state.title.length, 0);
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

test("latest request wins when newer success resolves first", async () => {
  const gate = createLatestRequestGate<number>();
  const applied: number[] = [];
  const firstDeferred = createDeferred<number>();

  const first = gate.run(
    () => firstDeferred.promise,
    (value) => applied.push(value),
  );

  const second = gate.run(async () => 2, (value) => applied.push(value));

  firstDeferred.resolve(1);
  await Promise.all([first, second]);

  assert.deepEqual(applied, [2]);
});

test("stale request errors do not surface after a newer request starts", async () => {
  const gate = createLatestRequestGate<number>();
  const errors: string[] = [];
  const firstDeferred = createDeferred<number>();

  const first = gate.run(
    () => firstDeferred.promise,
    () => {},
    (error) => errors.push(String(error)),
  );

  const second = gate.run(async () => 2, () => {});

  firstDeferred.reject(new Error("stale failure"));
  await Promise.allSettled([first, second]);

  assert.deepEqual(errors, []);
});
