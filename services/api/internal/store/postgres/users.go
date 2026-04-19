package postgres

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"time"

	"memory-safe-english/services/api/internal/domain"
	"memory-safe-english/services/api/internal/repository"
)

func (s *Store) CreateUserWithPassword(ctx context.Context, input repository.NewAuthUser) (domain.User, error) {
	userID := newID("usr")
	now := time.Now().UTC()

	tx, err := s.db.BeginTx(ctx, nil)
	if err != nil {
		return domain.User{}, fmt.Errorf("begin create user tx: %w", err)
	}
	defer tx.Rollback()

	if _, err := tx.ExecContext(
		ctx,
		`INSERT INTO users (id, email, password_hash, auth_provider, subscription_status, created_at, updated_at)
		 VALUES ($1, $2, $3, $4, 'free', $5, $5)`,
		userID,
		input.Email,
		input.PasswordHash,
		input.AuthProvider,
		now,
	); err != nil {
		return domain.User{}, mapError(err)
	}

	if _, err := tx.ExecContext(
		ctx,
		`INSERT INTO user_profiles (user_id, display_name, created_at, updated_at)
		 VALUES ($1, $2, $3, $3)`,
		userID,
		input.DisplayName,
		now,
	); err != nil {
		return domain.User{}, mapError(err)
	}

	if err := tx.Commit(); err != nil {
		return domain.User{}, fmt.Errorf("commit create user tx: %w", err)
	}

	return domain.User{
		ID:                 userID,
		Email:              input.Email,
		DisplayName:        input.DisplayName,
		AuthProvider:       input.AuthProvider,
		SubscriptionStatus: "free",
		CreatedAt:          now,
	}, nil
}

func (s *Store) FindUserByEmail(ctx context.Context, email string) (domain.User, string, error) {
	row := s.db.QueryRowContext(
		ctx,
		`SELECT u.id, u.email, p.display_name, u.auth_provider, u.subscription_status, u.password_hash, u.created_at
		 FROM users u
		 JOIN user_profiles p ON p.user_id = u.id
		 WHERE u.email = $1`,
		email,
	)

	var user domain.User
	var passwordHash string
	if err := row.Scan(
		&user.ID,
		&user.Email,
		&user.DisplayName,
		&user.AuthProvider,
		&user.SubscriptionStatus,
		&passwordHash,
		&user.CreatedAt,
	); err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return domain.User{}, "", domain.ErrNotFound
		}
		return domain.User{}, "", fmt.Errorf("find user by email: %w", err)
	}
	return user, passwordHash, nil
}

func (s *Store) GetUser(ctx context.Context, userID string) (domain.User, error) {
	row := s.db.QueryRowContext(
		ctx,
		`SELECT u.id, u.email, p.display_name, u.auth_provider, u.subscription_status, u.created_at
		 FROM users u
		 JOIN user_profiles p ON p.user_id = u.id
		 WHERE u.id = $1`,
		userID,
	)

	var user domain.User
	if err := row.Scan(
		&user.ID,
		&user.Email,
		&user.DisplayName,
		&user.AuthProvider,
		&user.SubscriptionStatus,
		&user.CreatedAt,
	); err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return domain.User{}, domain.ErrNotFound
		}
		return domain.User{}, fmt.Errorf("get user: %w", err)
	}
	return user, nil
}
