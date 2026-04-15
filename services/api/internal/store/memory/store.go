package memory

import (
	"crypto/rand"
	"encoding/hex"
	"sync"
	"time"

	"memory-safe-english/services/api/internal/domain"
)

type Store struct {
	mu       sync.RWMutex
	users    map[string]domain.User
	sessions map[string]domain.Session
	events   map[string][]domain.EventLog
}

func NewStore() *Store {
	return &Store{
		users:    make(map[string]domain.User),
		sessions: make(map[string]domain.Session),
		events:   make(map[string][]domain.EventLog),
	}
}

func (s *Store) CreateUser(email, displayName, authProvider string) domain.User {
	s.mu.Lock()
	defer s.mu.Unlock()

	now := time.Now().UTC()
	user := domain.User{
		ID:                 newID("usr"),
		Email:              email,
		DisplayName:        displayName,
		AuthProvider:       authProvider,
		SubscriptionStatus: "free",
		CreatedAt:          now,
	}
	s.users[user.ID] = user
	return user
}

func (s *Store) CreateUser(email, displayName, authProvider string) (domain.User, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	now := time.Now().UTC()
	user := domain.User{
		ID:                 newID("usr"),
		Email:              email,
		DisplayName:        displayName,
		AuthProvider:       authProvider,
		SubscriptionStatus: "free",
		CreatedAt:          now,
	}
	s.users[user.ID] = user
	return user, nil
}

func (s *Store) GetUser(userID string) (domain.User, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	user, ok := s.users[userID]
	if !ok {
		return domain.User{}, domain.ErrNotFound
	}
	return user, nil
}

func (s *Store) StartSession(userID, mode, contentID string) (domain.Session, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	if _, ok := s.users[userID]; !ok {
		return domain.Session{}, domain.ErrNotFound
	}

	session := domain.Session{
		ID:              newID("ses"),
		UserID:          userID,
		Mode:            mode,
		ContentID:       contentID,
		StartedAt:       time.Now().UTC(),
		CompletionState: "started",
	}
	s.sessions[session.ID] = session
	return session, nil
}

func (s *Store) CompleteSession(sessionID string) (domain.Session, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	session, ok := s.sessions[sessionID]
	if !ok {
		return domain.Session{}, domain.ErrNotFound
	}

	session.CompletedAt = time.Now().UTC()
	session.CompletionState = "completed"
	s.sessions[sessionID] = session
	return session, nil
}

func (s *Store) GetSession(sessionID string) (domain.Session, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	session, ok := s.sessions[sessionID]
	if !ok {
		return domain.Session{}, domain.ErrNotFound
	}
	return session, nil
}

func (s *Store) AddEvent(userID, sessionID, eventType string, payload map[string]any, occurredAt time.Time) (domain.EventLog, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	session, ok := s.sessions[sessionID]
	if !ok {
		return domain.EventLog{}, domain.ErrNotFound
	}
	if session.UserID != userID {
		return domain.EventLog{}, domain.ErrNotFound
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
	s.events[sessionID] = append(s.events[sessionID], event)
	return event, nil
}

func newID(prefix string) string {
	buf := make([]byte, 6)
	if _, err := rand.Read(buf); err != nil {
		return prefix + "_" + time.Now().UTC().Format("20060102150405")
	}
	return prefix + "_" + hex.EncodeToString(buf)
}
