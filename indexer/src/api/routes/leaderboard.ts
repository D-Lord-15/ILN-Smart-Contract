import { Router } from 'express';
import type Database from 'better-sqlite3';
import { getLeaderboard, type LeaderboardEntry } from '../../services/leaderboardService.js';

export function createLeaderboardRouter(db: Database.Database): Router {
  const router = Router();

  router.get('/leaderboard', (req, res) => {
    const limitParam = req.query.limit;
    const token = req.query.token as string | undefined;

    let limit = 50;
    if (limitParam !== undefined) {
      const parsed = parseInt(limitParam as string, 10);
      if (isNaN(parsed) || parsed < 1) {
        res.status(400).json({ error: 'Invalid limit parameter' });
        return;
      }
      limit = Math.min(parsed, 100);
    }

    const entries: LeaderboardEntry[] = getLeaderboard(db, limit, token);
    res.json(entries);
  });

  return router;
}
