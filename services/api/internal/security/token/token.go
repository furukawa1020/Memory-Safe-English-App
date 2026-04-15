package token

import (
	"crypto/hmac"
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"time"

	"memory-safe-english/services/api/internal/domain"
)

var (
	ErrInvalidToken = errors.New("invalid token")
	ErrExpiredToken = errors.New("expired token")
)

type Claims struct {
	Subject   string    `json:"sub"`
	Email     string    `json:"email"`
	TokenUse  string    `json:"token_use"`
	IssuedAt  time.Time `json:"iat"`
	ExpiresAt time.Time `json:"exp"`
}

type TokenPair struct {
	AccessToken  string    `json:"access_token"`
	RefreshToken string    `json:"refresh_token"`
	TokenType    string    `json:"token_type"`
	ExpiresAt    time.Time `json:"expires_at"`
}

type Manager struct {
	secret          []byte
	accessTokenTTL  time.Duration
	refreshTokenTTL time.Duration
}

func NewManager(secret string, accessTokenTTL, refreshTokenTTL time.Duration) Manager {
	return Manager{
		secret:          []byte(secret),
		accessTokenTTL:  accessTokenTTL,
		refreshTokenTTL: refreshTokenTTL,
	}
}

func (m Manager) Issue(user domain.User) (TokenPair, error) {
	now := time.Now().UTC()

	accessClaims := Claims{
		Subject:   user.ID,
		Email:     user.Email,
		TokenUse:  "access",
		IssuedAt:  now,
		ExpiresAt: now.Add(m.accessTokenTTL),
	}
	refreshClaims := Claims{
		Subject:   user.ID,
		Email:     user.Email,
		TokenUse:  "refresh",
		IssuedAt:  now,
		ExpiresAt: now.Add(m.refreshTokenTTL),
	}

	accessToken, err := m.sign(accessClaims)
	if err != nil {
		return TokenPair{}, err
	}
	refreshToken, err := m.sign(refreshClaims)
	if err != nil {
		return TokenPair{}, err
	}

	return TokenPair{
		AccessToken:  accessToken,
		RefreshToken: refreshToken,
		TokenType:    "Bearer",
		ExpiresAt:    accessClaims.ExpiresAt,
	}, nil
}

func (m Manager) ParseAccessToken(token string) (Claims, error) {
	claims, err := m.parse(token)
	if err != nil {
		return Claims{}, err
	}
	if claims.TokenUse != "access" {
		return Claims{}, ErrInvalidToken
	}
	return claims, nil
}

func (m Manager) sign(claims Claims) (string, error) {
	payload, err := json.Marshal(claims)
	if err != nil {
		return "", err
	}

	payloadEncoded := base64.RawURLEncoding.EncodeToString(payload)
	mac := hmac.New(sha256.New, m.secret)
	if _, err := mac.Write([]byte(payloadEncoded)); err != nil {
		return "", err
	}
	signatureEncoded := base64.RawURLEncoding.EncodeToString(mac.Sum(nil))

	return fmt.Sprintf("%s.%s", payloadEncoded, signatureEncoded), nil
}

func (m Manager) parse(token string) (Claims, error) {
	parts := strings.Split(token, ".")
	if len(parts) != 2 {
		return Claims{}, ErrInvalidToken
	}

	mac := hmac.New(sha256.New, m.secret)
	if _, err := mac.Write([]byte(parts[0])); err != nil {
		return Claims{}, err
	}
	expectedSignature := mac.Sum(nil)

	actualSignature, err := base64.RawURLEncoding.DecodeString(parts[1])
	if err != nil {
		return Claims{}, ErrInvalidToken
	}
	if !hmac.Equal(actualSignature, expectedSignature) {
		return Claims{}, ErrInvalidToken
	}

	payload, err := base64.RawURLEncoding.DecodeString(parts[0])
	if err != nil {
		return Claims{}, ErrInvalidToken
	}

	var claims Claims
	if err := json.Unmarshal(payload, &claims); err != nil {
		return Claims{}, ErrInvalidToken
	}
	if time.Now().UTC().After(claims.ExpiresAt) {
		return Claims{}, ErrExpiredToken
	}
	if claims.Subject == "" {
		return Claims{}, ErrInvalidToken
	}

	return claims, nil
}
