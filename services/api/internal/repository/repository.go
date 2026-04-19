package repository

import (
	"context"
	"time"

	"memory-safe-english/services/api/internal/domain"
)

type UserRepository interface {
	GetUser(ctx context.Context, userID string) (domain.User, error)
}

type NewAuthUser struct {
	Email        string
	DisplayName  string
	AuthProvider string
	PasswordHash string
}

type AuthRepository interface {
	CreateUserWithPassword(ctx context.Context, input NewAuthUser) (domain.User, error)
	FindUserByEmail(ctx context.Context, email string) (domain.User, string, error)
}

type SessionRepository interface {
	StartSession(ctx context.Context, userID, mode, contentID string) (domain.Session, error)
	CompleteSession(ctx context.Context, sessionID string) (domain.Session, error)
	GetSession(ctx context.Context, sessionID string) (domain.Session, error)
	AddEvent(ctx context.Context, userID, sessionID, eventType string, payload map[string]any, occurredAt time.Time) (domain.EventLog, error)
}

type ContentFilter struct {
	ContentType string
	Level       string
	Topic       string
	Language    string
}

type ContentRepository interface {
	ListContents(ctx context.Context, filter ContentFilter) ([]domain.Content, error)
	GetContent(ctx context.Context, contentID string) (domain.Content, error)
	CreateContent(ctx context.Context, content domain.Content) (domain.Content, error)
	UpdateContent(ctx context.Context, content domain.Content) (domain.Content, error)
	GetChunkingResult(ctx context.Context, contentID string) (domain.ChunkingResult, error)
	SaveChunkingResult(ctx context.Context, contentID string, result domain.ChunkingResult) error
	DeleteChunkingResult(ctx context.Context, contentID string) error
}
