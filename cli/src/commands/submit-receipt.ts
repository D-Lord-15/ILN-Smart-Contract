import type { SubmitResult } from "./submit-types.js";

export function buildReceiptRows(result: SubmitResult): Array<[string, string]> {
  const rows: Array<[string, string]> = [
    ["Invoice ID", result.invoiceId],
    ["TX Hash", result.txHash],
    ["Payer", result.payer],
    ["Amount", `${result.amount} ${result.token}`],
    ["Discount Rate", `${result.rateBps} bps (${result.yieldPct}%)`],
    ["Due Date", result.dueDate],
  ];
  if (result.referral) rows.push(["Referral", result.referral]);
  return rows;
}

export function printReceiptTable(result: SubmitResult): void {
  const rows = buildReceiptRows(result);
  const labelWidth = Math.max(...rows.map(([l]) => l.length)) + 2;
  console.log("\n┌" + "─".repeat(labelWidth + 32) + "┐");
  for (const [label, value] of rows) {
    console.log(`│ ${label.padEnd(labelWidth)} ${value}`);
  }
  console.log("└" + "─".repeat(labelWidth + 32) + "┘");
}

export function bpsToYieldPct(rateBps: number): string {
  return (rateBps / 100).toFixed(2);
}
