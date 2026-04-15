package app

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"memory-safe-english/services/api/internal/config"
)

func TestServerRegisterAndMeFlow(t *testing.T) {
	server, err := NewServer(config.Config{
		HTTPAddr:               ":0",
		AppEnv:                 "test",
		AuthTokenSecret:        "test-secret",
		AccessTokenTTL:         15 * time.Minute,
		RefreshTokenTTL:        30 * time.Minute,
		PasswordHashIterations: 100000,
	})
	if err != nil {
		t.Fatalf("NewServer() error = %v", err)
	}

	registerBody := map[string]any{
		"email":           "user@example.com",
		"password":        "secret1234567",
		"display_name":    "Aki",
		"agreed_to_terms": true,
	}
	bodyBytes, _ := json.Marshal(registerBody)

	req := httptest.NewRequest(http.MethodPost, "/auth/register", bytes.NewReader(bodyBytes))
	req.Header.Set("Content-Type", "application/json")
	rec := httptest.NewRecorder()
	server.Handler.ServeHTTP(rec, req)

	if rec.Code != http.StatusCreated {
		t.Fatalf("register status = %d, body = %s", rec.Code, rec.Body.String())
	}

	var registerResponse struct {
		User struct {
			UserID string `json:"user_id"`
		} `json:"user"`
		Tokens struct {
			AccessToken string `json:"access_token"`
		} `json:"tokens"`
	}
	if err := json.Unmarshal(rec.Body.Bytes(), &registerResponse); err != nil {
		t.Fatalf("unmarshal register response: %v", err)
	}
	if registerResponse.User.UserID == "" {
		t.Fatalf("expected user_id in register response")
	}
	if registerResponse.Tokens.AccessToken == "" {
		t.Fatalf("expected access token in register response")
	}

	meReq := httptest.NewRequest(http.MethodGet, "/me", nil)
	meReq.Header.Set("Authorization", "Bearer "+registerResponse.Tokens.AccessToken)
	meRec := httptest.NewRecorder()
	server.Handler.ServeHTTP(meRec, meReq)

	if meRec.Code != http.StatusOK {
		t.Fatalf("me status = %d, body = %s", meRec.Code, meRec.Body.String())
	}
	if got := meRec.Header().Get("X-Request-ID"); got == "" {
		t.Fatalf("expected X-Request-ID header")
	}
	if got := meRec.Header().Get("X-Content-Type-Options"); got != "nosniff" {
		t.Fatalf("expected security header, got %q", got)
	}
}

func TestServerRejectsProtectedRouteWithoutToken(t *testing.T) {
	server, err := NewServer(config.Config{
		HTTPAddr:               ":0",
		AppEnv:                 "test",
		AuthTokenSecret:        "test-secret",
		AccessTokenTTL:         15 * time.Minute,
		RefreshTokenTTL:        30 * time.Minute,
		PasswordHashIterations: 100000,
	})
	if err != nil {
		t.Fatalf("NewServer() error = %v", err)
	}

	req := httptest.NewRequest(http.MethodGet, "/me", nil)
	rec := httptest.NewRecorder()
	server.Handler.ServeHTTP(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("status = %d, body = %s", rec.Code, rec.Body.String())
	}
}

func TestServerRefreshFlow(t *testing.T) {
	server, err := NewServer(config.Config{
		HTTPAddr:               ":0",
		AppEnv:                 "test",
		AuthTokenSecret:        "test-secret",
		AccessTokenTTL:         15 * time.Minute,
		RefreshTokenTTL:        30 * time.Minute,
		PasswordHashIterations: 100000,
	})
	if err != nil {
		t.Fatalf("NewServer() error = %v", err)
	}

	registerBody := map[string]any{
		"email":           "user@example.com",
		"password":        "secret1234567",
		"display_name":    "Aki",
		"agreed_to_terms": true,
	}
	registerBytes, _ := json.Marshal(registerBody)

	registerReq := httptest.NewRequest(http.MethodPost, "/auth/register", bytes.NewReader(registerBytes))
	registerReq.Header.Set("Content-Type", "application/json")
	registerRec := httptest.NewRecorder()
	server.Handler.ServeHTTP(registerRec, registerReq)

	if registerRec.Code != http.StatusCreated {
		t.Fatalf("register status = %d, body = %s", registerRec.Code, registerRec.Body.String())
	}

	var registerResponse struct {
		Tokens struct {
			RefreshToken string `json:"refresh_token"`
		} `json:"tokens"`
	}
	if err := json.Unmarshal(registerRec.Body.Bytes(), &registerResponse); err != nil {
		t.Fatalf("unmarshal register response: %v", err)
	}
	if registerResponse.Tokens.RefreshToken == "" {
		t.Fatalf("expected refresh token in register response")
	}

	refreshBody := map[string]any{
		"refresh_token": registerResponse.Tokens.RefreshToken,
	}
	refreshBytes, _ := json.Marshal(refreshBody)

	refreshReq := httptest.NewRequest(http.MethodPost, "/auth/refresh", bytes.NewReader(refreshBytes))
	refreshReq.Header.Set("Content-Type", "application/json")
	refreshRec := httptest.NewRecorder()
	server.Handler.ServeHTTP(refreshRec, refreshReq)

	if refreshRec.Code != http.StatusOK {
		t.Fatalf("refresh status = %d, body = %s", refreshRec.Code, refreshRec.Body.String())
	}

	var refreshResponse struct {
		Tokens struct {
			AccessToken string `json:"access_token"`
		} `json:"tokens"`
	}
	if err := json.Unmarshal(refreshRec.Body.Bytes(), &refreshResponse); err != nil {
		t.Fatalf("unmarshal refresh response: %v", err)
	}
	if refreshResponse.Tokens.AccessToken == "" {
		t.Fatalf("expected access token in refresh response")
	}
}
