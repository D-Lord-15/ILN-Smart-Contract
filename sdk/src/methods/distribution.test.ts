import { vi, describe, it, expect, beforeAll, beforeEach } from 'vitest';
import { getDistributionAccrual } from "./distribution.js";
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

let VALID_PARTICIPANT: string;
let CONTRACT_ID: string;

beforeAll(() => {
  VALID_PARTICIPANT = Keypair.random().publicKey();
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

describe("getDistributionAccrual", () => {
  it("returns accrued tokens as a number on success", async () => {
    const server = serverWith({ result: { retval: {} } });
    mockScValToNative.mockReturnValue(50000000n);

    const accrual = await getDistributionAccrual(server, CONTRACT_ID, VALID_PARTICIPANT);
    expect(accrual).toBe(50000000);
    expect(typeof accrual).toBe("number");
  });

  it("returns 0 if no retval in simulation", async () => {
    const server = serverWith({ result: { retval: null } });

    const accrual = await getDistributionAccrual(server, CONTRACT_ID, VALID_PARTICIPANT);
    expect(accrual).toBe(0);
  });

  it("throws validation error for invalid contractId or participant address", async () => {
    const server = serverWith({});
    await expect(getDistributionAccrual(server, "invalid", VALID_PARTICIPANT)).rejects.toThrow("Invalid contract ID");
    await expect(getDistributionAccrual(server, CONTRACT_ID, "invalid")).rejects.toThrow("Invalid Stellar address");
  });

  it("throws when simulation returns an error object", async () => {
    const server = serverWith({ error: "simulation crash", _parsed: true });

    await expect(
      getDistributionAccrual(server, CONTRACT_ID, VALID_PARTICIPANT)
    ).rejects.toThrow("get_accrual simulation failed");
  });
});
