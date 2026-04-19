package app

import (
	"context"
	"log"
	"time"
)

type refreshCleanupStore interface {
	DeleteExpiredRefreshSessions(ctx context.Context, now time.Time) (int64, error)
}

func runRefreshCleanupLoop(ctx context.Context, interval time.Duration, store refreshCleanupStore) {
	ticker := time.NewTicker(interval)
	defer ticker.Stop()

	runRefreshCleanupOnce(ctx, store)

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			runRefreshCleanupOnce(ctx, store)
		}
	}
}

func runRefreshCleanupOnce(ctx context.Context, store refreshCleanupStore) {
	cleanupCtx, cancel := context.WithTimeout(ctx, 15*time.Second)
	defer cancel()

	removed, err := store.DeleteExpiredRefreshSessions(cleanupCtx, time.Now().UTC())
	if err != nil {
		log.Printf("component=auth_refresh_cleanup ok=false error=%q", err.Error())
		return
	}
	if removed > 0 {
		log.Printf("component=auth_refresh_cleanup ok=true removed=%d", removed)
	}
}
