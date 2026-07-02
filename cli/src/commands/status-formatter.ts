import type { InvoiceDetail, InvoiceState } from "./status-types.js";

export function stateBadge(state: InvoiceState): string {
  const badges: Record<InvoiceState, string> = {
    Paid: "[PAID]",
    Funded: "[FUNDED]",
    Pending: "[PENDING]",
    Expired: "[EXPIRED]",
    Disputed: "[DISPUTED]",
    Cancelled: "[CANCELLED]",
  };
  return badges[state] ?? `[${state}]`;
}

export function timeUntilExpiry(dueDate: string): string {
  const ms = new Date(dueDate).getTime() - Date.now();
  if (ms <= 0) return "Expired";
  const days = Math.floor(ms / 86_400_000);
  const hours = Math.floor((ms % 86_400_000) / 3_600_000);
  if (days > 0) return `${days}d ${hours}h`;
  const minutes = Math.floor((ms % 3_600_000) / 60_000);
  return `${hours}h ${minutes}m`;
}

export function formatDetail(inv: InvoiceDetail): string {
  const lines: string[] = [
    "",
    `  Invoice ID      ${inv.id}`,
    `  State           ${stateBadge(inv.state)}`,
    `  Submitter       ${inv.submitter}`,
    `  Payer           ${inv.payer}`,
    `  LP              ${inv.lp ?? "—"}`,
    `  Token           ${inv.token}`,
    `  Amount          ${inv.amount} ${inv.token}`,
    `  Discount Rate   ${inv.discountRateBps} bps`,
    `  Effective Yield ${inv.effectiveYieldPct}%`,
    `  Due Date        ${inv.dueDate}`,
    `  Time to Expiry  ${timeUntilExpiry(inv.dueDate)}`,
    "",
  ];
  return lines.join("\n");
}
