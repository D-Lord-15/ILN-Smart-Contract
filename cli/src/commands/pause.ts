import { Command } from "commander";
import * as readline from "readline";
import { resolveProfile } from "../config.js";

export interface PauseResult {
  txHash: string;
  paused: boolean;
}

export type PauseExecutor = () => Promise<PauseResult>;
export type StateChecker = () => Promise<boolean>;

async function promptConfirm(message: string): Promise<boolean> {
  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  return new Promise((resolve) => {
    rl.question(`${message} `, (answer) => {
      rl.close();
      resolve(answer.trim().toLowerCase() === "y");
    });
  });
}

// Default mock executors
async function defaultPauseExecutor(): Promise<PauseResult> {
  return {
    txHash: `TX${Math.random().toString(36).slice(2).toUpperCase()}`,
    paused: true,
  };
}

async function defaultUnpauseExecutor(): Promise<PauseResult> {
  return {
    txHash: `TX${Math.random().toString(36).slice(2).toUpperCase()}`,
    paused: false,
  };
}

let defaultState = false;
async function defaultStateChecker(): Promise<boolean> {
  return defaultState;
}

export function makePauseCommand(
  stateChecker: StateChecker = defaultStateChecker,
  pauseExecutor: PauseExecutor = defaultPauseExecutor,
  confirm: (msg: string) => Promise<boolean> = promptConfirm
): Command {
  const cmd = new Command("pause").description("Pause all contract operations");

  cmd
    .option("--yes", "Skip confirmation prompt")
    .action(async (opts: { yes?: boolean }) => {
      try {
        // Require admin authentication
        const parentOpts = cmd.parent?.opts() as { profile?: string } | undefined;
        const profile = resolveProfile(parentOpts?.profile);
        if (!profile) {
          console.error("Error: No connected wallet found. Run: iln wallet generate");
          process.exit(1);
          return;
        }

        // Check current state
        const isCurrentlyPaused = await stateChecker();
        if (isCurrentlyPaused) {
          console.log("Contract is already paused. No changes made.");
          return;
        }

        // Confirmation prompt
        if (!opts.yes) {
          const msg = "Confirm pause of contract? [y/N]";
          const confirmed = await confirm(msg);
          if (!confirmed) {
            console.log("Aborted — contract not paused.");
            return;
          }
        }

        const result = await pauseExecutor();
        // Update defaultState if using defaultStateChecker
        defaultState = true;

        console.log(`Contract paused. TX: ${result.txHash}`);
        console.log(`Contract State: Paused`);
      } catch (err) {
        console.error(`Error: ${(err as Error).message}`);
        process.exit(1);
      }
    });

  return cmd;
}

export function makeUnpauseCommand(
  stateChecker: StateChecker = defaultStateChecker,
  unpauseExecutor: PauseExecutor = defaultUnpauseExecutor,
  confirm: (msg: string) => Promise<boolean> = promptConfirm
): Command {
  const cmd = new Command("unpause").description("Unpause contract operations");

  cmd
    .option("--yes", "Skip confirmation prompt")
    .action(async (opts: { yes?: boolean }) => {
      try {
        // Require admin authentication
        const parentOpts = cmd.parent?.opts() as { profile?: string } | undefined;
        const profile = resolveProfile(parentOpts?.profile);
        if (!profile) {
          console.error("Error: No connected wallet found. Run: iln wallet generate");
          process.exit(1);
          return;
        }

        // Check current state
        const isCurrentlyPaused = await stateChecker();
        if (!isCurrentlyPaused) {
          console.log("Contract is already unpaused. No changes made.");
          return;
        }

        // Confirmation prompt
        if (!opts.yes) {
          const msg = "Confirm unpause of contract? [y/N]";
          const confirmed = await confirm(msg);
          if (!confirmed) {
            console.log("Aborted — contract not unpaused.");
            return;
          }
        }

        const result = await unpauseExecutor();
        // Update defaultState if using defaultStateChecker
        defaultState = false;

        console.log(`Contract unpaused. TX: ${result.txHash}`);
        console.log(`Contract State: Active`);
      } catch (err) {
        console.error(`Error: ${(err as Error).message}`);
        process.exit(1);
      }
    });

  return cmd;
}
