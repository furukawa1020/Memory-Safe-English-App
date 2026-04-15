package service

import (
	"context"
	"testing"

	"memory-safe-english/services/api/internal/security/password"
	"memory-safe-english/services/api/internal/security/token"
	"memory-safe-english/services/api/internal/store/memory"
)

func TestSessionServiceStartAndComplete(t *testing.T) {
	store := memory.NewStore()
	auth := NewAuthService(store, password.NewHasher(100000), token.NewManager("test-secret", 15, 30))
	sessionSvc := NewSessionService(store, store)

	authResult, err := auth.Register(context.Background(), RegisterInput{
		Email:         "user@example.com",
		Password:      "secret1234567",
		DisplayName:   "Aki",
		AgreedToTerms: true,
	})
	if err != nil {
		t.Fatalf("Register() error = %v", err)
	}

	session, err := sessionSvc.Start(context.Background(), StartSessionInput{
		UserID: authResult.User.ID,
		Mode:   "reading",
	})
	if err != nil {
		t.Fatalf("Start() error = %v", err)
	}
	if session.ID == "" {
		t.Fatalf("expected session id to be set")
	}

	completed, err := sessionSvc.Complete(context.Background(), session.ID)
	if err != nil {
		t.Fatalf("Complete() error = %v", err)
	}
	if completed.CompletionState != "completed" {
		t.Fatalf("expected completed session, got %q", completed.CompletionState)
	}
}
