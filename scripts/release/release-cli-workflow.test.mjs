import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";

const workflow = fs.readFileSync(
  new URL("../../.github/workflows/release-cli.yml", import.meta.url),
  "utf8",
);
const ciWorkflow = fs.readFileSync(
  new URL("../../.github/workflows/ci.yml", import.meta.url),
  "utf8",
);

function jobBlock(jobName) {
  const match = workflow.match(
    new RegExp(`\\n  ${jobName}:\\n([\\s\\S]*?)(?=\\n  [a-zA-Z0-9_]+:\\n|\\n?$)`),
  );
  assert.ok(match, `${jobName} job is present`);
  return match[1];
}

test("CLI npm publish job configures npmjs registry authentication", () => {
  const publishJob = jobBlock("publish_cli");
  const setupNode = publishJob.match(
    /- name: Setup Node\.js\n([\s\S]*?)(?=\n      - name:)/,
  );

  assert.ok(setupNode, "publish job has a Setup Node.js step");
  assert.match(setupNode[1], /registry-url:\s*https:\/\/registry\.npmjs\.org/);
  assert.match(setupNode[1], /scope:\s*"@murongg"/);
});

test("CLI releases never become the repository latest release", () => {
  const publishJob = jobBlock("publish_cli");

  assert.match(publishJob, /make_latest:\s*false/);
});

test("ordinary CI does not run for every push", () => {
  assert.doesNotMatch(ciWorkflow, /\n\s+push:/);
  assert.match(ciWorkflow, /\n\s+pull_request:/);
  assert.match(ciWorkflow, /types:\s*\[opened,\s*reopened,\s*ready_for_review\]/);
  assert.doesNotMatch(ciWorkflow, /\bsynchronize\b/);
  assert.match(ciWorkflow, /\n\s+workflow_dispatch:/);
});
