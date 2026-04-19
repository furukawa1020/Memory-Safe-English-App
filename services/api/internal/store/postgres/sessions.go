package postgres

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"memory-safe-english/services/api/internal/domain"
)

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
	payloadJSON, err := json.Marshal(payload)
	if err != nil {
		return domain.EventLog{}, fmt.Errorf("marshal event payload: %w", err)
	}

	event := domain.EventLog{
		ID:         newID("evt"),
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
