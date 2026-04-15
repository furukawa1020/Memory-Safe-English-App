package repository

import (
	"time"

	"memory-safe-english/services/api/internal/domain"
)

type UserRepository interface {
	CreateUser(email, displayName, authProvider string) (domain.User, error)
	GetUser(userID string) (domain.User, error)
}

type SessionRepository interface {
	StartSession(userID, mode, contentID string) (domain.Session, error)
	CompleteSession(sessionID string) (domain.Session, error)
	GetSession(sessionID string) (domain.Session, error)
	AddEvent(userID, sessionID, eventType string, payload map[string]any, occurredAt time.Time) (domain.EventLog, error)
}
