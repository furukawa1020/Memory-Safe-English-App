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

func TestServerAnalyzeChunksFlow(t *testing.T) {
	worker := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Header.Get("X-Worker-Api-Key") == "" || r.Header.Get("X-Worker-Signature") == "" || r.Header.Get("X-Worker-Timestamp") == "" {
			t.Fatalf("expected worker auth headers")
		}
		_ = json.NewEncoder(w).Encode(map[string]any{
			"language": "en",
			"chunks": []map[string]any{
				{"order": 1, "text": "we propose", "role": "core", "skeleton_rank": 1},
			},
			"summary": "we propose",
		})
	}))
	defer worker.Close()

	server, err := NewServer(config.Config{
		HTTPAddr:               ":0",
		AppEnv:                 "test",
		AuthTokenSecret:        "test-secret",
		AccessTokenTTL:         15 * time.Minute,
		RefreshTokenTTL:        30 * time.Minute,
		PasswordHashIterations: 100000,
		WorkerBaseURL:          worker.URL,
		WorkerAPIKey:           "worker-api-key",
		WorkerSignatureKey:     "worker-signature-key",
		WorkerTimeout:          2 * time.Second,
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

	var registerResp struct {
		Tokens struct {
			AccessToken string `json:"access_token"`
		} `json:"tokens"`
	}
	if err := json.Unmarshal(registerRec.Body.Bytes(), &registerResp); err != nil {
		t.Fatalf("unmarshal register response: %v", err)
	}

	analysisBody := map[string]any{
		"text":     "We propose a memory safe interface.",
		"language": "en",
	}
	analysisBytes, _ := json.Marshal(analysisBody)
	analysisReq := httptest.NewRequest(http.MethodPost, "/analysis/chunks", bytes.NewReader(analysisBytes))
	analysisReq.Header.Set("Content-Type", "application/json")
	analysisReq.Header.Set("Authorization", "Bearer "+registerResp.Tokens.AccessToken)
	analysisRec := httptest.NewRecorder()
	server.Handler.ServeHTTP(analysisRec, analysisReq)

	if analysisRec.Code != http.StatusOK {
		t.Fatalf("analysis status = %d, body = %s", analysisRec.Code, analysisRec.Body.String())
	}
	var payload map[string]any
	if err := json.Unmarshal(analysisRec.Body.Bytes(), &payload); err != nil {
		t.Fatalf("unmarshal analysis response: %v", err)
	}
	if payload["summary"] == "" {
		t.Fatalf("expected summary in analysis response")
	}
}
