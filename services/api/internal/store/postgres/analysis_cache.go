package postgres

import (
	"context"
	"database/sql"
	"encoding/json"
	"errors"
	"fmt"
	"time"

	"memory-safe-english/services/api/internal/domain"
)

func (s *Store) GetChunkingResult(ctx context.Context, contentID string) (domain.ChunkingResult, error) {
	return loadAnalysisResult[domain.ChunkingResult](ctx, s.db, contentID, "chunking_result")
}

func (s *Store) SaveChunkingResult(ctx context.Context, contentID string, result domain.ChunkingResult) error {
	return saveAnalysisResult(ctx, s.db, contentID, "chunking_result", result)
}

func (s *Store) DeleteChunkingResult(ctx context.Context, contentID string) error {
	return deleteAnalysisField(ctx, s.db, contentID, "chunking_result")
}

func (s *Store) GetSkeletonResult(ctx context.Context, contentID string) (domain.SkeletonResult, error) {
	return loadAnalysisResult[domain.SkeletonResult](ctx, s.db, contentID, "skeleton_result")
}

func (s *Store) SaveSkeletonResult(ctx context.Context, contentID string, result domain.SkeletonResult) error {
	return saveAnalysisResult(ctx, s.db, contentID, "skeleton_result", result)
}

func (s *Store) DeleteSkeletonResult(ctx context.Context, contentID string) error {
	return deleteAnalysisField(ctx, s.db, contentID, "skeleton_result")
}

func loadAnalysisResult[T any](ctx context.Context, db *sql.DB, contentID string, field string) (T, error) {
	var zero T
	query := fmt.Sprintf(`SELECT %s FROM content_analysis_cache WHERE content_id = $1 AND %s IS NOT NULL`, field, field)
	row := db.QueryRowContext(ctx, query, contentID)

	var raw []byte
	if err := row.Scan(&raw); err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return zero, domain.ErrNotFound
		}
		return zero, fmt.Errorf("load %s: %w", field, err)
	}
	if err := json.Unmarshal(raw, &zero); err != nil {
		return zero, fmt.Errorf("decode %s: %w", field, err)
	}
	return zero, nil
}

func saveAnalysisResult(ctx context.Context, db *sql.DB, contentID string, field string, payload any) error {
	raw, err := json.Marshal(payload)
	if err != nil {
		return fmt.Errorf("encode %s: %w", field, err)
	}

	query := fmt.Sprintf(
		`INSERT INTO content_analysis_cache (content_id, %s, updated_at)
		 VALUES ($1, $2, $3)
		 ON CONFLICT (content_id)
		 DO UPDATE SET %s = EXCLUDED.%s, updated_at = EXCLUDED.updated_at`,
		field,
		field,
		field,
	)
	if _, err := db.ExecContext(ctx, query, contentID, raw, time.Now().UTC()); err != nil {
		return mapError(err)
	}
	return nil
}

func deleteAnalysisField(ctx context.Context, db *sql.DB, contentID string, field string) error {
	query := fmt.Sprintf(
		`UPDATE content_analysis_cache
		 SET %s = NULL, updated_at = $2
		 WHERE content_id = $1`,
		field,
	)
	if _, err := db.ExecContext(ctx, query, contentID, time.Now().UTC()); err != nil {
		return mapError(err)
	}
	return nil
}
