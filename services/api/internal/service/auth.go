package service

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"strings"
	"time"

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

	return s.newAuthResult(ctx, user)
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

	return s.newAuthResult(ctx, user)
}

func (s AuthService) Guest(ctx context.Context) (AuthResult, error) {
	passwordHash, err := s.hasher.Hash(newSecureID("guest_secret"))
	if err != nil {
		return AuthResult{}, err
	}

	guestID := newSecureID("guest")
	user, err := s.auth.CreateUserWithPassword(ctx, repository.NewAuthUser{
		Email:        guestID + "@guest.memory-safe.local",
		DisplayName:  "Guest " + strings.ToUpper(guestID[len(guestID)-4:]),
		AuthProvider: "guest",
		PasswordHash: passwordHash,
	})
	if err != nil {
		return AuthResult{}, err
	}

	result, err := s.newAuthResult(ctx, user)
	if err != nil {
		return AuthResult{}, err
	}
	result.NativeNotice = "guest_session"
	return result, nil
}

func (s AuthService) Refresh(ctx context.Context, refreshToken string) (AuthResult, error) {
	if strings.TrimSpace(refreshToken) == "" {
		return AuthResult{}, domain.ErrInvalidInput
	}

	rawToken := strings.TrimSpace(refreshToken)
	claims, err := s.tokens.ParseRefreshToken(rawToken)
	if err != nil {
		return AuthResult{}, domain.ErrUnauthorized
	}

	session, err := s.auth.GetRefreshSession(ctx, claims.TokenID)
	if err != nil {
		return AuthResult{}, domain.ErrUnauthorized
	}
	family, err := s.auth.GetRefreshFamily(ctx, claims.FamilyID)
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
	if family.UserID != user.ID || session.UserID != user.ID || session.FamilyID != family.ID {
		_ = s.auth.RevokeRefreshFamily(ctx, family.ID)
		return AuthResult{}, domain.ErrUnauthorized
	}
	if family.RevokedAt != nil || session.RevokedAt != nil {
		_ = s.auth.RevokeRefreshFamily(ctx, family.ID)
		return AuthResult{}, domain.ErrUnauthorized
	}
	if time.Now().UTC().After(session.ExpiresAt) {
		_ = s.auth.RevokeRefreshFamily(ctx, family.ID)
		return AuthResult{}, domain.ErrUnauthorized
	}

	currentHash := s.tokens.HashToken(rawToken)
	if session.TokenHash != currentHash {
		_ = s.auth.RevokeRefreshFamily(ctx, family.ID)
		return AuthResult{}, domain.ErrUnauthorized
	}

	return s.rotateAuthResult(ctx, user, family.ID, session.ID, currentHash)
}

func (s AuthService) newAuthResult(ctx context.Context, user domain.User) (AuthResult, error) {
	return s.issueNewFamilyAuthResult(ctx, user)
}

func (s AuthService) issueNewFamilyAuthResult(ctx context.Context, user domain.User) (AuthResult, error) {
	familyID := newSecureID("rfm")
	refreshTokenID := newSecureID("rft")

	tokens, err := s.tokens.Issue(user, refreshTokenID, familyID)
	if err != nil {
		return AuthResult{}, err
	}

	if err := s.auth.CreateRefreshSession(ctx, domain.RefreshTokenFamily{
		ID:        familyID,
		UserID:    user.ID,
		CreatedAt: time.Now().UTC(),
	}, domain.RefreshSession{
		ID:        refreshTokenID,
		FamilyID:  familyID,
		UserID:    user.ID,
		TokenHash: s.tokens.HashToken(tokens.RefreshToken),
		ExpiresAt: tokens.RefreshExpiresAt,
		CreatedAt: time.Now().UTC(),
	}); err != nil {
		return AuthResult{}, err
	}

	return AuthResult{
		User:   user,
		Tokens: tokens,
	}, nil
}

func (s AuthService) rotateAuthResult(ctx context.Context, user domain.User, familyID, currentTokenID, currentTokenHash string) (AuthResult, error) {
	nextTokenID := newSecureID("rft")
	tokens, err := s.tokens.Issue(user, nextTokenID, familyID)
	if err != nil {
		return AuthResult{}, err
	}

	if err := s.auth.RotateRefreshSession(ctx, currentTokenID, currentTokenHash, domain.RefreshSession{
		ID:        nextTokenID,
		FamilyID:  familyID,
		UserID:    user.ID,
		TokenHash: s.tokens.HashToken(tokens.RefreshToken),
		ExpiresAt: tokens.RefreshExpiresAt,
		CreatedAt: time.Now().UTC(),
	}); err != nil {
		if err == domain.ErrConflict || err == domain.ErrExpired {
			return AuthResult{}, domain.ErrUnauthorized
		}
		return AuthResult{}, err
	}

	return AuthResult{
		User:   user,
		Tokens: tokens,
	}, nil
}

func newSecureID(prefix string) string {
	buf := make([]byte, 12)
	if _, err := rand.Read(buf); err != nil {
		return prefix + "_" + time.Now().UTC().Format("20060102150405.000000000")
	}
	return prefix + "_" + hex.EncodeToString(buf)
}
