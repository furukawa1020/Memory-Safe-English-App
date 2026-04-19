package service

import (
	"context"
	"testing"

	"memory-safe-english/services/api/internal/domain"
)

type stubAnalyzer struct {
	chunkResult    domain.ChunkingResult
	skeletonResult domain.SkeletonResult
	err            error
}

func (s stubAnalyzer) AnalyzeChunks(_ context.Context, text, language string) (domain.ChunkingResult, error) {
	return s.chunkResult, s.err
}

func (s stubAnalyzer) AnalyzeSkeleton(_ context.Context, text, language string) (domain.SkeletonResult, error) {
	return s.skeletonResult, s.err
}

func TestAnalysisServiceAnalyzeChunks(t *testing.T) {
	svc := NewAnalysisService(stubAnalyzer{
		chunkResult: domain.ChunkingResult{
			Version:  "2026-04-19",
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
	svc := NewAnalysisService(stubAnalyzer{}, stubAnalyzer{})

	_, err := svc.AnalyzeChunks(context.Background(), domain.AnalyzeChunksInput{})
	if err == nil {
		t.Fatalf("expected validation error")
	}
}

func TestAnalysisServiceAnalyzeChunksRejectsTooLongText(t *testing.T) {
	svc := NewAnalysisService(stubAnalyzer{}, stubAnalyzer{})

	longText := make([]rune, domain.MaxAnalysisTextLength+1)
	for i := range longText {
		longText[i] = 'a'
	}

	_, err := svc.AnalyzeChunks(context.Background(), domain.AnalyzeChunksInput{Text: string(longText)})
	if err == nil {
		t.Fatalf("expected validation error for too long text")
	}
}

func TestAnalysisServiceAnalyzeSkeleton(t *testing.T) {
	svc := NewAnalysisService(
		stubAnalyzer{},
		stubAnalyzer{
			skeletonResult: domain.SkeletonResult{
				Version:  "2026-04-19",
				Language: "en",
				Parts: []domain.SkeletonPart{
					{Order: 1, Text: "we propose", Role: "core", Emphasis: 2},
				},
				Summary: "we propose",
			},
		},
	)

	result, err := svc.AnalyzeSkeleton(context.Background(), domain.AnalyzeSkeletonInput{
		Text:     "We propose a memory safe interface.",
		Language: "",
	})
	if err != nil {
		t.Fatalf("AnalyzeSkeleton() error = %v", err)
	}
	if result.Language != "en" {
		t.Fatalf("expected language en, got %q", result.Language)
	}
	if len(result.Parts) != 1 {
		t.Fatalf("expected 1 part, got %d", len(result.Parts))
	}
}
