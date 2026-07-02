/**
 * Unit tests for submit-receipt helpers (#229).
 */
import { buildReceiptRows, bpsToYieldPct } from "../../src/commands/submit-receipt";
import type { SubmitResult } from "../../src/commands/submit-types";

function makeResult(overrides: Partial<SubmitResult> = {}): SubmitResult {
  return {
    invoiceId: "INV-001",
    txHash: "TXABC",
    payer: "GPAY000",
    amount: "100",
    token: "USDC",
    rateBps: 300,
    yieldPct: "3.00",
    dueDate: "2025-12-31",
    ...overrides,
  };
}

describe("bpsToYieldPct", () => {
  it("converts 300 bps to 3.00%", () => expect(bpsToYieldPct(300)).toBe("3.00"));
  it("converts 0 bps to 0.00%", () => expect(bpsToYieldPct(0)).toBe("0.00"));
  it("converts 10000 bps to 100.00%", () => expect(bpsToYieldPct(10000)).toBe("100.00"));
  it("converts 150 bps to 1.50%", () => expect(bpsToYieldPct(150)).toBe("1.50"));
});

describe("buildReceiptRows", () => {
  it("always includes Invoice ID row", () => {
    const rows = buildReceiptRows(makeResult());
    expect(rows.some(([l]) => l === "Invoice ID")).toBe(true);
  });

  it("includes TX Hash row", () => {
    const rows = buildReceiptRows(makeResult());
    expect(rows.some(([l, v]) => l === "TX Hash" && v === "TXABC")).toBe(true);
  });

  it("includes Amount with token", () => {
    const rows = buildReceiptRows(makeResult({ amount: "250", token: "EURC" }));
    expect(rows.some(([l, v]) => l === "Amount" && v.includes("250 EURC"))).toBe(true);
  });

  it("omits Referral row when not set", () => {
    const rows = buildReceiptRows(makeResult({ referral: undefined }));
    expect(rows.some(([l]) => l === "Referral")).toBe(false);
  });

  it("includes Referral row when set", () => {
    const rows = buildReceiptRows(makeResult({ referral: "REF42" }));
    expect(rows.some(([l, v]) => l === "Referral" && v === "REF42")).toBe(true);
  });
});
