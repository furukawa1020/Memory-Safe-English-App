package postgres

import (
	"errors"
	"strings"

	"memory-safe-english/services/api/internal/domain"

	"github.com/jackc/pgconn"
)

func mapError(err error) error {
	var pgErr *pgconn.PgError
	if errors.As(err, &pgErr) {
		switch pgErr.Code {
		case "23505":
			return domain.ErrConflict
		case "23503":
			return domain.ErrNotFound
		}
	}
	return err
}

func nullIfEmpty(value string) string {
	return strings.TrimSpace(value)
}
