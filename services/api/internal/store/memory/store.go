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
	mu              sync.RWMutex
	users           map[string]domain.User
	usersByEmail    map[string]string
	passwordHashes  map[string]string
	refreshFamilies map[string]domain.RefreshTokenFamily
	refreshSessions map[string]domain.RefreshSession
	sessions        map[string]domain.Session
	events          map[string][]domain.EventLog
	contents        map[string]domain.Content
	contentChunks   map[string]domain.ChunkingResult
	contentSkeleton map[string]domain.SkeletonResult
}

func NewStore() *Store {
	store := &Store{
		users:           make(map[string]domain.User),
		usersByEmail:    make(map[string]string),
		passwordHashes:  make(map[string]string),
		refreshFamilies: make(map[string]domain.RefreshTokenFamily),
		refreshSessions: make(map[string]domain.RefreshSession),
		sessions:        make(map[string]domain.Session),
		events:          make(map[string][]domain.EventLog),
		contents:        make(map[string]domain.Content),
		contentChunks:   make(map[string]domain.ChunkingResult),
		contentSkeleton: make(map[string]domain.SkeletonResult),
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

func (s *Store) CreateRefreshSession(_ context.Context, family domain.RefreshTokenFamily, session domain.RefreshSession) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	if _, exists := s.refreshFamilies[family.ID]; exists {
		return domain.ErrConflict
	}
	if _, exists := s.refreshSessions[session.ID]; exists {
		return domain.ErrConflict
	}
	s.refreshFamilies[family.ID] = family
	s.refreshSessions[session.ID] = session
	return nil
}

func (s *Store) GetRefreshSession(_ context.Context, tokenID string) (domain.RefreshSession, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	session, ok := s.refreshSessions[tokenID]
	if !ok {
		return domain.RefreshSession{}, domain.ErrNotFound
	}
	return session, nil
}

func (s *Store) GetRefreshFamily(_ context.Context, familyID string) (domain.RefreshTokenFamily, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	family, ok := s.refreshFamilies[familyID]
	if !ok {
		return domain.RefreshTokenFamily{}, domain.ErrNotFound
	}
	return family, nil
}

func (s *Store) RotateRefreshSession(_ context.Context, currentTokenID, currentTokenHash string, nextSession domain.RefreshSession) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	current, ok := s.refreshSessions[currentTokenID]
	if !ok {
		return domain.ErrNotFound
	}
	family, ok := s.refreshFamilies[current.FamilyID]
	if !ok {
		return domain.ErrNotFound
	}
	if family.RevokedAt != nil || current.RevokedAt != nil {
		return domain.ErrConflict
	}
	if current.TokenHash != currentTokenHash {
		return domain.ErrConflict
	}
	if time.Now().UTC().After(current.ExpiresAt) {
		return domain.ErrExpired
	}
	if _, exists := s.refreshSessions[nextSession.ID]; exists {
		return domain.ErrConflict
	}

	now := time.Now().UTC()
	current.RevokedAt = &now
	current.ReplacedByTokenID = nextSession.ID
	s.refreshSessions[currentTokenID] = current
	s.refreshSessions[nextSession.ID] = nextSession
	return nil
}

func (s *Store) RevokeRefreshFamily(_ context.Context, familyID string) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	family, ok := s.refreshFamilies[familyID]
	if !ok {
		return domain.ErrNotFound
	}
	if family.RevokedAt != nil {
		return nil
	}

	now := time.Now().UTC()
	family.RevokedAt = &now
	s.refreshFamilies[familyID] = family

	for id, session := range s.refreshSessions {
		if session.FamilyID != familyID || session.RevokedAt != nil {
			continue
		}
		session.RevokedAt = &now
		s.refreshSessions[id] = session
	}
	return nil
}

func (s *Store) DeleteExpiredRefreshSessions(_ context.Context, now time.Time) (int64, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	var removed int64
	for id, session := range s.refreshSessions {
		if session.ExpiresAt.After(now) {
			continue
		}
		delete(s.refreshSessions, id)
		removed++
	}
	return removed, nil
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

func (s *Store) GetSkeletonResult(_ context.Context, contentID string) (domain.SkeletonResult, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	result, ok := s.contentSkeleton[contentID]
	if !ok {
		return domain.SkeletonResult{}, domain.ErrNotFound
	}
	return result, nil
}

func (s *Store) SaveSkeletonResult(_ context.Context, contentID string, result domain.SkeletonResult) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	if _, ok := s.contents[contentID]; !ok {
		return domain.ErrNotFound
	}
	s.contentSkeleton[contentID] = result
	return nil
}

func (s *Store) DeleteSkeletonResult(_ context.Context, contentID string) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	delete(s.contentSkeleton, contentID)
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
			ID:          "cnt_self_intro_002",
			Title:       "Short Self Introduction Template",
			ContentType: "speaking_template",
			Level:       "intro",
			Topic:       "self_intro",
			Language:    "en",
			RawText:     "Hello. My name is Hana. I study psychology. Today I want to talk about learning support.",
			SummaryText: "Short self introduction in stable short units",
			CreatedAt:   now,
			UpdatedAt:   now,
		},
		{
			ID:          "cnt_daily_001",
			Title:       "Ordering Coffee",
			ContentType: "reading",
			Level:       "intro",
			Topic:       "daily",
			Language:    "en",
			RawText:     "I would like a small latte, please. If possible, can I get it with oat milk?",
			SummaryText: "Short daily request at a cafe",
			CreatedAt:   now,
			UpdatedAt:   now,
		},
		{
			ID:          "cnt_daily_002",
			Title:       "Daily Listening: Delayed Train",
			ContentType: "listening",
			Level:       "intro",
			Topic:       "daily",
			Language:    "en",
			RawText:     "The train is delayed by ten minutes. Please wait on platform three for the next announcement.",
			SummaryText: "Simple delay announcement with one action point",
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
		{
			ID:          "cnt_research_002",
			Title:       "Research Method Overview",
			ContentType: "reading",
			Level:       "intermediate",
			Topic:       "research",
			Language:    "en",
			RawText:     "We compared the new interface with a standard reader and measured how often participants lost the main point of the sentence.",
			SummaryText: "Method sentence with comparison and measurement",
			CreatedAt:   now,
			UpdatedAt:   now,
		},
		{
			ID:          "cnt_research_003",
			Title:       "Research Result Listening",
			ContentType: "listening",
			Level:       "intermediate",
			Topic:       "research",
			Language:    "en",
			RawText:     "Participants using the new interface reported lower overload and gave more accurate summaries after each reading task.",
			SummaryText: "Result sentence with two outcomes",
			CreatedAt:   now,
			UpdatedAt:   now,
		},
		{
			ID:          "cnt_research_004",
			Title:       "Research Explanation Template",
			ContentType: "speaking_template",
			Level:       "intermediate",
			Topic:       "research",
			Language:    "en",
			RawText:     "Our topic is reading overload. We built a safer interface. The main result is lower cognitive strain.",
			SummaryText: "Three-step research explanation template",
			CreatedAt:   now,
			UpdatedAt:   now,
		},
		{
			ID:          "cnt_meeting_001",
			Title:       "Meeting Decision Summary",
			ContentType: "reading",
			Level:       "intermediate",
			Topic:       "meeting",
			Language:    "en",
			RawText:     "We decided to move the user test to Friday, and Ken will send the updated schedule by this afternoon.",
			SummaryText: "Decision plus action item",
			CreatedAt:   now,
			UpdatedAt:   now,
		},
		{
			ID:          "cnt_meeting_002",
			Title:       "Meeting Listening: Next Action",
			ContentType: "listening",
			Level:       "intermediate",
			Topic:       "meeting",
			Language:    "en",
			RawText:     "Before the next meeting, please review the draft slides and write down one risk we should discuss.",
			SummaryText: "Short meeting instruction with one task",
			CreatedAt:   now,
			UpdatedAt:   now,
		},
		{
			ID:          "cnt_rescue_001",
			Title:       "Rescue: Ask for the Main Point",
			ContentType: "rescue",
			Level:       "intro",
			Topic:       "rescue",
			Language:    "en",
			RawText:     "Can you tell me the main point first?",
			SummaryText: "Rescue phrase for overload during explanations",
			CreatedAt:   now,
			UpdatedAt:   now,
		},
		{
			ID:          "cnt_rescue_002",
			Title:       "Rescue: Ask for a Shorter Version",
			ContentType: "rescue",
			Level:       "intro",
			Topic:       "rescue",
			Language:    "en",
			RawText:     "Could you say that in a shorter way?",
			SummaryText: "Rescue phrase to reduce sentence length",
			CreatedAt:   now,
			UpdatedAt:   now,
		},
	}
	for _, item := range items {
		s.contents[item.ID] = item
	}
}
