import { spawnSync } from "node:child_process";

export function buildCliBumpArgs(args) {
  return [
    "node_modules/bumpp/bin/bumpp.mjs",
    ...args,
    "--tag",
    "cli-v%s",
    "crates/aah-cli/Cargo.toml",
    "packages/aah-cli/package.json",
    "--configFilePath",
    "bump.config.ts",
  ];
}

function main() {
  const result = spawnSync("node", buildCliBumpArgs(process.argv.slice(2)), {
    stdio: "inherit",
  });

  if (typeof result.status === "number") {
    process.exit(result.status);
  }

  process.exit(1);
}

if (process.argv[1] && import.meta.url === new URL(process.argv[1], "file:").href) {
  main();
}
