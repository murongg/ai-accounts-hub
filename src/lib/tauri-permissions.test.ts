import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

test("allows opener.openPath for the desktop capability", () => {
  const capability = JSON.parse(readFileSync("src-tauri/capabilities/default.json", "utf8")) as {
    permissions: Array<
      | string
      | {
          identifier: string;
          allow?: Array<{ path?: string }>;
        }
    >;
  };

  const openerScope = capability.permissions.find(
    (permission) => typeof permission === "object" && permission.identifier === "opener:allow-open-path",
  );

  assert.ok(openerScope && typeof openerScope === "object");

  const scopedPermission = openerScope as {
    identifier: string;
    allow?: Array<{ path?: string }>;
  };

  assert.deepEqual(scopedPermission.allow, [{ path: "$APPDATA/codex" }, { path: "$APPDATA/codex/**" }]);
});
