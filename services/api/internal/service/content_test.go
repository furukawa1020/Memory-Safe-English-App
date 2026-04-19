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

func TestContentServiceCreateAndUpdate(t *testing.T) {
	store := memory.NewStore()
	svc := NewContentService(store, stubAnalyzer{})

	created, err := svc.Create(context.Background(), domain.ContentUpsertInput{
		Title:       "Lab Introduction",
		ContentType: "reading",
		Level:       "intermediate",
		Topic:       "research",
		Language:    "en",
		RawText:     "Our lab studies accessible interfaces for language learning.",
		SummaryText: "Lab overview",
	})
	if err != nil {
		t.Fatalf("Create() error = %v", err)
	}
	if created.ID == "" {
		t.Fatalf("expected content id")
	}

	updated, err := svc.Update(context.Background(), created.ID, domain.ContentUpsertInput{
		Title:       "Updated Lab Introduction",
		ContentType: "reading",
		Level:       "intermediate",
		Topic:       "research",
		Language:    "en",
		RawText:     "Our lab studies accessible interfaces and memory-safe reading support.",
		SummaryText: "Updated lab overview",
	})
	if err != nil {
		t.Fatalf("Update() error = %v", err)
	}
	if updated.Title != "Updated Lab Introduction" {
		t.Fatalf("expected updated title, got %q", updated.Title)
	}
	if updated.CreatedAt.IsZero() || updated.UpdatedAt.IsZero() {
		t.Fatalf("expected timestamps to be set")
	}
	if !updated.UpdatedAt.After(updated.CreatedAt) && !updated.UpdatedAt.Equal(updated.CreatedAt) {
		t.Fatalf("expected updated timestamp to be on or after created timestamp")
	}
}

func TestContentServiceUpdateInvalidatesChunkCache(t *testing.T) {
	store := memory.NewStore()
	calls := 0
	svc := NewContentService(store, analyzerFunc(func(ctx context.Context, text, language string) (domain.ChunkingResult, error) {
		calls++
		return domain.ChunkingResult{
			Language: language,
			Chunks: []domain.Chunk{
				{Order: 1, Text: text, Role: "core", SkeletonRank: 1},
			},
			Summary: text,
		}, nil
	}))

	if _, err := svc.GetChunks(context.Background(), "cnt_research_001"); err != nil {
		t.Fatalf("GetChunks() initial call error = %v", err)
	}

	_, err := svc.Update(context.Background(), "cnt_research_001", domain.ContentUpsertInput{
		Title:       "Research Presentation Opening",
		ContentType: "reading",
		Level:       "intermediate",
		Topic:       "research",
		Language:    "en",
		RawText:     "We designed a calmer interface for memory-safe English reading.",
		SummaryText: "Updated research opening sentence",
	})
	if err != nil {
		t.Fatalf("Update() error = %v", err)
	}

	if _, err := svc.GetChunks(context.Background(), "cnt_research_001"); err != nil {
		t.Fatalf("GetChunks() after update error = %v", err)
	}

	if calls != 2 {
		t.Fatalf("expected analyzer to be called twice after cache invalidation, got %d", calls)
	}
}

type analyzerFunc func(ctx context.Context, text, language string) (domain.ChunkingResult, error)

func (f analyzerFunc) AnalyzeChunks(ctx context.Context, text, language string) (domain.ChunkingResult, error) {
	return f(ctx, text, language)
}
