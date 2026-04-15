package service

import (
	"context"
	"testing"
	"time"

	"memory-safe-english/services/api/internal/security/password"
	"memory-safe-english/services/api/internal/security/token"
	"memory-safe-english/services/api/internal/store/memory"
)

func TestAuthServiceRegisterAndLogin(t *testing.T) {
	store := memory.NewStore()
	svc := NewAuthService(store, store, password.NewHasher(100000), token.NewManager("test-secret", 15*time.Minute, 30*time.Minute))

	result, err := svc.Register(context.Background(), RegisterInput{
		Email:         "user@example.com",
		Password:      "secret1234567",
		DisplayName:   "Aki",
		AgreedToTerms: true,
	})
	if err != nil {
		t.Fatalf("Register() error = %v", err)
	}
	if result.User.ID == "" {
		t.Fatalf("expected user id to be set")
	}

	loginResult, err := svc.Login(context.Background(), result.User.Email, "secret1234567")
	if err != nil {
		t.Fatalf("Login() error = %v", err)
	}
	if loginResult.User.ID != result.User.ID {
		t.Fatalf("expected login user id %q, got %q", result.User.ID, loginResult.User.ID)
	}
	if loginResult.Tokens.AccessToken == "" {
		t.Fatalf("expected access token to be issued")
	}
}

func TestAuthServiceRegisterRejectsInvalidInput(t *testing.T) {
	store := memory.NewStore()
	svc := NewAuthService(store, store, password.NewHasher(100000), token.NewManager("test-secret", 15*time.Minute, 30*time.Minute))

	_, err := svc.Register(context.Background(), RegisterInput{})
	if err == nil {
		t.Fatalf("expected validation error")
	}
}

func TestAuthServiceRefresh(t *testing.T) {
	store := memory.NewStore()
	svc := NewAuthService(store, store, password.NewHasher(100000), token.NewManager("test-secret", 15*time.Minute, 30*time.Minute))

	result, err := svc.Register(context.Background(), RegisterInput{
		Email:         "user@example.com",
		Password:      "secret1234567",
		DisplayName:   "Aki",
		AgreedToTerms: true,
	})
	if err != nil {
		t.Fatalf("Register() error = %v", err)
	}

	refreshed, err := svc.Refresh(context.Background(), result.Tokens.RefreshToken)
	if err != nil {
		t.Fatalf("Refresh() error = %v", err)
	}
	if refreshed.User.ID != result.User.ID {
		t.Fatalf("expected same user id %q, got %q", result.User.ID, refreshed.User.ID)
	}
	if refreshed.Tokens.AccessToken == "" {
		t.Fatalf("expected new access token")
	}
}
