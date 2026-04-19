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

func newTestConfig() config.Config {
	return config.Config{
		HTTPAddr:               ":0",
		AppEnv:                 "test",
		AuthTokenSecret:        "test-secret",
		AccessTokenTTL:         15 * time.Minute,
		RefreshTokenTTL:        30 * time.Minute,
		PasswordHashIterations: 100000,
		AuthRateLimitWindow:    10 * time.Minute,
		LoginMaxAttempts:       10,
		RegisterMaxAttempts:    5,
		RefreshMaxAttempts:     20,
		WorkerBaseURL:          "http://127.0.0.1:8090",
		WorkerAPIKey:           "worker-api-key",
		WorkerSignatureKey:     "worker-signature-key",
		WorkerTimeout:          2 * time.Second,
	}
}

func registerTestUser(t *testing.T, server *http.Server) string {
	t.Helper()

	registerBody := map[string]any{
		"email":           "user@example.com",
		"password":        "secret1234567",
		"display_name":    "Aki",
		"agreed_to_terms": true,
	}

	rec := jsonRequest(t, server, http.MethodPost, "/auth/register", "", registerBody)
	if rec.Code != http.StatusCreated {
		t.Fatalf("register status = %d, body = %s", rec.Code, rec.Body.String())
	}

	var registerResp struct {
		Tokens struct {
			AccessToken string `json:"access_token"`
		} `json:"tokens"`
	}
	if err := json.Unmarshal(rec.Body.Bytes(), &registerResp); err != nil {
		t.Fatalf("unmarshal register response: %v", err)
	}
	return registerResp.Tokens.AccessToken
}

func authorizedJSONRequest(t *testing.T, server *http.Server, accessToken, method, path string, body map[string]any) *httptest.ResponseRecorder {
	t.Helper()
	return jsonRequest(t, server, method, path, accessToken, body)
}

func jsonRequest(t *testing.T, server *http.Server, method, path, accessToken string, body map[string]any) *httptest.ResponseRecorder {
	t.Helper()
	return jsonRequestWithHeaders(t, server, method, path, accessToken, body, nil)
}

func jsonRequestWithHeaders(t *testing.T, server *http.Server, method, path, accessToken string, body map[string]any, headers map[string]string) *httptest.ResponseRecorder {
	t.Helper()

	var reader *bytes.Reader
	if body == nil {
		reader = bytes.NewReader(nil)
	} else {
		payload, err := json.Marshal(body)
		if err != nil {
			t.Fatalf("marshal request body: %v", err)
		}
		reader = bytes.NewReader(payload)
	}

	req := httptest.NewRequest(method, path, reader)
	req.Header.Set("Content-Type", "application/json")
	if accessToken != "" {
		req.Header.Set("Authorization", "Bearer "+accessToken)
	}
	for key, value := range headers {
		req.Header.Set(key, value)
	}
	rec := httptest.NewRecorder()
	server.Handler.ServeHTTP(rec, req)
	return rec
}
