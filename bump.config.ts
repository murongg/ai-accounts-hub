import path from "node:path";
import { defineConfig } from "bumpp";
import { syncCargoLock } from "./scripts/release/sync-cargo-lock.mjs";

const cargoLockPath = "src-tauri/Cargo.lock";

export default defineConfig({
  async execute(operation) {
    syncCargoLock();

    const absoluteCargoLockPath = path.resolve(operation.options.cwd, cargoLockPath);

    if (!operation.state.updatedFiles.includes(absoluteCargoLockPath)) {
      operation.update({
        updatedFiles: [...operation.state.updatedFiles, absoluteCargoLockPath],
      });
    }
  },
});
