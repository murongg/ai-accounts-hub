import { execFileSync } from "node:child_process";

export function syncCargoLock() {
  execFileSync("cargo", ["update", "--manifest-path", "src-tauri/Cargo.toml", "-w"], {
    stdio: "inherit",
  });
}

if (process.argv[1] && import.meta.url === new URL(process.argv[1], "file:").href) {
  syncCargoLock();
}
