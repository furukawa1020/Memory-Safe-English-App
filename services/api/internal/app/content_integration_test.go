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

func TestServerContentFlow(t *testing.T) {
	workerCalls := 0
	worker := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		workerCalls++
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

	accessToken := registerTestUser(t, server)

	listReq := httptest.NewRequest(http.MethodGet, "/contents?type=reading", nil)
	listReq.Header.Set("Authorization", "Bearer "+accessToken)
	listRec := httptest.NewRecorder()
	server.Handler.ServeHTTP(listRec, listReq)
	if listRec.Code != http.StatusOK {
		t.Fatalf("list contents status = %d, body = %s", listRec.Code, listRec.Body.String())
	}

	chunksReq := httptest.NewRequest(http.MethodGet, "/contents/cnt_research_001/chunks", nil)
	chunksReq.Header.Set("Authorization", "Bearer "+accessToken)
	chunksRec := httptest.NewRecorder()
	server.Handler.ServeHTTP(chunksRec, chunksReq)
	if chunksRec.Code != http.StatusOK {
		t.Fatalf("get chunks status = %d, body = %s", chunksRec.Code, chunksRec.Body.String())
	}

	chunksReq2 := httptest.NewRequest(http.MethodGet, "/contents/cnt_research_001/chunks", nil)
	chunksReq2.Header.Set("Authorization", "Bearer "+accessToken)
	chunksRec2 := httptest.NewRecorder()
	server.Handler.ServeHTTP(chunksRec2, chunksReq2)
	if chunksRec2.Code != http.StatusOK {
		t.Fatalf("get chunks second status = %d, body = %s", chunksRec2.Code, chunksRec2.Body.String())
	}

	if workerCalls != 1 {
		t.Fatalf("expected worker to be called once due to cache, got %d", workerCalls)
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
	return registerResp.Tokens.AccessToken
}
