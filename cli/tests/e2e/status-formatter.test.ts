/**
 * Unit tests for status-formatter helpers (#231).
 */
import { stateBadge, timeUntilExpiry, formatDetail } from "../../src/commands/status-formatter";
import { buildTimeline } from "../../src/commands/status-timeline";
import type { InvoiceDetail } from "../../src/commands/status-types";

function mockInvoice(overrides: Partial<InvoiceDetail> = {}): InvoiceDetail {
  return {
    id: "INV-X",
    state: "Pending",
    submitter: "GSUB000",
    payer: "GPAY000",
    token: "USDC",
    amount: "100",
    discountRateBps: 300,
    effectiveYieldPct: "3.00",
    dueDate: "2099-12-31",
    createdAt: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}

describe("stateBadge", () => {
  it("returns [PAID] for Paid state", () => expect(stateBadge("Paid")).toBe("[PAID]"));
  it("returns [FUNDED] for Funded state", () => expect(stateBadge("Funded")).toBe("[FUNDED]"));
  it("returns [PENDING] for Pending state", () => expect(stateBadge("Pending")).toBe("[PENDING]"));
  it("returns [EXPIRED] for Expired state", () => expect(stateBadge("Expired")).toBe("[EXPIRED]"));
  it("returns [DISPUTED] for Disputed state", () => expect(stateBadge("Disputed")).toBe("[DISPUTED]"));
  it("returns [CANCELLED] for Cancelled state", () => expect(stateBadge("Cancelled")).toBe("[CANCELLED]"));
});

describe("timeUntilExpiry", () => {
  it("returns 'Expired' for a past date", () => {
    expect(timeUntilExpiry("2020-01-01")).toBe("Expired");
  });

  it("returns days and hours for a future date", () => {
    const future = new Date(Date.now() + 3 * 86_400_000).toISOString().slice(0, 10);
    expect(timeUntilExpiry(future)).toMatch(/\d+d \d+h/);
  });
});

describe("formatDetail", () => {
  it("includes all required fields", () => {
    const out = formatDetail(mockInvoice({ id: "INV-X", amount: "100", token: "USDC" }));
    expect(out).toContain("INV-X");
    expect(out).toContain("100 USDC");
    expect(out).toContain("3.00%");
    expect(out).toContain("[PENDING]");
  });

  it("shows '—' when LP is absent", () => {
    const out = formatDetail(mockInvoice({ lp: undefined }));
    expect(out).toContain("—");
  });
});

describe("buildTimeline", () => {
  it("marks Pending as current with ◉", () => {
    expect(buildTimeline("Pending")).toContain("◉ Pending");
  });

  it("marks Funded steps before Paid as completed with ●", () => {
    const timeline = buildTimeline("Paid");
    expect(timeline).toContain("● Pending");
    expect(timeline).toContain("● Funded");
  });

  it("marks Paid as current with ◉", () => {
    expect(buildTimeline("Paid")).toContain("◉ Paid");
  });
});
