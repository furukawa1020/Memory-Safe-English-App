package repository

import (
	"context"
	"time"

	"memory-safe-english/services/api/internal/domain"
)

type UserRepository interface {
	CreateUser(ctx context.Context, email, displayName, authProvider string) (domain.User, error)
	GetUser(ctx context.Context, userID string) (domain.User, error)
}

type SessionRepository interface {
	StartSession(ctx context.Context, userID, mode, contentID string) (domain.Session, error)
	CompleteSession(ctx context.Context, sessionID string) (domain.Session, error)
	GetSession(ctx context.Context, sessionID string) (domain.Session, error)
	AddEvent(ctx context.Context, userID, sessionID, eventType string, payload map[string]any, occurredAt time.Time) (domain.EventLog, error)
}
