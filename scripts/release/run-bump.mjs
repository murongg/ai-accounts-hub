import { spawnSync } from "node:child_process";

const commandArgs = [
  "node_modules/bumpp/bin/bumpp.mjs",
  ...process.argv.slice(2),
  "package.json",
  "src-tauri/tauri.conf.json",
  "src-tauri/Cargo.toml",
  "--configFilePath",
  "bump.config.ts",
];

const result = spawnSync("node", commandArgs, {
  stdio: "inherit",
});

if (typeof result.status === "number") {
  process.exit(result.status);
}

process.exit(1);
