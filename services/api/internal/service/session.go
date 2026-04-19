package service

import (
	"context"
	"strings"
	"time"

	"memory-safe-english/services/api/internal/domain"
	"memory-safe-english/services/api/internal/repository"
)

type SessionService struct {
	sessions SessionStore
	users    UserReader
}

type SessionStore interface {
	repository.SessionRepository
}

type StartSessionInput struct {
	UserID    string
	Mode      string
	ContentID string
}

type AddEventInput struct {
	UserID     string
	SessionID  string
	EventType  string
	Payload    map[string]any
	OccurredAt time.Time
}

func NewSessionService(users UserReader, sessions SessionStore) SessionService {
	return SessionService{
		sessions: sessions,
		users:    users,
	}
}

func (s SessionService) Start(ctx context.Context, input StartSessionInput) (domain.Session, error) {
	if strings.TrimSpace(input.UserID) == "" {
		return domain.Session{}, domain.ErrUnauthorized
	}
	if strings.TrimSpace(input.Mode) == "" {
		return domain.Session{}, domain.ErrInvalidInput
	}
	if _, err := s.users.GetUser(ctx, strings.TrimSpace(input.UserID)); err != nil {
		return domain.Session{}, err
	}
	return s.sessions.StartSession(ctx, strings.TrimSpace(input.UserID), strings.TrimSpace(input.Mode), strings.TrimSpace(input.ContentID))
}

func (s SessionService) Complete(ctx context.Context, sessionID string) (domain.Session, error) {
	if strings.TrimSpace(sessionID) == "" {
		return domain.Session{}, domain.ErrInvalidInput
	}
	return s.sessions.CompleteSession(ctx, strings.TrimSpace(sessionID))
}

func (s SessionService) AddEvent(ctx context.Context, input AddEventInput) (domain.EventLog, error) {
	if strings.TrimSpace(input.UserID) == "" {
		return domain.EventLog{}, domain.ErrUnauthorized
	}
	if strings.TrimSpace(input.SessionID) == "" || strings.TrimSpace(input.EventType) == "" {
		return domain.EventLog{}, domain.ErrInvalidInput
	}
	occurredAt := input.OccurredAt
	if occurredAt.IsZero() {
		occurredAt = time.Now().UTC()
	}
	return s.sessions.AddEvent(
		ctx,
		strings.TrimSpace(input.UserID),
		strings.TrimSpace(input.SessionID),
		strings.TrimSpace(input.EventType),
		input.Payload,
		occurredAt,
	)
}
