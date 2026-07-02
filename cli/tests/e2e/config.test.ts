/**
 * Tests for config module — iln config command (#245, #247).
 */
import fs from "fs";
import os from "os";
import path from "path";
import {
  loadConfig,
  saveConfig,
  resetConfig,
  getConfigValue,
  setConfigValue,
  DEFAULTS,
} from "../../src/config";

let tmpDir: string;

beforeEach(() => {
  tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "iln-cfg-"));
});

afterEach(() => {
  fs.rmSync(tmpDir, { recursive: true, force: true });
});

describe("config module (#245)", () => {
  it("loadConfig returns defaults when no config file exists", () => {
    const cfg = loadConfig(tmpDir);
    expect(cfg.network).toBe("testnet");
    expect(cfg.rpcUrl).toContain("soroban-testnet");
  });

  it("saveConfig and loadConfig round-trip correctly", () => {
    saveConfig({ network: "mainnet", rpcUrl: "https://rpc.example.com" }, tmpDir);
    const cfg = loadConfig(tmpDir);
    expect(cfg.network).toBe("mainnet");
    expect(cfg.rpcUrl).toBe("https://rpc.example.com");
  });

  it("setConfigValue persists network and loadConfig reads it back", () => {
    setConfigValue("network", "mainnet", tmpDir);
    expect(loadConfig(tmpDir).network).toBe("mainnet");
  });

  it("setConfigValue persists rpcUrl", () => {
    setConfigValue("rpcUrl", "https://custom.rpc", tmpDir);
    expect(getConfigValue("rpcUrl", tmpDir)).toBe("https://custom.rpc");
  });

  it("setConfigValue throws on unknown key", () => {
    expect(() => setConfigValue("unknownKey", "foo", tmpDir)).toThrow(/Unknown config key/);
  });

  it("setConfigValue throws on invalid network value", () => {
    expect(() => setConfigValue("network", "devnet", tmpDir)).toThrow(/testnet.*mainnet/);
  });

  it("getConfigValue returns the stored value", () => {
    setConfigValue("rpcUrl", "https://custom-rpc.example.com", tmpDir);
    expect(getConfigValue("rpcUrl", tmpDir)).toBe("https://custom-rpc.example.com");
  });

  it("resetConfig restores defaults", () => {
    setConfigValue("network", "mainnet", tmpDir);
    resetConfig(tmpDir);
    expect(loadConfig(tmpDir).network).toBe(DEFAULTS.network);
  });
});
