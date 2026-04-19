package postgres

import (
	"context"
	"database/sql"
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"time"

	"memory-safe-english/services/api/internal/domain"
	"memory-safe-english/services/api/internal/repository"

	"github.com/jackc/pgconn"
	_ "github.com/jackc/pgx/v5/stdlib"
)

type Store struct {
	db *sql.DB
}

func NewStore(databaseURL string) (*Store, error) {
	db, err := sql.Open("pgx", databaseURL)
	if err != nil {
		return nil, fmt.Errorf("open postgres: %w", err)
	}

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	if err := db.PingContext(ctx); err != nil {
		_ = db.Close()
		return nil, fmt.Errorf("ping postgres: %w", err)
	}

	return &Store{db: db}, nil
}

func (s *Store) Close(_ context.Context) error {
	return s.db.Close()
}

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

func (s *Store) StartSession(ctx context.Context, userID, mode, contentID string) (domain.Session, error) {
	session := domain.Session{
		ID:              newID("ses"),
		UserID:          userID,
		Mode:            mode,
		ContentID:       nullIfEmpty(contentID),
		StartedAt:       time.Now().UTC(),
		CompletionState: "started",
	}
	if _, err := s.db.ExecContext(
		ctx,
		`INSERT INTO sessions (id, user_id, mode, content_id, started_at, completion_state)
		 VALUES ($1, $2, $3, NULLIF($4, ''), $5, $6)`,
		session.ID,
		session.UserID,
		session.Mode,
		session.ContentID,
		session.StartedAt,
		session.CompletionState,
	); err != nil {
		return domain.Session{}, mapError(err)
	}
	return session, nil
}

func (s *Store) CompleteSession(ctx context.Context, sessionID string) (domain.Session, error) {
	row := s.db.QueryRowContext(
		ctx,
		`UPDATE sessions
		 SET completed_at = $2, completion_state = 'completed'
		 WHERE id = $1
		 RETURNING id, user_id, mode, COALESCE(content_id, ''), started_at, COALESCE(completed_at, TIMESTAMPTZ '0001-01-01'), completion_state`,
		sessionID,
		time.Now().UTC(),
	)
	return scanSession(row)
}

func (s *Store) GetSession(ctx context.Context, sessionID string) (domain.Session, error) {
	row := s.db.QueryRowContext(
		ctx,
		`SELECT id, user_id, mode, COALESCE(content_id, ''), started_at, COALESCE(completed_at, TIMESTAMPTZ '0001-01-01'), completion_state
		 FROM sessions
		 WHERE id = $1`,
		sessionID,
	)
	return scanSession(row)
}

func (s *Store) AddEvent(ctx context.Context, userID, sessionID, eventType string, payload map[string]any, occurredAt time.Time) (domain.EventLog, error) {
	eventID := newID("evt")
	payloadJSON, err := json.Marshal(payload)
	if err != nil {
		return domain.EventLog{}, fmt.Errorf("marshal event payload: %w", err)
	}

	event := domain.EventLog{
		ID:         eventID,
		UserID:     userID,
		SessionID:  sessionID,
		EventType:  eventType,
		Payload:    payload,
		OccurredAt: occurredAt,
		CreatedAt:  time.Now().UTC(),
	}
	if _, err := s.db.ExecContext(
		ctx,
		`INSERT INTO event_logs (id, user_id, session_id, event_type, payload_json, occurred_at, created_at)
		 VALUES ($1, $2, $3, $4, $5, $6, $7)`,
		event.ID,
		event.UserID,
		event.SessionID,
		event.EventType,
		payloadJSON,
		event.OccurredAt,
		event.CreatedAt,
	); err != nil {
		return domain.EventLog{}, mapError(err)
	}
	return event, nil
}

func (s *Store) ListContents(ctx context.Context, filter repository.ContentFilter) ([]domain.Content, error) {
	query := `SELECT id, title, content_type, level, topic, language, raw_text, COALESCE(summary_text, ''), created_at, updated_at
	          FROM contents`
	var clauses []string
	var args []any
	if filter.ContentType != "" {
		clauses = append(clauses, fmt.Sprintf("content_type = $%d", len(args)+1))
		args = append(args, filter.ContentType)
	}
	if filter.Level != "" {
		clauses = append(clauses, fmt.Sprintf("level = $%d", len(args)+1))
		args = append(args, filter.Level)
	}
	if filter.Topic != "" {
		clauses = append(clauses, fmt.Sprintf("topic = $%d", len(args)+1))
		args = append(args, filter.Topic)
	}
	if filter.Language != "" {
		clauses = append(clauses, fmt.Sprintf("language = $%d", len(args)+1))
		args = append(args, filter.Language)
	}
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
