package postgres

import (
	"database/sql"
	"errors"
	"fmt"

	"memory-safe-english/services/api/internal/domain"
)

type scanner interface {
	Scan(dest ...any) error
}

func scanContent(row scanner) (domain.Content, error) {
	var content domain.Content
	if err := row.Scan(
		&content.ID,
		&content.Title,
		&content.ContentType,
		&content.Level,
		&content.Topic,
		&content.Language,
		&content.RawText,
		&content.SummaryText,
		&content.CreatedAt,
		&content.UpdatedAt,
	); err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return domain.Content{}, domain.ErrNotFound
		}
		return domain.Content{}, fmt.Errorf("scan content: %w", err)
	}
	return content, nil
}

func scanSession(row scanner) (domain.Session, error) {
	var session domain.Session
	if err := row.Scan(
		&session.ID,
		&session.UserID,
		&session.Mode,
		&session.ContentID,
		&session.StartedAt,
		&session.CompletedAt,
		&session.CompletionState,
	); err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return domain.Session{}, domain.ErrNotFound
		}
		return domain.Session{}, fmt.Errorf("scan session: %w", err)
	}
	return session, nil
}
