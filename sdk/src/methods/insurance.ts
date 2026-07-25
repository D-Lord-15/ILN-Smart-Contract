/**
 * Insurance pool queries — read status and configuration of the
 * default-protection insurance pool.
 */

import {
  Contract,
  SorobanRpc,
  TransactionBuilder,
  Account,
  BASE_FEE,
  scValToNative,
  Address,
  Networks,
} from "@stellar/stellar-sdk";
import { retry } from "../utils/retry.js";
import { validateGAddress, validateContractId } from "../utils/validate.js";
import { InsuranceContractError } from "../errors.js";
import type { InsurancePoolInfo } from "@invoice-liquidity/types";

/**
 * Helper to execute a read-only contract simulation call.
 */
async function simulateCall(
  server: SorobanRpc.Server,
  contractId: string,
  methodName: string,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  args: any[] = [],
  networkPassphrase: string = Networks.TESTNET
) {
  const contract = new Contract(contractId);
  const op = contract.call(methodName, ...args);
  const sourceAccount = new Account(
    "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
    "0"
  );
  const tx = new TransactionBuilder(sourceAccount, {
    fee: BASE_FEE,
    networkPassphrase,
  })
    .addOperation(op)
    .setTimeout(30)
    .build();

  const sim = await retry(() => server.simulateTransaction(tx));

  if (SorobanRpc.Api.isSimulationError(sim)) {
    throw InsuranceContractError.fromError(sim.error);
  }

  return sim.result?.retval;
}

/**
 * Get the current insurance pool balance.
 *
 * @param server              - Soroban RPC server for the target network
 * @param contractId          - Deployed insurance pool contract address
 * @param networkPassphrase   - Stellar network passphrase (default: TESTNET)
 * @returns The current pool balance as a bigint
 */
export async function getPoolBalance(
  server: SorobanRpc.Server,
  contractId: string,
  networkPassphrase: string = Networks.TESTNET
): Promise<bigint> {
  validateContractId(contractId);
  const retval = await simulateCall(server, contractId, "get_pool_balance", [], networkPassphrase);
  if (!retval) {
    return 0n;
  }
  return scValToNative(retval) as bigint;
}

/**
 * Get the configured coverage cap.
 *
 * @param server              - Soroban RPC server for the target network
 * @param contractId          - Deployed insurance pool contract address
 * @param networkPassphrase   - Stellar network passphrase (default: TESTNET)
 * @returns The configured coverage cap as a bigint
 */
export async function getCoverage(
  server: SorobanRpc.Server,
  contractId: string,
  networkPassphrase: string = Networks.TESTNET
): Promise<bigint> {
  validateContractId(contractId);
  const retval = await simulateCall(server, contractId, "get_coverage", [], networkPassphrase);
  if (!retval) {
    return 0n;
  }
  return scValToNative(retval) as bigint;
}

/**
 * Check if a liquidity provider is enrolled in the insurance pool.
 *
 * @param server              - Soroban RPC server for the target network
 * @param contractId          - Deployed insurance pool contract address
 * @param lpAddress           - The LP's Stellar G... address
 * @param networkPassphrase   - Stellar network passphrase (default: TESTNET)
 * @returns True if enrolled, false otherwise
 */
export async function isEnrolled(
  server: SorobanRpc.Server,
  contractId: string,
  lpAddress: string,
  networkPassphrase: string = Networks.TESTNET
): Promise<boolean> {
  validateContractId(contractId);
  validateGAddress(lpAddress);
  const retval = await simulateCall(
    server,
    contractId,
    "is_enrolled",
    [new Address(lpAddress).toScVal()],
    networkPassphrase
  );
  if (!retval) {
    return false;
  }
  return scValToNative(retval) as boolean;
}

/**
 * Get total premiums paid by an LP.
 *
 * @param server              - Soroban RPC server for the target network
 * @param contractId          - Deployed insurance pool contract address
 * @param lpAddress           - The LP's Stellar G... address
 * @param networkPassphrase   - Stellar network passphrase (default: TESTNET)
 * @returns The total premiums paid as a bigint
 */
export async function getPremiumsPaid(
  server: SorobanRpc.Server,
  contractId: string,
  lpAddress: string,
  networkPassphrase: string = Networks.TESTNET
): Promise<bigint> {
  validateContractId(contractId);
  validateGAddress(lpAddress);
  const retval = await simulateCall(
    server,
    contractId,
    "get_premiums_paid",
    [new Address(lpAddress).toScVal()],
    networkPassphrase
  );
  if (!retval) {
    return 0n;
  }
  return scValToNative(retval) as bigint;
}

/**
 * Get all insurance pool info for an LP in a single call.
 *
 * @param server              - Soroban RPC server for the target network
 * @param contractId          - Deployed insurance pool contract address
 * @param lpAddress           - The LP's Stellar G... address
 * @param networkPassphrase   - Stellar network passphrase (default: TESTNET)
 * @returns Combined InsurancePoolInfo object
 */
export async function getInsurancePoolInfo(
  server: SorobanRpc.Server,
  contractId: string,
  lpAddress: string,
  networkPassphrase: string = Networks.TESTNET
): Promise<InsurancePoolInfo> {
  validateContractId(contractId);
  validateGAddress(lpAddress);
  const [poolBalance, coverage, isEnrolledVal, premiumsPaid] = await Promise.all([
    getPoolBalance(server, contractId, networkPassphrase),
    getCoverage(server, contractId, networkPassphrase),
    isEnrolled(server, contractId, lpAddress, networkPassphrase),
    getPremiumsPaid(server, contractId, lpAddress, networkPassphrase),
  ]);

  return {
    poolBalance,
    coverage,
    isEnrolled: isEnrolledVal,
    premiumsPaid,
  };
}
