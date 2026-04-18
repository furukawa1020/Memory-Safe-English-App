package workerclient

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func TestClientAnalyzeChunks(t *testing.T) {
	var gotAPIKey, gotTimestamp, gotSignature string
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotAPIKey = r.Header.Get("X-Worker-Api-Key")
		gotTimestamp = r.Header.Get("X-Worker-Timestamp")
		gotSignature = r.Header.Get("X-Worker-Signature")
		_ = json.NewEncoder(w).Encode(ChunkingResult{
			Language: "en",
			Chunks: []Chunk{
				{Order: 1, Text: "we propose", Role: "core", SkeletonRank: 1},
			},
			Summary: "we propose",
		})
	}))
	defer server.Close()

	client := New(server.URL, "api-key", "signature-key", 2*time.Second)
	client.now = func() time.Time { return time.Unix(100, 0).UTC() }

	result, err := client.AnalyzeChunks(context.Background(), "We propose a memory safe interface.", "en")
	if err != nil {
		t.Fatalf("AnalyzeChunks() error = %v", err)
	}
	if gotAPIKey != "api-key" {
		t.Fatalf("expected api key header, got %q", gotAPIKey)
	}
	if gotTimestamp != "100" {
		t.Fatalf("expected timestamp 100, got %q", gotTimestamp)
	}
	if gotSignature == "" {
		t.Fatalf("expected signature header")
	}
	if result.Language != "en" {
		t.Fatalf("expected language en, got %q", result.Language)
	}
}
