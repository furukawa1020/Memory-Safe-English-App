package postgres

import (
	"context"
	"fmt"
	"strings"

	"memory-safe-english/services/api/internal/domain"
	"memory-safe-english/services/api/internal/repository"
)

func (s *Store) ListContents(ctx context.Context, filter repository.ContentFilter) ([]domain.Content, error) {
	query := `SELECT id, title, content_type, level, topic, language, raw_text, COALESCE(summary_text, ''), created_at, updated_at
	          FROM contents`

	clauses, args := buildContentFilter(filter)
	if len(clauses) > 0 {
		query += " WHERE " + strings.Join(clauses, " AND ")
	}
	query += " ORDER BY created_at ASC"

	rows, err := s.db.QueryContext(ctx, query, args...)
	if err != nil {
		return nil, fmt.Errorf("list contents: %w", err)
	}
	defer rows.Close()

	var contents []domain.Content
	for rows.Next() {
		content, err := scanContent(rows)
		if err != nil {
			return nil, err
		}
		contents = append(contents, content)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate contents: %w", err)
	}
	return contents, nil
}

func (s *Store) GetContent(ctx context.Context, contentID string) (domain.Content, error) {
	row := s.db.QueryRowContext(
		ctx,
		`SELECT id, title, content_type, level, topic, language, raw_text, COALESCE(summary_text, ''), created_at, updated_at
		 FROM contents
		 WHERE id = $1`,
		contentID,
	)
	return scanContent(row)
}

func (s *Store) CreateContent(ctx context.Context, content domain.Content) (domain.Content, error) {
	if _, err := s.db.ExecContext(
		ctx,
		`INSERT INTO contents (id, title, content_type, level, topic, language, raw_text, summary_text, created_at, updated_at)
		 VALUES ($1, $2, $3, $4, $5, $6, $7, NULLIF($8, ''), $9, $10)`,
		content.ID,
		content.Title,
		content.ContentType,
		content.Level,
		content.Topic,
		content.Language,
		content.RawText,
		content.SummaryText,
		content.CreatedAt,
		content.UpdatedAt,
	); err != nil {
		return domain.Content{}, mapError(err)
	}
	return content, nil
}

func (s *Store) UpdateContent(ctx context.Context, content domain.Content) (domain.Content, error) {
	result, err := s.db.ExecContext(
		ctx,
		`UPDATE contents
		 SET title = $2, content_type = $3, level = $4, topic = $5, language = $6, raw_text = $7, summary_text = NULLIF($8, ''), updated_at = $9
		 WHERE id = $1`,
		content.ID,
		content.Title,
		content.ContentType,
		content.Level,
		content.Topic,
		content.Language,
		content.RawText,
		content.SummaryText,
		content.UpdatedAt,
	)
	if err != nil {
		return domain.Content{}, mapError(err)
	}

	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return domain.Content{}, fmt.Errorf("update content rows affected: %w", err)
	}
	if rowsAffected == 0 {
		return domain.Content{}, domain.ErrNotFound
	}
	return content, nil
}

func buildContentFilter(filter repository.ContentFilter) ([]string, []any) {
	var clauses []string
	var args []any

	appendFilter := func(column, value string) {
		if value == "" {
			return
		}
		clauses = append(clauses, fmt.Sprintf("%s = $%d", column, len(args)+1))
		args = append(args, value)
	}

	appendFilter("content_type", filter.ContentType)
	appendFilter("level", filter.Level)
	appendFilter("topic", filter.Topic)
	appendFilter("language", filter.Language)

	return clauses, args
}
