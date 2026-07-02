/**
 * `iln status --id X` — display a rich, human-readable invoice summary.
 *
 * Flags:
 *   --json    Output raw JSON for piping
 *   --watch   Refresh every 10 seconds until a terminal state is reached
 *
 * Issue: #231
 */
import { Command } from "commander";
import type { InvoiceDetail } from "./status-types.js";
import { TERMINAL_STATES } from "./status-types.js";
import { formatDetail } from "./status-formatter.js";
import { buildTimeline } from "./status-timeline.js";

export type InvoiceFetcher = (id: string) => Promise<InvoiceDetail>;

async function defaultFetcher(id: string): Promise<InvoiceDetail> {
  return {
    id,
    state: "Pending",
    submitter: "GSUBMITTER000000000000000000000000000000000000000000000",
    payer: "GPAYER000000000000000000000000000000000000000000000000000",
    token: "USDC",
    amount: "100",
    discountRateBps: 300,
    effectiveYieldPct: "3.00",
    dueDate: new Date(Date.now() + 7 * 86_400_000).toISOString().slice(0, 10),
    createdAt: new Date().toISOString(),
  };
}

export function makeStatusCommand(
  fetchInvoice: InvoiceFetcher = defaultFetcher,
  setIntervalFn: typeof setInterval = setInterval,
  clearIntervalFn: typeof clearInterval = clearInterval
): Command {
  const cmd = new Command("status").description(
    "Display a rich summary of an invoice"
  );

  cmd
    .requiredOption("--id <invoice-id>", "Invoice ID")
    .option("--json", "Output raw JSON")
    .option("--watch", "Refresh every 10 seconds until terminal state")
    .action(async (opts: { id: string; json?: boolean; watch?: boolean }) => {
      async function printStatus(): Promise<boolean> {
        try {
          const inv = await fetchInvoice(opts.id);
          if (opts.json) {
            console.log(JSON.stringify(inv, null, 2));
          } else {
            console.log(formatDetail(inv));
            console.log(buildTimeline(inv.state));
          }
          return TERMINAL_STATES.includes(inv.state);
        } catch (err) {
          console.error(`Status error: ${(err as Error).message}`);
          process.exit(1);
          return true;
        }
      }

      const done = await printStatus();
      if (!opts.watch || done) return;

      const timer = setIntervalFn(async () => {
        const finished = await printStatus();
        if (finished) clearIntervalFn(timer);
      }, 10_000);
    });

  return cmd;
}
