package app

import (
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

	createBody := map[string]any{
		"title":        "Conference Self Intro",
		"content_type": "reading",
		"level":        "intro",
		"topic":        "self_intro",
		"language":     "en",
		"raw_text":     "Hello, my name is Aki, and I study cognitive support interfaces.",
		"summary_text": "Conference intro",
	}
	createRec := authorizedJSONRequest(t, server, accessToken, http.MethodPost, "/contents", createBody)
	if createRec.Code != http.StatusCreated {
		t.Fatalf("create content status = %d, body = %s", createRec.Code, createRec.Body.String())
	}

	var created struct {
		ContentID string `json:"content_id"`
	}
	if err := json.Unmarshal(createRec.Body.Bytes(), &created); err != nil {
		t.Fatalf("unmarshal create content response: %v", err)
	}
	if created.ContentID == "" {
		t.Fatalf("expected created content id")
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

	updateBody := map[string]any{
		"title":        "Research Presentation Opening",
		"content_type": "reading",
		"level":        "intermediate",
		"topic":        "research",
		"language":     "en",
		"raw_text":     "We redesigned reading support to lower memory load during English processing.",
		"summary_text": "Updated research opening",
	}
	updateRec := authorizedJSONRequest(t, server, accessToken, http.MethodPatch, "/contents/cnt_research_001", updateBody)
	if updateRec.Code != http.StatusOK {
		t.Fatalf("update content status = %d, body = %s", updateRec.Code, updateRec.Body.String())
	}

	chunksReq3 := httptest.NewRequest(http.MethodGet, "/contents/cnt_research_001/chunks", nil)
	chunksReq3.Header.Set("Authorization", "Bearer "+accessToken)
	chunksRec3 := httptest.NewRecorder()
	server.Handler.ServeHTTP(chunksRec3, chunksReq3)
	if chunksRec3.Code != http.StatusOK {
		t.Fatalf("get chunks third status = %d, body = %s", chunksRec3.Code, chunksRec3.Body.String())
	}

	if workerCalls != 2 {
		t.Fatalf("expected worker to be called twice after cache invalidation, got %d", workerCalls)
	}
}
