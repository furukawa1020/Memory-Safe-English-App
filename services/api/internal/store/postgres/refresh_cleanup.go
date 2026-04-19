package postgres

import (
	"context"
	"fmt"
	"time"
)

func (s *Store) DeleteExpiredRefreshSessions(ctx context.Context, now time.Time) (int64, error) {
	result, err := s.db.ExecContext(
		ctx,
		`DELETE FROM refresh_sessions
		 WHERE expires_at < $1`,
		now,
	)
	if err != nil {
		return 0, fmt.Errorf("delete expired refresh sessions: %w", err)
	}
	affected, err := result.RowsAffected()
	if err != nil {
		return 0, fmt.Errorf("expired refresh sessions rows affected: %w", err)
	}
	return affected, nil
}
