import test from "node:test";
import assert from "node:assert/strict";

import { GITHUB_REPOSITORY_URL, openRepositoryHome } from "./external-links.ts";

test("openRepositoryHome opens the repository homepage url", async () => {
  const openedUrls: string[] = [];

  await openRepositoryHome(async (url) => {
    openedUrls.push(url);
  });

  assert.deepEqual(openedUrls, [GITHUB_REPOSITORY_URL]);
});
