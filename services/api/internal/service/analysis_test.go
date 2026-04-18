package service

import (
	"context"
	"testing"

	"memory-safe-english/services/api/internal/domain"
)

type stubAnalyzer struct {
	result domain.ChunkingResult
	err    error
}

func (s stubAnalyzer) AnalyzeChunks(_ context.Context, text, language string) (domain.ChunkingResult, error) {
	return s.result, s.err
}

func TestAnalysisServiceAnalyzeChunks(t *testing.T) {
	svc := NewAnalysisService(stubAnalyzer{
		result: domain.ChunkingResult{
			Language: "en",
			Chunks: []domain.Chunk{
				{Order: 1, Text: "we propose", Role: "core", SkeletonRank: 1},
			},
			Summary: "we propose",
		},
	})

	result, err := svc.AnalyzeChunks(context.Background(), domain.AnalyzeChunksInput{
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

	_, err := svc.AnalyzeChunks(context.Background(), domain.AnalyzeChunksInput{})
	if err == nil {
		t.Fatalf("expected validation error")
	}
}

func TestAnalysisServiceAnalyzeChunksRejectsTooLongText(t *testing.T) {
	svc := NewAnalysisService(stubAnalyzer{})

	longText := make([]rune, domain.MaxAnalysisTextLength+1)
	for i := range longText {
		longText[i] = 'a'
	}

	_, err := svc.AnalyzeChunks(context.Background(), domain.AnalyzeChunksInput{Text: string(longText)})
	if err == nil {
		t.Fatalf("expected validation error for too long text")
	}
}
