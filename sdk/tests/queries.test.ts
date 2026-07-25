import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
import { getInvoice, listInvoicesBySubmitter, listInvoicesByLP, getSubmitterInvoices } from '../src/methods/queries.js';
import { ILNError } from '../src/errors.js';
import { Account, SorobanRpc } from '@stellar/stellar-sdk';

describe('queries', () => {
  const mockServer = { simulateTransaction: vi.fn() } as unknown as SorobanRpc.Server;
  const mockAccount = new Account("GAGZSXAR7P7PASD2PGYISBMEZCMSI35TRJXYZTZNNCAUZRDEMHQM2XJS", "1");

  it('getInvoice throws InvoiceNotFound', async () => {
    // @ts-ignore
    mockServer.simulateTransaction.mockResolvedValue({ error: 'NotFound' });
    await expect(getInvoice(mockServer, "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4", 1n, mockAccount, 'pass')).rejects.toThrow(ILNError.InvoiceNotFound);
  });

  it('getSubmitterInvoices returns empty array when retval is null', async () => {
    // @ts-ignore
    mockServer.simulateTransaction.mockResolvedValue({ result: { retval: null } });
    const result = await getSubmitterInvoices(mockServer, "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4", "GAGZSXAR7P7PASD2PGYISBMEZCMSI35TRJXYZTZNNCAUZRDEMHQM2XJS", mockAccount, 'pass', 0, 10);
    expect(result).toEqual([]);
  });

  it('getSubmitterInvoices throws error on simulation error', async () => {
    // @ts-ignore
    mockServer.simulateTransaction.mockResolvedValue({ error: 'ContractError' });
    await expect(getSubmitterInvoices(mockServer, "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4", "GAGZSXAR7P7PASD2PGYISBMEZCMSI35TRJXYZTZNNCAUZRDEMHQM2XJS", mockAccount, 'pass', 0, 10)).rejects.toThrow();
  });

  it('getSubmitterInvoices uses default pagination values', async () => {
    // @ts-ignore
    mockServer.simulateTransaction.mockResolvedValue({ result: { retval: null } });
    const result = await getSubmitterInvoices(mockServer, "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4", "GAGZSXAR7P7PASD2PGYISBMEZCMSI35TRJXYZTZNNCAUZRDEMHQM2XJS", mockAccount, 'pass');
    expect(result).toEqual([]);
  });
});
