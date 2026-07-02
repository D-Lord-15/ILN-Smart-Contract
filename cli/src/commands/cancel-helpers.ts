import type { InvoiceSummary } from "./cancel-types.js";

export function validatePendingState(invoice: InvoiceSummary): void {
  if (invoice.state !== "Pending") {
    throw new Error(
      `Invoice #${invoice.id} is in state "${invoice.state}" — only Pending invoices can be cancelled.`
    );
  }
}

export function formatConfirmMessage(invoice: InvoiceSummary): string {
  return `Cancel Invoice #${invoice.id} (${invoice.amount} ${invoice.token}, due ${invoice.dueDate})? This cannot be undone. [y/N]`;
}
