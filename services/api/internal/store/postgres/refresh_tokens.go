package postgres

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"time"

	"memory-safe-english/services/api/internal/domain"
)

func (s *Store) CreateRefreshSession(ctx context.Context, family domain.RefreshTokenFamily, session domain.RefreshSession) error {
	tx, err := s.db.BeginTx(ctx, nil)
	if err != nil {
		return fmt.Errorf("begin create refresh session tx: %w", err)
	}
	defer tx.Rollback()

	if _, err := tx.ExecContext(
		ctx,
		`INSERT INTO refresh_token_families (id, user_id, created_at)
		 VALUES ($1, $2, $3)`,
		family.ID,
		family.UserID,
		family.CreatedAt,
	); err != nil {
		return mapError(err)
	}

	if _, err := tx.ExecContext(
		ctx,
		`INSERT INTO refresh_sessions (id, family_id, user_id, token_hash, expires_at, created_at)
		 VALUES ($1, $2, $3, $4, $5, $6)`,
		session.ID,
		session.FamilyID,
		session.UserID,
		session.TokenHash,
		session.ExpiresAt,
		session.CreatedAt,
	); err != nil {
		return mapError(err)
	}

	if err := tx.Commit(); err != nil {
		return fmt.Errorf("commit create refresh session tx: %w", err)
	}
	return nil
}

func (s *Store) GetRefreshSession(ctx context.Context, tokenID string) (domain.RefreshSession, error) {
	row := s.db.QueryRowContext(
		ctx,
		`SELECT id, family_id, user_id, token_hash, expires_at, created_at, revoked_at, COALESCE(replaced_by_token_id, '')
		 FROM refresh_sessions
		 WHERE id = $1`,
		tokenID,
	)

	var session domain.RefreshSession
	if err := row.Scan(
		&session.ID,
		&session.FamilyID,
		&session.UserID,
		&session.TokenHash,
		&session.ExpiresAt,
		&session.CreatedAt,
		&session.RevokedAt,
		&session.ReplacedByTokenID,
	); err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return domain.RefreshSession{}, domain.ErrNotFound
		}
		return domain.RefreshSession{}, fmt.Errorf("get refresh session: %w", err)
	}
	return session, nil
}

func (s *Store) GetRefreshFamily(ctx context.Context, familyID string) (domain.RefreshTokenFamily, error) {
	row := s.db.QueryRowContext(
		ctx,
		`SELECT id, user_id, created_at, revoked_at
		 FROM refresh_token_families
		 WHERE id = $1`,
		familyID,
	)

	var family domain.RefreshTokenFamily
	if err := row.Scan(
		&family.ID,
		&family.UserID,
		&family.CreatedAt,
		&family.RevokedAt,
	); err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return domain.RefreshTokenFamily{}, domain.ErrNotFound
		}
		return domain.RefreshTokenFamily{}, fmt.Errorf("get refresh family: %w", err)
	}
	return family, nil
}

func (s *Store) RotateRefreshSession(ctx context.Context, currentTokenID, currentTokenHash string, nextSession domain.RefreshSession) error {
	tx, err := s.db.BeginTx(ctx, &sql.TxOptions{})
	if err != nil {
		return fmt.Errorf("begin rotate refresh session tx: %w", err)
	}
	defer tx.Rollback()

	var current domain.RefreshSession
	row := tx.QueryRowContext(
		ctx,
		`SELECT id, family_id, user_id, token_hash, expires_at, created_at, revoked_at, COALESCE(replaced_by_token_id, '')
		 FROM refresh_sessions
		 WHERE id = $1
		 FOR UPDATE`,
		currentTokenID,
	)
	if err := row.Scan(
		&current.ID,
		&current.FamilyID,
		&current.UserID,
		&current.TokenHash,
		&current.ExpiresAt,
		&current.CreatedAt,
		&current.RevokedAt,
		&current.ReplacedByTokenID,
	); err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return domain.ErrNotFound
		}
		return fmt.Errorf("load refresh session for rotation: %w", err)
	}

	var familyRevokedAt sql.NullTime
	if err := tx.QueryRowContext(
		ctx,
		`SELECT revoked_at
		 FROM refresh_token_families
		 WHERE id = $1
		 FOR UPDATE`,
		current.FamilyID,
	).Scan(&familyRevokedAt); err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return domain.ErrNotFound
		}
		return fmt.Errorf("load refresh family for rotation: %w", err)
	}

	if familyRevokedAt.Valid || current.RevokedAt != nil {
		return domain.ErrConflict
	}
	if current.TokenHash != currentTokenHash {
		return domain.ErrConflict
	}
	if time.Now().UTC().After(current.ExpiresAt) {
		return domain.ErrExpired
	}

	now := time.Now().UTC()
	if _, err := tx.ExecContext(
		ctx,
		`UPDATE refresh_sessions
		 SET revoked_at = $2, replaced_by_token_id = $3
		 WHERE id = $1`,
		currentTokenID,
		now,
		nextSession.ID,
	); err != nil {
		return fmt.Errorf("revoke current refresh session: %w", err)
	}

	if _, err := tx.ExecContext(
		ctx,
		`INSERT INTO refresh_sessions (id, family_id, user_id, token_hash, expires_at, created_at)
		 VALUES ($1, $2, $3, $4, $5, $6)`,
		nextSession.ID,
		nextSession.FamilyID,
		nextSession.UserID,
		nextSession.TokenHash,
		nextSession.ExpiresAt,
		nextSession.CreatedAt,
	); err != nil {
		return mapError(err)
	}

	if err := tx.Commit(); err != nil {
		return fmt.Errorf("commit rotate refresh session tx: %w", err)
	}
	return nil
}

func (s *Store) RevokeRefreshFamily(ctx context.Context, familyID string) error {
	tx, err := s.db.BeginTx(ctx, nil)
	if err != nil {
		return fmt.Errorf("begin revoke refresh family tx: %w", err)
	}
	defer tx.Rollback()

	now := time.Now().UTC()
	result, err := tx.ExecContext(
		ctx,
		`UPDATE refresh_token_families
		 SET revoked_at = COALESCE(revoked_at, $2)
		 WHERE id = $1`,
		familyID,
		now,
	)
	if err != nil {
		return fmt.Errorf("revoke refresh family: %w", err)
	}
	affected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("refresh family rows affected: %w", err)
	}
	if affected == 0 {
		return domain.ErrNotFound
	}

	if _, err := tx.ExecContext(
		ctx,
		`UPDATE refresh_sessions
		 SET revoked_at = COALESCE(revoked_at, $2)
		 WHERE family_id = $1`,
		familyID,
		now,
	); err != nil {
		return fmt.Errorf("revoke refresh sessions: %w", err)
	}

	if err := tx.Commit(); err != nil {
		return fmt.Errorf("commit revoke refresh family tx: %w", err)
	}
	return nil
}
