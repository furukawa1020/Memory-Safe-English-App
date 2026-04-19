package app

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
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

	cfg := newTestConfig()
	cfg.WorkerBaseURL = worker.URL

	server, err := NewServer(cfg)
	if err != nil {
		t.Fatalf("NewServer() error = %v", err)
	}

	registerRec := jsonRequest(t, server, http.MethodPost, "/auth/register", "", map[string]any{
		"email":           "user@example.com",
		"password":        "secret1234567",
		"display_name":    "Aki",
		"agreed_to_terms": true,
	})
	var registerResp struct {
		Tokens struct {
			AccessToken string `json:"access_token"`
		} `json:"tokens"`
	}
	if err := json.Unmarshal(registerRec.Body.Bytes(), &registerResp); err != nil {
		t.Fatalf("unmarshal register response: %v", err)
	}

	analysisRec := authorizedJSONRequest(t, server, registerResp.Tokens.AccessToken, http.MethodPost, "/analysis/chunks", map[string]any{
		"text":     "We propose a memory safe interface.",
		"language": "en",
	})

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
