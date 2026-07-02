/**
 * Shared helpers for CLI E2E tests (#247).
 * Spawns the CLI via ts-node so no build step is required in tests.
 */
import { execSync } from "child_process";
import fs from "fs";
import os from "os";
import path from "path";

const CLI_ENTRY = path.resolve(__dirname, "../../src/index.ts");

export interface ExecResult {
  stdout: string;
  stderr: string;
  exitCode: number;
}

/** Run the CLI with ts-node and capture output + exit code. */
export function runCLI(args: string, env?: Record<string, string>): ExecResult {
  try {
    const stdout = execSync(
      `npx ts-node --skip-project ${CLI_ENTRY} ${args}`,
      {
        encoding: "utf-8",
        env: { ...process.env, ...env },
        stdio: ["pipe", "pipe", "pipe"],
      }
    );
    return { stdout, stderr: "", exitCode: 0 };
  } catch (err: any) {
    return {
      stdout: err.stdout ?? "",
      stderr: err.stderr ?? "",
      exitCode: err.status ?? 1,
    };
  }
}

/** Create a temp directory and point ILN_DIR at it for test isolation. */
export function makeTempHome(): { dir: string; cleanup: () => void } {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "iln-test-"));
  return {
    dir,
    cleanup: () => fs.rmSync(dir, { recursive: true, force: true }),
  };
}
