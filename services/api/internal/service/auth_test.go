package service

import (
	"testing"

	"memory-safe-english/services/api/internal/store/memory"
)

func TestAuthServiceRegisterAndLogin(t *testing.T) {
	store := memory.NewStore()
	svc := NewAuthService(store)

	result, err := svc.Register(RegisterInput{
		Email:         "user@example.com",
		Password:      "secret123",
		DisplayName:   "Aki",
		AgreedToTerms: true,
	})
	if err != nil {
		t.Fatalf("Register() error = %v", err)
	}
	if result.User.ID == "" {
		t.Fatalf("expected user id to be set")
	}

	loginResult, err := svc.Login(result.User.ID)
	if err != nil {
		t.Fatalf("Login() error = %v", err)
	}
	if loginResult.User.ID != result.User.ID {
		t.Fatalf("expected login user id %q, got %q", result.User.ID, loginResult.User.ID)
	}
}

func TestAuthServiceRegisterRejectsInvalidInput(t *testing.T) {
	store := memory.NewStore()
	svc := NewAuthService(store)

	_, err := svc.Register(RegisterInput{})
	if err == nil {
		t.Fatalf("expected validation error")
	}
}
