package app

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"memory-safe-english/services/api/internal/config"
)

func TestServerRegisterAndMeFlow(t *testing.T) {
	server, err := NewServer(config.Config{HTTPAddr: ":0", AppEnv: "test"})
	if err != nil {
		t.Fatalf("NewServer() error = %v", err)
	}

	registerBody := map[string]any{
		"email":            "user@example.com",
		"password":         "secret123",
		"display_name":     "Aki",
		"agreed_to_terms":  true,
		"native_language":  "ja",
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
	}
	if err := json.Unmarshal(rec.Body.Bytes(), &registerResponse); err != nil {
		t.Fatalf("unmarshal register response: %v", err)
	}
	if registerResponse.User.UserID == "" {
		t.Fatalf("expected user_id in register response")
	}

	meReq := httptest.NewRequest(http.MethodGet, "/me", nil)
	meReq.Header.Set("X-User-ID", registerResponse.User.UserID)
	meRec := httptest.NewRecorder()
	server.Handler.ServeHTTP(meRec, meReq)

	if meRec.Code != http.StatusOK {
		t.Fatalf("me status = %d, body = %s", meRec.Code, meRec.Body.String())
	}
	if got := meRec.Header().Get("X-Request-ID"); got == "" {
		t.Fatalf("expected X-Request-ID header")
	}
}

func TestServerRejectsProtectedRouteWithoutUserHeader(t *testing.T) {
	server, err := NewServer(config.Config{HTTPAddr: ":0", AppEnv: "test"})
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
