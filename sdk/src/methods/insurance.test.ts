import { vi, describe, it, expect, beforeAll, beforeEach } from 'vitest';
import {
  getPoolBalance,
  getCoverage,
  isEnrolled,
  getPremiumsPaid,
  getInsurancePoolInfo,
} from "./insurance.js";
import { SorobanRpc, Keypair, Address } from "@stellar/stellar-sdk";

// ---------------------------------------------------------------------------
// vi.mock — patch scValToNative only
// ---------------------------------------------------------------------------

vi.mock("@stellar/stellar-sdk", async () => {
  const actual = await vi.importActual<typeof import("@stellar/stellar-sdk")>("@stellar/stellar-sdk");
  return {
    ...actual,
    scValToNative: vi.fn().mockImplementation(actual.scValToNative),
  };
});

import { scValToNative } from "@stellar/stellar-sdk";
const mockScValToNative = scValToNative as vi.Mock;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

let VALID_LP: string;
let CONTRACT_ID: string;

beforeAll(() => {
  VALID_LP = Keypair.random().publicKey();
  const buf = Buffer.alloc(32);
  for (let i = 0; i < 32; i++) buf[i] = i + 1;
  CONTRACT_ID = Address.contract(buf).toString();
});

beforeEach(() => {
  vi.clearAllMocks();
});

// ---------------------------------------------------------------------------
// Mock server helpers
// ---------------------------------------------------------------------------

function serverWith(sim: unknown): SorobanRpc.Server {
  return {
    simulateTransaction: vi.fn().mockResolvedValue(sim),
  } as unknown as SorobanRpc.Server;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("getPoolBalance", () => {
  it("returns pool balance on success", async () => {
    const server = serverWith({ result: { retval: {} } });
    mockScValToNative.mockReturnValue(5000n);

    const balance = await getPoolBalance(server, CONTRACT_ID);
    expect(balance).toBe(5000n);
  });

  it("returns 0n if no retval in simulation", async () => {
    const server = serverWith({ result: { retval: null } });

    const balance = await getPoolBalance(server, CONTRACT_ID);
    expect(balance).toBe(0n);
  });

  it("throws validation error for invalid contractId", async () => {
    const server = serverWith({});
    await expect(getPoolBalance(server, "invalid")).rejects.toThrow("Invalid contract ID");
  });
});

describe("getCoverage", () => {
  it("returns coverage on success", async () => {
    const server = serverWith({ result: { retval: {} } });
    mockScValToNative.mockReturnValue(1000n);

    const coverage = await getCoverage(server, CONTRACT_ID);
    expect(coverage).toBe(1000n);
  });

  it("returns 0n if no retval in simulation", async () => {
    const server = serverWith({ result: { retval: null } });

    const coverage = await getCoverage(server, CONTRACT_ID);
    expect(coverage).toBe(0n);
  });

  it("throws validation error for invalid contractId", async () => {
    const server = serverWith({});
    await expect(getCoverage(server, "invalid")).rejects.toThrow("Invalid contract ID");
  });
});

describe("isEnrolled", () => {
  it("returns is_enrolled boolean on success", async () => {
    const server = serverWith({ result: { retval: {} } });
    mockScValToNative.mockReturnValue(true);

    const enrolled = await isEnrolled(server, CONTRACT_ID, VALID_LP);
    expect(enrolled).toBe(true);
  });

  it("returns false if no retval in simulation", async () => {
    const server = serverWith({ result: { retval: null } });

    const enrolled = await isEnrolled(server, CONTRACT_ID, VALID_LP);
    expect(enrolled).toBe(false);
  });

  it("throws validation error for invalid contractId or LP address", async () => {
    const server = serverWith({});
    await expect(isEnrolled(server, "invalid", VALID_LP)).rejects.toThrow("Invalid contract ID");
    await expect(isEnrolled(server, CONTRACT_ID, "invalid")).rejects.toThrow("Invalid Stellar address");
  });
});

describe("getPremiumsPaid", () => {
  it("returns premiums paid on success", async () => {
    const server = serverWith({ result: { retval: {} } });
    mockScValToNative.mockReturnValue(200n);

    const premiums = await getPremiumsPaid(server, CONTRACT_ID, VALID_LP);
    expect(premiums).toBe(200n);
  });

  it("returns 0n if no retval in simulation", async () => {
    const server = serverWith({ result: { retval: null } });

    const premiums = await getPremiumsPaid(server, CONTRACT_ID, VALID_LP);
    expect(premiums).toBe(0n);
  });

  it("throws validation error for invalid contractId or LP address", async () => {
    const server = serverWith({});
    await expect(getPremiumsPaid(server, "invalid", VALID_LP)).rejects.toThrow("Invalid contract ID");
    await expect(getPremiumsPaid(server, CONTRACT_ID, "invalid")).rejects.toThrow("Invalid Stellar address");
  });
});

describe("getInsurancePoolInfo", () => {
  it("combines all pool queries successfully", async () => {
    const server = serverWith({ result: { retval: {} } });
    mockScValToNative
      .mockReturnValueOnce(5000n)  // getPoolBalance
      .mockReturnValueOnce(1000n)  // getCoverage
      .mockReturnValueOnce(true)   // isEnrolled
      .mockReturnValueOnce(200n);  // getPremiumsPaid

    const info = await getInsurancePoolInfo(server, CONTRACT_ID, VALID_LP);
    expect(info).toEqual({
      poolBalance: 5000n,
      coverage: 1000n,
      isEnrolled: true,
      premiumsPaid: 200n,
    });
  });

  it("throws validation error for invalid contractId or LP address", async () => {
    const server = serverWith({});
    await expect(getInsurancePoolInfo(server, "invalid", VALID_LP)).rejects.toThrow("Invalid contract ID");
    await expect(getInsurancePoolInfo(server, CONTRACT_ID, "invalid")).rejects.toThrow("Invalid Stellar address");
  });
});
