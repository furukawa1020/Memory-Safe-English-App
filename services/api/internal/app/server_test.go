package app

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestServerRegisterAndMeFlow(t *testing.T) {
	server, err := NewServer(newTestConfig())
	if err != nil {
		t.Fatalf("NewServer() error = %v", err)
	}

	rec := jsonRequest(t, server, http.MethodPost, "/auth/register", "", map[string]any{
		"email":           "user@example.com",
		"password":        "secret1234567",
		"display_name":    "Aki",
		"agreed_to_terms": true,
	})
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
	server, err := NewServer(newTestConfig())
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
	server, err := NewServer(newTestConfig())
	if err != nil {
		t.Fatalf("NewServer() error = %v", err)
	}

	registerRec := jsonRequest(t, server, http.MethodPost, "/auth/register", "", map[string]any{
		"email":           "user@example.com",
		"password":        "secret1234567",
		"display_name":    "Aki",
		"agreed_to_terms": true,
	})
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

	refreshRec := jsonRequest(t, server, http.MethodPost, "/auth/refresh", "", map[string]any{
		"refresh_token": registerResponse.Tokens.RefreshToken,
	})

	if refreshRec.Code != http.StatusOK {
		t.Fatalf("refresh status = %d, body = %s", refreshRec.Code, refreshRec.Body.String())
	}

	var refreshResponse struct {
		Tokens struct {
			AccessToken  string `json:"access_token"`
			RefreshToken string `json:"refresh_token"`
		} `json:"tokens"`
	}
	if err := json.Unmarshal(refreshRec.Body.Bytes(), &refreshResponse); err != nil {
		t.Fatalf("unmarshal refresh response: %v", err)
	}
	if refreshResponse.Tokens.AccessToken == "" {
		t.Fatalf("expected access token in refresh response")
	}
	if refreshResponse.Tokens.RefreshToken == "" {
		t.Fatalf("expected rotated refresh token in refresh response")
	}
	if refreshResponse.Tokens.RefreshToken == registerResponse.Tokens.RefreshToken {
		t.Fatalf("expected refresh token rotation")
	}

	reusedRec := jsonRequest(t, server, http.MethodPost, "/auth/refresh", "", map[string]any{
		"refresh_token": registerResponse.Tokens.RefreshToken,
	})
	if reusedRec.Code != http.StatusUnauthorized {
		t.Fatalf("expected reused refresh token to be rejected, got %d body=%s", reusedRec.Code, reusedRec.Body.String())
	}

	latestRec := jsonRequest(t, server, http.MethodPost, "/auth/refresh", "", map[string]any{
		"refresh_token": refreshResponse.Tokens.RefreshToken,
	})
	if latestRec.Code != http.StatusUnauthorized {
		t.Fatalf("expected rotated family to be revoked after reuse, got %d body=%s", latestRec.Code, latestRec.Body.String())
	}
}
