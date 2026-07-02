import type { InvoiceState } from "./status-types.js";

const STATE_ORDER: InvoiceState[] = ["Pending", "Funded", "Paid"];

export function buildTimeline(currentState: InvoiceState): string {
  const steps = STATE_ORDER.map((s) => {
    if (s === currentState) return `[ ${s} ]`;
    const idx = STATE_ORDER.indexOf(s);
    const currentIdx = STATE_ORDER.indexOf(currentState);
    if (currentIdx === -1) return `  ${s}  `;
    return idx < currentIdx ? `  ${s}  ` : `  ${s}  `;
  });

  const filled = steps.map((s, i) => {
    const idx = STATE_ORDER.indexOf(STATE_ORDER[i]);
    const currentIdx = STATE_ORDER.indexOf(currentState);
    if (currentIdx === -1) return `○ ${STATE_ORDER[i]}`;
    if (idx < currentIdx) return `● ${STATE_ORDER[i]}`;
    if (idx === currentIdx) return `◉ ${STATE_ORDER[i]}`;
    return `○ ${STATE_ORDER[i]}`;
  });

  return "\n  Timeline:  " + filled.join("  →  ") + "\n";
}
