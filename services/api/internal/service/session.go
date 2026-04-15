package service

import (
	"strings"
	"time"

	"memory-safe-english/services/api/internal/domain"
	"memory-safe-english/services/api/internal/repository"
)

type SessionService struct {
	sessions repository.SessionRepository
	users    repository.UserRepository
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

func NewSessionService(users repository.UserRepository, sessions repository.SessionRepository) SessionService {
	return SessionService{
		sessions: sessions,
		users:    users,
	}
}

func (s SessionService) Start(input StartSessionInput) (domain.Session, error) {
	if strings.TrimSpace(input.UserID) == "" {
		return domain.Session{}, domain.ErrUnauthorized
	}
	if strings.TrimSpace(input.Mode) == "" {
		return domain.Session{}, domain.ErrInvalidInput
	}
	if _, err := s.users.GetUser(strings.TrimSpace(input.UserID)); err != nil {
		return domain.Session{}, err
	}
	return s.sessions.StartSession(strings.TrimSpace(input.UserID), strings.TrimSpace(input.Mode), strings.TrimSpace(input.ContentID))
}

func (s SessionService) Complete(sessionID string) (domain.Session, error) {
	if strings.TrimSpace(sessionID) == "" {
		return domain.Session{}, domain.ErrInvalidInput
	}
	return s.sessions.CompleteSession(strings.TrimSpace(sessionID))
}

func (s SessionService) AddEvent(input AddEventInput) (domain.EventLog, error) {
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
		strings.TrimSpace(input.UserID),
		strings.TrimSpace(input.SessionID),
		strings.TrimSpace(input.EventType),
		input.Payload,
		occurredAt,
	)
}
