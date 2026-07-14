import { mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const output = resolve(root, "snapshots", "rustdoc.json");
mkdirSync(dirname(output), { recursive: true });

const result = spawnSync(
  "cargo",
  [
    "run",
    "--release",
    "--quiet",
    "--manifest-path",
    resolve(root, "node_modules", "sourcey", "dist", "core", "sourcey-rustdoc", "Cargo.toml"),
    "--",
    "--manifest",
    resolve(root, "adapter", "Cargo.toml"),
    "--toolchain",
    "nightly-2025-11-22",
    "--output",
    output
  ],
  { cwd: root, stdio: "inherit" }
);

process.exit(result.status ?? 1);
