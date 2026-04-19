package memory

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"sort"
	"sync"
	"time"

	"memory-safe-english/services/api/internal/domain"
	"memory-safe-english/services/api/internal/repository"
)

type Store struct {
	mu             sync.RWMutex
	users          map[string]domain.User
	usersByEmail   map[string]string
	passwordHashes map[string]string
	sessions       map[string]domain.Session
	events         map[string][]domain.EventLog
	contents       map[string]domain.Content
	contentChunks  map[string]domain.ChunkingResult
}

func NewStore() *Store {
	store := &Store{
		users:          make(map[string]domain.User),
		usersByEmail:   make(map[string]string),
		passwordHashes: make(map[string]string),
		sessions:       make(map[string]domain.Session),
		events:         make(map[string][]domain.EventLog),
		contents:       make(map[string]domain.Content),
		contentChunks:  make(map[string]domain.ChunkingResult),
	}
	store.seedContents()
	return store
}

func (s *Store) CreateUserWithPassword(_ context.Context, input repository.NewAuthUser) (domain.User, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	if _, exists := s.usersByEmail[input.Email]; exists {
		return domain.User{}, domain.ErrConflict
	}

	now := time.Now().UTC()
	user := domain.User{
		ID:                 newID("usr"),
		Email:              input.Email,
		DisplayName:        input.DisplayName,
		AuthProvider:       input.AuthProvider,
		SubscriptionStatus: "free",
		CreatedAt:          now,
	}
	s.users[user.ID] = user
	s.usersByEmail[input.Email] = user.ID
	s.passwordHashes[user.ID] = input.PasswordHash
	return user, nil
}

func (s *Store) GetUser(_ context.Context, userID string) (domain.User, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	user, ok := s.users[userID]
	if !ok {
		return domain.User{}, domain.ErrNotFound
	}
	return user, nil
}

func (s *Store) FindUserByEmail(_ context.Context, email string) (domain.User, string, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	userID, ok := s.usersByEmail[email]
	if !ok {
		return domain.User{}, "", domain.ErrNotFound
	}

	user, ok := s.users[userID]
	if !ok {
		return domain.User{}, "", domain.ErrNotFound
	}

	passwordHash, ok := s.passwordHashes[userID]
	if !ok {
		return domain.User{}, "", domain.ErrNotFound
	}

	return user, passwordHash, nil
}

func (s *Store) StartSession(_ context.Context, userID, mode, contentID string) (domain.Session, error) {
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

func (s *Store) CompleteSession(_ context.Context, sessionID string) (domain.Session, error) {
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

func (s *Store) GetSession(_ context.Context, sessionID string) (domain.Session, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	session, ok := s.sessions[sessionID]
	if !ok {
		return domain.Session{}, domain.ErrNotFound
	}
	return session, nil
}

func (s *Store) AddEvent(_ context.Context, userID, sessionID, eventType string, payload map[string]any, occurredAt time.Time) (domain.EventLog, error) {
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

func (s *Store) ListContents(_ context.Context, filter repository.ContentFilter) ([]domain.Content, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	result := make([]domain.Content, 0, len(s.contents))
	for _, content := range s.contents {
		if filter.ContentType != "" && content.ContentType != filter.ContentType {
			continue
		}
		if filter.Level != "" && content.Level != filter.Level {
			continue
		}
		if filter.Topic != "" && content.Topic != filter.Topic {
			continue
		}
		if filter.Language != "" && content.Language != filter.Language {
			continue
		}
		result = append(result, content)
	}
	sort.Slice(result, func(i, j int) bool {
		return result[i].CreatedAt.Before(result[j].CreatedAt)
	})
	return result, nil
}

func (s *Store) GetContent(_ context.Context, contentID string) (domain.Content, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	content, ok := s.contents[contentID]
	if !ok {
		return domain.Content{}, domain.ErrNotFound
	}
	return content, nil
}

func (s *Store) CreateContent(_ context.Context, content domain.Content) (domain.Content, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	if _, exists := s.contents[content.ID]; exists {
		return domain.Content{}, domain.ErrConflict
	}
	s.contents[content.ID] = content
	return content, nil
}

func (s *Store) UpdateContent(_ context.Context, content domain.Content) (domain.Content, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	existing, exists := s.contents[content.ID]
	if !exists {
		return domain.Content{}, domain.ErrNotFound
	}
	content.CreatedAt = existing.CreatedAt
	s.contents[content.ID] = content
	return content, nil
}

func (s *Store) GetChunkingResult(_ context.Context, contentID string) (domain.ChunkingResult, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	result, ok := s.contentChunks[contentID]
	if !ok {
		return domain.ChunkingResult{}, domain.ErrNotFound
	}
	return result, nil
}

func (s *Store) SaveChunkingResult(_ context.Context, contentID string, result domain.ChunkingResult) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	if _, ok := s.contents[contentID]; !ok {
		return domain.ErrNotFound
	}
	s.contentChunks[contentID] = result
	return nil
}

func (s *Store) DeleteChunkingResult(_ context.Context, contentID string) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	delete(s.contentChunks, contentID)
	return nil
}

func (s *Store) seedContents() {
	now := time.Now().UTC()
	items := []domain.Content{
		{
			ID:          "cnt_self_intro_001",
			Title:       "Self Introduction",
			ContentType: "reading",
			Level:       "intro",
			Topic:       "self_intro",
			Language:    "en",
			RawText:     "Hello, my name is Aki, and I study human computer interaction at university.",
			SummaryText: "Simple self introduction",
			CreatedAt:   now,
			UpdatedAt:   now,
		},
		{
			ID:          "cnt_research_001",
			Title:       "Research Presentation Opening",
			ContentType: "reading",
			Level:       "intermediate",
			Topic:       "research",
			Language:    "en",
			RawText:     "In this study, we propose a memory safe interface that reduces cognitive overload during English reading.",
			SummaryText: "Research opening sentence",
			CreatedAt:   now,
			UpdatedAt:   now,
		},
	}
	for _, item := range items {
		s.contents[item.ID] = item
	}
}
