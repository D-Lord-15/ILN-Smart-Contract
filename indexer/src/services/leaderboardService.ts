import type Database from 'better-sqlite3';

export interface LeaderboardEntry {
  rank: number;
  address: string;
  score: number;
  invoicesPaid: number;
  invoicesDefaulted: number;
  totalVolume: string;
}

interface ReputationRow {
  address: string;
  new_score: number;
  invoices_paid: number;
  invoices_defaulted: number;
}

export function getLeaderboard(
  db: Database.Database,
  limit: number = 50,
  token?: string
): LeaderboardEntry[] {
  const safeLimit = Math.min(Math.max(1, limit), 100);

  let rows: ReputationRow[];

  if (token) {
    rows = db
      .prepare(
        `
        SELECT
          r.address,
          r.new_score,
          r.invoices_paid,
          r.invoices_defaulted
        FROM reputation_updates r
        WHERE r.id IN (
          SELECT MAX(id) FROM reputation_updates GROUP BY address
        )
        AND r.address IN (
          SELECT DISTINCT payer FROM invoices WHERE token = ?
        )
        ORDER BY r.new_score DESC
        LIMIT ?
      `
      )
      .all(token, safeLimit) as ReputationRow[];
  } else {
    rows = db
      .prepare(
        `
        SELECT
          r.address,
          r.new_score,
          r.invoices_paid,
          r.invoices_defaulted
        FROM reputation_updates r
        WHERE r.id IN (
          SELECT MAX(id) FROM reputation_updates GROUP BY address
        )
        ORDER BY r.new_score DESC
        LIMIT ?
      `
      )
      .all(safeLimit) as ReputationRow[];
  }

  const volumeStmt = db.prepare(
    `SELECT COALESCE(SUM(CAST(amount_paid AS INTEGER)), 0) as vol
     FROM events e
     JOIN invoices i ON e.invoice_id = i.id
     WHERE e.event_type = 'paid' AND e.data LIKE '%"payer":"%'
     AND json_extract(e.data, '$.payer') = ?`
  );

  return rows.map((row, idx) => {
    const volResult = volumeStmt.get(row.address) as { vol: number } | undefined;
    return {
      rank: idx + 1,
      address: row.address,
      score: row.new_score,
      invoicesPaid: row.invoices_paid,
      invoicesDefaulted: row.invoices_defaulted,
      totalVolume: String(volResult?.vol || 0),
    };
  });
}
