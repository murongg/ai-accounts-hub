import test from "node:test";
import assert from "node:assert/strict";

import { selectLatestAppRelease, selectLatestAppReleaseTag } from "./releases.ts";

test("selectLatestAppReleaseTag skips CLI releases", () => {
  const tag = selectLatestAppReleaseTag([
    { tag_name: "cli-v0.1.3" },
    { tag_name: "v0.3.19" },
    { tag_name: "v0.3.18" },
  ]);

  assert.equal(tag, "v0.3.19");
});

test("selectLatestAppReleaseTag ignores drafts and prereleases", () => {
  const tag = selectLatestAppReleaseTag([
    { tag_name: "v0.4.0-beta.1", prerelease: true },
    { tag_name: "v0.3.20", draft: true },
    { tag_name: "v0.3.19" },
  ]);

  assert.equal(tag, "v0.3.19");
});

test("selectLatestAppRelease returns the matching app release object", () => {
  const release = selectLatestAppRelease([
    { tag_name: "cli-v0.1.8", assets: [{ name: "cli" }] },
    { tag_name: "v0.3.20", assets: [{ name: "app" }] },
  ]);

  assert.deepEqual(release, {
    tag_name: "v0.3.20",
    assets: [{ name: "app" }],
  });
});
