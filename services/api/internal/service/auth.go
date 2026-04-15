package service

import (
	"context"
	"strings"

	"memory-safe-english/services/api/internal/domain"
	"memory-safe-english/services/api/internal/repository"
)

type AuthService struct {
	users repository.UserRepository
}

type RegisterInput struct {
	Email         string
	Password      string
	DisplayName   string
	AgreedToTerms bool
}

type AuthResult struct {
	User         domain.User       `json:"user"`
	Tokens       map[string]string `json:"tokens"`
	NativeNotice string            `json:"native_notice,omitempty"`
}

func NewAuthService(users repository.UserRepository) AuthService {
	return AuthService{users: users}
}

func (s AuthService) Register(ctx context.Context, input RegisterInput) (AuthResult, error) {
	if strings.TrimSpace(input.Email) == "" || strings.TrimSpace(input.Password) == "" || strings.TrimSpace(input.DisplayName) == "" {
		return AuthResult{}, domain.ErrInvalidInput
	}
	if !input.AgreedToTerms {
		return AuthResult{}, domain.ErrInvalidInput
	}

	user, err := s.users.CreateUser(ctx, strings.TrimSpace(input.Email), strings.TrimSpace(input.DisplayName), "email")
	if err != nil {
		return AuthResult{}, err
	}

	return newAuthResult(user), nil
}

func (s AuthService) Login(ctx context.Context, userID string) (AuthResult, error) {
	if strings.TrimSpace(userID) == "" {
		return AuthResult{}, domain.ErrInvalidInput
	}

	user, err := s.users.GetUser(ctx, strings.TrimSpace(userID))
	if err != nil {
		return AuthResult{}, err
	}

	return newAuthResult(user), nil
}

func newAuthResult(user domain.User) AuthResult {
	return AuthResult{
		User: user,
		Tokens: map[string]string{
			"access_token":  "dev-access-" + user.ID,
			"refresh_token": "dev-refresh-" + user.ID,
		},
	}
}
