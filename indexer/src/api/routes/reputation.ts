import { Router } from 'express';
import type Database from 'better-sqlite3';

interface ReputationUpdateRow {
  new_score: number;
  invoices_submitted: number;
  invoices_paid: number;
  invoices_defaulted: number;
  ledger: number;
  timestamp: number;
  event_type: string;
}

interface HistoryEntry {
  ledger: number;
  score: number;
  eventType: string;
  timestamp: number;
}

interface ReputationResponse {
  score: number;
  invoicesPaid: number;
  invoicesDefaulted: number;
  invoicesSubmitted: number;
  lastActivityLedger: number;
  history: HistoryEntry[];
}

export function createReputationRouter(db: Database.Database): Router {
  const router = Router();

  router.get('/reputation/:address', (req, res) => {
    const { address } = req.params;
    const historyPeriod = (req.query.historyPeriod as string) || 'all';

    const latestRow = db
      .prepare(
        `
        SELECT new_score, invoices_submitted, invoices_paid, invoices_defaulted, ledger
        FROM reputation_updates
        WHERE address = ?
        ORDER BY id DESC
        LIMIT 1
      `
      )
      .get(address) as ReputationUpdateRow | undefined;

    if (!latestRow) {
      res.json({
        score: 0,
        invoicesPaid: 0,
        invoicesDefaulted: 0,
        invoicesSubmitted: 0,
        lastActivityLedger: 0,
        history: [],
      });
      return;
    }

    let dateFilter = '';
    if (historyPeriod === '30d') {
      dateFilter = 'AND timestamp >= ?';
    } else if (historyPeriod === '90d') {
      dateFilter = 'AND timestamp >= ?';
    }

    let historyParams: (string | number)[] = [address];
    if (historyPeriod === '30d') {
      historyParams.push(Math.floor(Date.now() / 1000) - 30 * 24 * 60 * 60);
    } else if (historyPeriod === '90d') {
      historyParams.push(Math.floor(Date.now() / 1000) - 90 * 24 * 60 * 60);
    }

    const historyRows = db
      .prepare(
        `
        SELECT new_score, ledger, event_type, timestamp
        FROM reputation_updates
        WHERE address = ? ${dateFilter}
        ORDER BY timestamp ASC
      `
      )
      .all(...historyParams) as Array<{
      new_score: number;
      ledger: number;
      event_type: string;
      timestamp: number;
    }>;

    const response: ReputationResponse = {
      score: latestRow.new_score,
      invoicesPaid: latestRow.invoices_paid,
      invoicesDefaulted: latestRow.invoices_defaulted,
      invoicesSubmitted: latestRow.invoices_submitted,
      lastActivityLedger: latestRow.ledger,
      history: historyRows.map((row) => ({
        ledger: row.ledger,
        score: row.new_score,
        eventType: row.event_type,
        timestamp: row.timestamp,
      })),
    };

    res.json(response);
  });

  return router;
}
