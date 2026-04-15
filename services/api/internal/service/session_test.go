package service

import (
	"testing"

	"memory-safe-english/services/api/internal/store/memory"
)

func TestSessionServiceStartAndComplete(t *testing.T) {
	store := memory.NewStore()
	auth := NewAuthService(store)
	sessionSvc := NewSessionService(store, store)

	authResult, err := auth.Register(RegisterInput{
		Email:         "user@example.com",
		Password:      "secret123",
		DisplayName:   "Aki",
		AgreedToTerms: true,
	})
	if err != nil {
		t.Fatalf("Register() error = %v", err)
	}

	session, err := sessionSvc.Start(StartSessionInput{
		UserID: authResult.User.ID,
		Mode:   "reading",
	})
	if err != nil {
		t.Fatalf("Start() error = %v", err)
	}
	if session.ID == "" {
		t.Fatalf("expected session id to be set")
	}

	completed, err := sessionSvc.Complete(session.ID)
	if err != nil {
		t.Fatalf("Complete() error = %v", err)
	}
	if completed.CompletionState != "completed" {
		t.Fatalf("expected completed session, got %q", completed.CompletionState)
	}
}
