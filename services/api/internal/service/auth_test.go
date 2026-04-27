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
	if refreshed.Tokens.RefreshToken == result.Tokens.RefreshToken {
		t.Fatalf("expected rotated refresh token")
	}
}

func TestAuthServiceRefreshRejectsReusedTokenFamily(t *testing.T) {
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
		t.Fatalf("first Refresh() error = %v", err)
	}

	if _, err := svc.Refresh(context.Background(), result.Tokens.RefreshToken); err == nil {
		t.Fatalf("expected reused refresh token to be rejected")
	}

	if _, err := svc.Refresh(context.Background(), refreshed.Tokens.RefreshToken); err == nil {
		t.Fatalf("expected latest refresh token to be rejected after family revoke")
	}
}

func TestAuthServiceGuest(t *testing.T) {
	store := memory.NewStore()
	svc := NewAuthService(store, store, password.NewHasher(100000), token.NewManager("test-secret", 15*time.Minute, 30*time.Minute))

	result, err := svc.Guest(context.Background())
	if err != nil {
		t.Fatalf("Guest() error = %v", err)
	}
	if result.User.ID == "" {
		t.Fatalf("expected guest user id")
	}
	if result.User.AuthProvider != "guest" {
		t.Fatalf("expected guest auth provider, got %q", result.User.AuthProvider)
	}
	if result.Tokens.AccessToken == "" || result.Tokens.RefreshToken == "" {
		t.Fatalf("expected guest tokens")
	}
	if result.NativeNotice != "guest_session" {
		t.Fatalf("expected guest notice, got %q", result.NativeNotice)
	}
}
