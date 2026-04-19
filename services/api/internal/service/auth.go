package service

import (
	"context"
	"strings"

	"memory-safe-english/services/api/internal/domain"
	"memory-safe-english/services/api/internal/repository"
	"memory-safe-english/services/api/internal/security/password"
	"memory-safe-english/services/api/internal/security/token"
)

type AuthService struct {
	auth   AuthStore
	users  UserReader
	hasher password.Hasher
	tokens token.Manager
}

type AuthStore interface {
	repository.AuthRepository
}

type RegisterInput struct {
	Email         string
	Password      string
	DisplayName   string
	AgreedToTerms bool
}

type AuthResult struct {
	User         domain.User     `json:"user"`
	Tokens       token.TokenPair `json:"tokens"`
	NativeNotice string          `json:"native_notice,omitempty"`
}

func NewAuthService(auth AuthStore, users UserReader, hasher password.Hasher, tokens token.Manager) AuthService {
	return AuthService{
		auth:   auth,
		users:  users,
		hasher: hasher,
		tokens: tokens,
	}
}

func (s AuthService) Register(ctx context.Context, input RegisterInput) (AuthResult, error) {
	if strings.TrimSpace(input.Email) == "" || strings.TrimSpace(input.Password) == "" || strings.TrimSpace(input.DisplayName) == "" {
		return AuthResult{}, domain.ErrInvalidInput
	}
	if !input.AgreedToTerms {
		return AuthResult{}, domain.ErrInvalidInput
	}
	if len(input.Password) < 12 {
		return AuthResult{}, domain.ErrInvalidInput
	}

	passwordHash, err := s.hasher.Hash(input.Password)
	if err != nil {
		return AuthResult{}, err
	}

	user, err := s.auth.CreateUserWithPassword(ctx, repository.NewAuthUser{
		Email:        strings.TrimSpace(strings.ToLower(input.Email)),
		DisplayName:  strings.TrimSpace(input.DisplayName),
		AuthProvider: "email",
		PasswordHash: passwordHash,
	})
	if err != nil {
		return AuthResult{}, err
	}

	return s.newAuthResult(user)
}

func (s AuthService) Login(ctx context.Context, email, plainPassword string) (AuthResult, error) {
	if strings.TrimSpace(email) == "" || strings.TrimSpace(plainPassword) == "" {
		return AuthResult{}, domain.ErrInvalidInput
	}

	user, passwordHash, err := s.auth.FindUserByEmail(ctx, strings.TrimSpace(strings.ToLower(email)))
	if err != nil {
		return AuthResult{}, err
	}

	ok, err := s.hasher.Verify(plainPassword, passwordHash)
	if err != nil {
		return AuthResult{}, err
	}
	if !ok {
		return AuthResult{}, domain.ErrUnauthorized
	}

	return s.newAuthResult(user)
}

func (s AuthService) Refresh(ctx context.Context, refreshToken string) (AuthResult, error) {
	if strings.TrimSpace(refreshToken) == "" {
		return AuthResult{}, domain.ErrInvalidInput
	}

	claims, err := s.tokens.ParseRefreshToken(strings.TrimSpace(refreshToken))
	if err != nil {
		return AuthResult{}, domain.ErrUnauthorized
	}

	user, err := s.users.GetUser(ctx, claims.Subject)
	if err != nil {
		return AuthResult{}, err
	}

	if claims.Email != "" && user.Email != claims.Email {
		return AuthResult{}, domain.ErrUnauthorized
	}

	return s.newAuthResult(user)
}

func (s AuthService) newAuthResult(user domain.User) (AuthResult, error) {
	tokens, err := s.tokens.Issue(user)
	if err != nil {
		return AuthResult{}, err
	}

	return AuthResult{
		User:   user,
		Tokens: tokens,
	}, nil
}
