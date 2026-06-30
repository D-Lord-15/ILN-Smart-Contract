/**
 * getReputation — read an address's detailed reputation profile from
 * the on-chain invoice-liquidity contract.
 *
 * Wraps the `get_reputation(address)` view function. Unknown addresses
 * return a zeroed profile (matching the contract's lazy-init behaviour).
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
import { decodeReputationScore, type ReputationProfile } from "../utils/xdrDecoder.js";

// ---------------------------------------------------------------------------
// G-address validation
// ---------------------------------------------------------------------------

const G_ADDRESS_RE = /^G[A-Z2-7]{55}$/;

function isValidGAddress(address: string): boolean {
  return G_ADDRESS_RE.test(address);
}

// ---------------------------------------------------------------------------
// getReputation
// ---------------------------------------------------------------------------

/**
 * Query the reputation profile for a Stellar address.
 *
 * Performs a read-only Soroban simulation — no on-chain mutation, no
 * transaction fees, and no signer required.
 *
 * @param server              - Soroban RPC server for the target network
 * @param contractId          - Deployed invoice-liquidity contract address
 * @param address             - Stellar G… address to look up
 * @param networkPassphrase   - Stellar network passphrase (default: TESTNET)
 * @returns ReputationProfile (zeroed for unknown / never-active addresses)
 *
 * @throws When `address` is not a valid Stellar G-address
 * @throws When the Soroban simulation fails (RPC unreachable, contract not found)
 *
 * @example
 * ```ts
 * const rep = await getReputation(server, CONTRACT_ID, "GAA...");
 * console.log(`Score: ${rep.score}, Submitted: ${rep.invoicesSubmitted}`);
 * ```
 */
export async function getReputation(
  server: SorobanRpc.Server,
  contractId: string,
  address: string,
  networkPassphrase: string = Networks.TESTNET
): Promise<ReputationProfile> {
  if (!isValidGAddress(address)) {
    throw new Error(
      `Invalid Stellar address: "${address}". Must be a G… public key.`
    );
  }

  const contract = new Contract(contractId);
  const op = contract.call(
    "get_reputation",
    new Address(address).toScVal()
  );

  const sourceAccount = new Account(
    "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
    "0"
  );

  const simTx = new TransactionBuilder(sourceAccount, {
    fee: BASE_FEE,
    networkPassphrase,
  })
    .addOperation(op)
    .setTimeout(30)
    .build();

  const sim = await retry(() => server.simulateTransaction(simTx));

  if (SorobanRpc.Api.isSimulationError(sim)) {
    throw new Error(`get_reputation simulation failed: ${sim.error}`);
  }

  // The contract returns a zeroed ReputationProfile for unknown addresses
  if (!sim.result?.retval) {
    return decodeReputationScore({}, address);
  }

  const raw = scValToNative(sim.result.retval) as Record<string, unknown>;
  return decodeReputationScore(raw, address);
}
