/**
 * Distribution queries — read stats from the distribution reward contract.
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

/**
 * Fetch a participant's accrued distribution tokens.
 *
 * Wraps the `get_accrual(participant)` view function.
 *
 * @param server              - Soroban RPC server for the target network
 * @param contractId          - Deployed distribution contract address
 * @param participantAddress  - Stellar address (G...) of the participant
 * @param networkPassphrase   - Stellar network passphrase (default: TESTNET)
 * @returns The total earned tokens as a number
 */
export async function getDistributionAccrual(
  server: SorobanRpc.Server,
  contractId: string,
  participantAddress: string,
  networkPassphrase: string = Networks.TESTNET
): Promise<number> {
  validateContractId(contractId);
  validateGAddress(participantAddress);

  const contract = new Contract(contractId);
  const op = contract.call(
    "get_accrual",
    new Address(participantAddress).toScVal()
  );

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
    throw new Error(`get_accrual simulation failed: ${sim.error}`);
  }

  if (!sim.result?.retval) {
    return 0;
  }

  const rawVal = scValToNative(sim.result.retval) as bigint;
  return Number(rawVal);
}
