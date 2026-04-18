package service

import (
	"context"
	"testing"

	"memory-safe-english/services/api/internal/workerclient"
)

type stubAnalyzer struct {
	result workerclient.ChunkingResult
	err    error
}

func (s stubAnalyzer) AnalyzeChunks(_ context.Context, text, language string) (workerclient.ChunkingResult, error) {
	return s.result, s.err
}

func TestAnalysisServiceAnalyzeChunks(t *testing.T) {
	svc := NewAnalysisService(stubAnalyzer{
		result: workerclient.ChunkingResult{
			Language: "en",
			Chunks: []workerclient.Chunk{
				{Order: 1, Text: "we propose", Role: "core", SkeletonRank: 1},
			},
			Summary: "we propose",
		},
	})

	result, err := svc.AnalyzeChunks(context.Background(), AnalyzeChunksInput{
		Text:     "We propose a memory safe interface.",
		Language: "",
	})
	if err != nil {
		t.Fatalf("AnalyzeChunks() error = %v", err)
	}
	if result.Language != "en" {
		t.Fatalf("expected language en, got %q", result.Language)
	}
	if len(result.Chunks) != 1 {
		t.Fatalf("expected 1 chunk, got %d", len(result.Chunks))
	}
}

func TestAnalysisServiceAnalyzeChunksRejectsEmptyText(t *testing.T) {
	svc := NewAnalysisService(stubAnalyzer{})

	_, err := svc.AnalyzeChunks(context.Background(), AnalyzeChunksInput{})
	if err == nil {
		t.Fatalf("expected validation error")
	}
}
