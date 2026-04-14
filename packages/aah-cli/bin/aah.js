#!/usr/bin/env node
import { spawn } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const binary =
  process.platform === "win32"
    ? path.join(packageRoot, "vendor", "aah.exe")
    : path.join(packageRoot, "vendor", "aah");

const child = spawn(binary, process.argv.slice(2), { stdio: "inherit" });
child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
  }
  process.exit(code ?? 1);
});
