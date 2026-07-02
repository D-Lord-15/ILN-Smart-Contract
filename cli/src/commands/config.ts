/**
 * `iln config` — manage CLI settings.
 *
 * Sub-commands:
 *   iln config set <key> <value>
 *   iln config get <key>
 *   iln config list
 *   iln config reset
 *
 * Issue: #245
 */
import { Command } from "commander";
import {
  getConfigValue,
  loadConfig,
  resetConfig,
  setConfigValue,
} from "../config.js";

export function makeConfigCommand(): Command {
  const cmd = new Command("config").description(
    "Manage ILN CLI configuration stored in ~/.iln/config.json"
  );

  // iln config set <key> <value>
  cmd
    .command("set <key> <value>")
    .description(
      "Set a config value (keys: network, rpcUrl, defaultProfile)"
    )
    .action((key: string, value: string) => {
      try {
        setConfigValue(key, value);
        console.log(`✓ Set ${key} = ${value}`);
      } catch (err) {
        console.error(`Error: ${(err as Error).message}`);
        process.exit(1);
      }
    });

  // iln config get <key>
  cmd
    .command("get <key>")
    .description("Get a single config value")
    .action((key: string) => {
      const val = getConfigValue(key as "network" | "rpcUrl" | "defaultProfile");
      if (val === undefined) {
        console.error(`Key "${key}" not found in config.`);
        process.exit(1);
      }
      console.log(val);
    });

  // iln config list
  cmd
    .command("list")
    .description("Show all current config values")
    .option("--json", "Output as JSON")
    .action((opts: { json?: boolean }) => {
      const cfg = loadConfig();
      if (opts.json) {
        console.log(JSON.stringify(cfg, null, 2));
      } else {
        for (const [k, v] of Object.entries(cfg)) {
          console.log(`${k}: ${v}`);
        }
      }
    });

  // iln config reset
  cmd
    .command("reset")
    .description("Restore all config values to defaults")
    .action(() => {
      resetConfig();
      console.log("✓ Config reset to defaults.");
    });

  return cmd;
}
