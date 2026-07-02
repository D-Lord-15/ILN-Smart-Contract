/**
 * Unit tests for cancel-helpers standalone functions (#233).
 */
import { validatePendingState, formatConfirmMessage } from "../../src/commands/cancel-helpers";
import type { InvoiceSummary } from "../../src/commands/cancel-types";

function inv(state: InvoiceSummary["state"], id = "10"): InvoiceSummary {
  return { id, state, amount: "500", token: "USDC", dueDate: "2026-06-15" };
}

describe("validatePendingState", () => {
  const nonPending: InvoiceSummary["state"][] = ["Funded", "Paid", "Cancelled", "Expired", "Disputed"];

  it("passes silently for Pending invoices", () => {
    expect(() => validatePendingState(inv("Pending"))).not.toThrow();
  });

  for (const state of nonPending) {
    it(`throws for ${state} invoices`, () => {
      expect(() => validatePendingState(inv(state))).toThrow();
    });
  }
});

describe("formatConfirmMessage", () => {
  it("contains the invoice ID", () => {
    expect(formatConfirmMessage(inv("Pending", "88"))).toContain("#88");
  });

  it("contains amount and token", () => {
    expect(formatConfirmMessage(inv("Pending"))).toContain("500 USDC");
  });

  it("contains the due date", () => {
    expect(formatConfirmMessage(inv("Pending"))).toContain("2026-06-15");
  });

  it("contains the [y/N] prompt", () => {
    expect(formatConfirmMessage(inv("Pending"))).toContain("[y/N]");
  });

  it("mentions 'cannot be undone'", () => {
    expect(formatConfirmMessage(inv("Pending"))).toMatch(/cannot be undone/i);
  });
});
