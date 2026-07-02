/**
 * Tests for profile management — --profile flag (#246, #247).
 */
import fs from "fs";
import os from "os";
import path from "path";
import {
  saveProfile,
  loadProfile,
  listProfiles,
  resolveProfile,
  setConfigValue,
} from "../../src/config";

let tmpDir: string;

beforeEach(() => {
  tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "iln-prof-"));
});

afterEach(() => {
  fs.rmSync(tmpDir, { recursive: true, force: true });
});

describe("profile management (#246)", () => {
  it("saveProfile writes a JSON file under <base>/.iln/profiles/", () => {
    saveProfile({ name: "lp-account", publicKey: "GABC123" }, tmpDir);
    const file = path.join(tmpDir, ".iln", "profiles", "lp-account.json");
    expect(fs.existsSync(file)).toBe(true);
  });

  it("loadProfile reads back the saved profile", () => {
    saveProfile({ name: "my-profile", publicKey: "GXYZ", secretKey: "SXYZ" }, tmpDir);
    const p = loadProfile("my-profile", tmpDir);
    expect(p.name).toBe("my-profile");
    expect(p.publicKey).toBe("GXYZ");
  });

  it("loadProfile throws when profile does not exist", () => {
    expect(() => loadProfile("nonexistent", tmpDir)).toThrow(/not found/);
  });

  it("listProfiles returns all saved profiles", () => {
    saveProfile({ name: "a", publicKey: "GA" }, tmpDir);
    saveProfile({ name: "b", publicKey: "GB" }, tmpDir);
    const names = listProfiles(tmpDir).map((p) => p.name);
    expect(names).toEqual(expect.arrayContaining(["a", "b"]));
  });

  it("resolveProfile returns null when no matching profile exists", () => {
    expect(resolveProfile(undefined, tmpDir)).toBeNull();
  });

  it("resolveProfile returns the named profile when it exists", () => {
    saveProfile({ name: "freelancer", publicKey: "GFL" }, tmpDir);
    const result = resolveProfile("freelancer", tmpDir);
    expect(result?.name).toBe("freelancer");
  });

  it("resolveProfile uses config defaultProfile when no flag given", () => {
    saveProfile({ name: "main-acc", publicKey: "GMN" }, tmpDir);
    setConfigValue("defaultProfile", "main-acc", tmpDir);
    const result = resolveProfile(undefined, tmpDir);
    expect(result?.name).toBe("main-acc");
  });
});
