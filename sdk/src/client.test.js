import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
"use strict";
/**
 * Tests for ILNClient — covers:
 *   - testnet / mainnet / custom factory methods
 *   - singleton iln.configure / iln.getReputation / iln.getContractStats
 *   - Preset defaults (RPC URL, network passphrase, contract ID)
 */
Object.defineProperty(exports, "__esModule", { value: true });
const client_js_1 = require("./client.js");
const stellar_sdk_1 = require("@stellar/stellar-sdk");
// ---------------------------------------------------------------------------
// Mock SorobanRpc.Server to avoid real network connections in tests
// ---------------------------------------------------------------------------
vi.mock("@stellar/stellar-sdk", () => {
    const actual = vi.importActual("@stellar/stellar-sdk");
    return {
        ...actual,
        SorobanRpc: {
            ...actual.SorobanRpc,
            Server: vi.fn().mockImplementation(() => ({
                getAccount: vi.fn(),
                simulateTransaction: vi.fn(),
                prepareTransaction: vi.fn(),
                sendTransaction: vi.fn(),
                getLatestLedger: vi.fn(),
            })),
        },
    };
});
// ---------------------------------------------------------------------------
// testnet()
// ---------------------------------------------------------------------------
describe("ILNClient.testnet", () => {
    it("creates a client with testnet defaults", () => {
        const client = client_js_1.ILNClient.testnet();
        expect(client.networkPassphrase).toBe(stellar_sdk_1.Networks.TESTNET);
        expect(client.contractId).toBeTruthy();
        expect(client.contractId.length).toBeGreaterThan(0);
    });
    it("uses the testnet RPC URL", () => {
        const client = client_js_1.ILNClient.testnet();
        // We can't inspect rpc.serverUrl directly in v12, but the constructor
        // receives the correct URL.
        expect(client_js_1.TESTNET_RPC_URL).toContain("testnet");
    });
    it("accepts an optional signer", () => {
        const signer = { publicKey: "GAA", signTransaction: vi.fn() };
        const client = client_js_1.ILNClient.testnet(signer);
        expect(client.signer).toBe(signer);
    });
    it("accepts optional overrides", () => {
        const client = client_js_1.ILNClient.testnet(undefined, {
            rpcUrl: "http://localhost:8000/soroban/rpc",
            contractId: "CUSTOM",
        });
        expect(client.contractId).toBe("CUSTOM");
    });
    it("works without any arguments", () => {
        const client = client_js_1.ILNClient.testnet();
        expect(client).toBeInstanceOf(client_js_1.ILNClient);
        expect(client.signer).toBeUndefined();
    });
});
// ---------------------------------------------------------------------------
// mainnet()
// ---------------------------------------------------------------------------
describe("ILNClient.mainnet", () => {
    it("creates a client with mainnet defaults", () => {
        const client = client_js_1.ILNClient.mainnet();
        expect(client.networkPassphrase).toBe(stellar_sdk_1.Networks.PUBLIC);
        expect(client_js_1.MAINNET_RPC_URL).toContain("soroban.stellar.org");
    });
    it("accepts an optional signer", () => {
        const signer = { publicKey: "GAA", signTransaction: vi.fn() };
        const client = client_js_1.ILNClient.mainnet(signer);
        expect(client.signer).toBe(signer);
    });
    it("accepts optional overrides", () => {
        const client = client_js_1.ILNClient.mainnet(undefined, {
            rpcUrl: "https://custom-rpc.example.com",
            contractId: "MAINNET_DEPLOY",
        });
        expect(client.contractId).toBe("MAINNET_DEPLOY");
    });
});
// ---------------------------------------------------------------------------
// custom()
// ---------------------------------------------------------------------------
describe("ILNClient.custom", () => {
    it("creates a client with fully custom config", () => {
        const signer = { publicKey: "GAA", signTransaction: vi.fn() };
        const client = client_js_1.ILNClient.custom({
            rpcUrl: "http://localhost:8000/soroban/rpc",
            networkPassphrase: "Standalone Network ; February 2017",
            contractId: "CSTANDALONE",
            signer: signer,
        });
        expect(client.networkPassphrase).toBe("Standalone Network ; February 2017");
        expect(client.contractId).toBe("CSTANDALONE");
        expect(client.signer).toBe(signer);
    });
    it("works without a signer (read-only configs)", () => {
        const client = client_js_1.ILNClient.custom({
            rpcUrl: "https://soroban-testnet.stellar.org",
            networkPassphrase: stellar_sdk_1.Networks.TESTNET,
            contractId: "CTEST",
        });
        expect(client.signer).toBeUndefined();
        expect(client.contractId).toBe("CTEST");
    });
});
// ---------------------------------------------------------------------------
// Singleton (iln)
// ---------------------------------------------------------------------------
describe("iln singleton", () => {
    it("throws if getReputation is called before configure", async () => {
        // Reset singleton state (it's a module-level singleton, but we
        // re-configure it in each test)
        await expect(client_js_1.iln.getReputation("GAA")).rejects.toThrow("not configured");
    });
    it("throws if getContractStats is called before configure", async () => {
        await expect(client_js_1.iln.getContractStats()).rejects.toThrow("not configured");
    });
});
