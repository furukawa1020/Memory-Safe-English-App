package workerclient

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"memory-safe-english/services/api/internal/domain"
)

func TestClientAnalyzeChunks(t *testing.T) {
	var gotAPIKey, gotTimestamp, gotSignature string
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotAPIKey = r.Header.Get("X-Worker-Api-Key")
		gotTimestamp = r.Header.Get("X-Worker-Timestamp")
		gotSignature = r.Header.Get("X-Worker-Signature")
		_ = json.NewEncoder(w).Encode(domain.ChunkingResult{
			Language: "en",
			Chunks: []domain.Chunk{
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

func TestClientAnalyzeChunksReturnsUnavailableOnInvalidJSON(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_, _ = w.Write([]byte("{"))
	}))
	defer server.Close()

	client := New(server.URL, "api-key", "signature-key", 2*time.Second)
	_, err := client.AnalyzeChunks(context.Background(), "hello", "en")
	if err == nil {
		t.Fatalf("expected error")
	}
	if !errors.Is(err, domain.ErrUnavailable) {
		t.Fatalf("expected unavailable error, got %v", err)
	}
}

func TestClientAnalyzeChunksReturnsUpstreamError(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Error(w, "bad worker request", http.StatusUnauthorized)
	}))
	defer server.Close()

	client := New(server.URL, "api-key", "signature-key", 2*time.Second)
	_, err := client.AnalyzeChunks(context.Background(), "hello", "en")
	var upstreamErr UpstreamError
	if !errors.As(err, &upstreamErr) {
		t.Fatalf("expected upstream error, got %v", err)
	}
	if upstreamErr.StatusCode != http.StatusUnauthorized {
		t.Fatalf("expected status 401, got %d", upstreamErr.StatusCode)
	}
}
