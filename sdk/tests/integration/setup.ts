/**
 * Shared setup for the testnet integration suite.
 *
 * These tests run against the live Stellar testnet deployment (not mocks), so
 * they require two Friendbot-funded keypairs supplied via environment:
 *
 *   - TEST_SUBMITTER_SECRET — submits and cancels test invoices
 *   - TEST_LP_SECRET        — funds test invoices as the liquidity provider
 *
 * The suite is excluded from the unit-test run and executed nightly in CI.
 */

import {
  Keypair,
  SorobanRpc,
  Transaction,
  Networks,
} from "@stellar/stellar-sdk";
import { TESTNET_RPC_URL } from "../../src/client.js";

export const NETWORK_PASSPHRASE = Networks.TESTNET;
export const FRIENDBOT_URL = "https://friendbot.stellar.org";

/** Read a required secret key from the environment or throw a clear error. */
export function requireSecret(name: string): string {
  const value = process.env[name];
  if (!value) {
    throw new Error(
      `Missing ${name}. Integration tests require Friendbot-funded testnet ` +
        `keypairs in TEST_SUBMITTER_SECRET and TEST_LP_SECRET.`
    );
  }
  return value;
}

/** True when both integration secrets are present. */
export function hasIntegrationSecrets(): boolean {
  return Boolean(process.env["TEST_SUBMITTER_SECRET"] && process.env["TEST_LP_SECRET"]);
}

/** Create a Soroban RPC server pointed at testnet. */
export function testnetServer(): SorobanRpc.Server {
  return new SorobanRpc.Server(process.env["TEST_RPC_URL"] ?? TESTNET_RPC_URL);
}

/** Ensure an account exists on testnet by topping it up via Friendbot. */
export async function fundWithFriendbot(publicKey: string): Promise<void> {
  const res = await fetch(`${FRIENDBOT_URL}/?addr=${encodeURIComponent(publicKey)}`);
  // 400 typically means the account already exists and is funded — tolerate it.
  if (!res.ok && res.status !== 400) {
    throw new Error(`Friendbot funding failed for ${publicKey}: ${res.status}`);
  }
}

/**
 * Build a `signTransaction` callback that simulates (to attach the Soroban
 * footprint), signs with the given keypair, and returns the signed
 * transaction — matching the shape the SDK methods expect.
 */
export function keypairSignTx(
  keypair: Keypair,
  server: SorobanRpc.Server
): (tx: Transaction) => Promise<Transaction> {
  return async (tx: Transaction): Promise<Transaction> => {
    const prepared = await server.prepareTransaction(tx);
    (prepared as Transaction).sign(keypair);
    return prepared as Transaction;
  };
}
