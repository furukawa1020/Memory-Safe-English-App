package service

import (
	"context"
	"testing"

	"memory-safe-english/services/api/internal/domain"
	"memory-safe-english/services/api/internal/store/memory"
)

func TestContentServiceListAndGet(t *testing.T) {
	store := memory.NewStore()
	svc := NewContentService(store, stubAnalyzer{
		result: domain.ChunkingResult{
			Language: "en",
			Chunks: []domain.Chunk{
				{Order: 1, Text: "we propose", Role: "core", SkeletonRank: 1},
			},
			Summary: "we propose",
		},
	})

	items, err := svc.List(context.Background(), ListContentsInput{ContentType: "reading"})
	if err != nil {
		t.Fatalf("List() error = %v", err)
	}
	if len(items) == 0 {
		t.Fatalf("expected seeded contents")
	}

	content, err := svc.Get(context.Background(), "cnt_research_001")
	if err != nil {
		t.Fatalf("Get() error = %v", err)
	}
	if content.ID != "cnt_research_001" {
		t.Fatalf("unexpected content id %q", content.ID)
	}
}

func TestContentServiceGetChunksCachesResult(t *testing.T) {
	store := memory.NewStore()
	calls := 0
	svc := NewContentService(store, analyzerFunc(func(ctx context.Context, text, language string) (domain.ChunkingResult, error) {
		calls++
		return domain.ChunkingResult{
			Language: language,
			Chunks: []domain.Chunk{
				{Order: 1, Text: "we propose", Role: "core", SkeletonRank: 1},
			},
			Summary: "we propose",
		}, nil
	}))

	if _, err := svc.GetChunks(context.Background(), "cnt_research_001"); err != nil {
		t.Fatalf("GetChunks() first call error = %v", err)
	}
	if _, err := svc.GetChunks(context.Background(), "cnt_research_001"); err != nil {
		t.Fatalf("GetChunks() second call error = %v", err)
	}
	if calls != 1 {
		t.Fatalf("expected analyzer to be called once, got %d", calls)
	}
}

type analyzerFunc func(ctx context.Context, text, language string) (domain.ChunkingResult, error)

func (f analyzerFunc) AnalyzeChunks(ctx context.Context, text, language string) (domain.ChunkingResult, error) {
	return f(ctx, text, language)
}
