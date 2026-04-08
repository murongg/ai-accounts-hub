import { execFileSync } from "node:child_process";

execFileSync("cargo", ["update", "--manifest-path", "src-tauri/Cargo.toml", "-w"], {
  stdio: "inherit",
});
