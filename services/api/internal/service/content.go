package service

import (
	"context"
	"fmt"
	"time"

	"memory-safe-english/services/api/internal/domain"
	"memory-safe-english/services/api/internal/repository"
)

type ContentService struct {
	contents         ContentStore
	chunkAnalyzer    ChunkAnalyzer
	skeletonAnalyzer SkeletonAnalyzer
}

type ContentStore interface {
	repository.ContentRepository
}

type ListContentsInput struct {
	ContentType string
	Level       string
	Topic       string
	Language    string
}

func NewContentService(contents ContentStore, chunkAnalyzer ChunkAnalyzer, skeletonAnalyzer SkeletonAnalyzer) ContentService {
	return ContentService{
		contents:         contents,
		chunkAnalyzer:    chunkAnalyzer,
		skeletonAnalyzer: skeletonAnalyzer,
	}
}

func (s ContentService) List(ctx context.Context, input ListContentsInput) ([]domain.Content, error) {
	return s.contents.ListContents(ctx, repository.ContentFilter{
		ContentType: input.ContentType,
		Level:       input.Level,
		Topic:       input.Topic,
		Language:    input.Language,
	})
}

func (s ContentService) Get(ctx context.Context, contentID string) (domain.Content, error) {
	if contentID == "" {
		return domain.Content{}, domain.ErrInvalidInput
	}
	return s.contents.GetContent(ctx, contentID)
}

func (s ContentService) GetChunks(ctx context.Context, contentID string) (domain.ChunkingResult, error) {
	if contentID == "" {
		return domain.ChunkingResult{}, domain.ErrInvalidInput
	}
	if cached, err := s.contents.GetChunkingResult(ctx, contentID); err == nil {
		return cached, nil
	}
	content, err := s.contents.GetContent(ctx, contentID)
	if err != nil {
		return domain.ChunkingResult{}, err
	}
	result, err := s.chunkAnalyzer.AnalyzeChunks(ctx, content.RawText, content.Language)
	if err != nil {
		return domain.ChunkingResult{}, err
	}
	if err := s.contents.SaveChunkingResult(ctx, contentID, result); err != nil {
		return domain.ChunkingResult{}, err
	}
	return result, nil
}

func (s ContentService) GetSkeleton(ctx context.Context, contentID string) (domain.SkeletonResult, error) {
	if contentID == "" {
		return domain.SkeletonResult{}, domain.ErrInvalidInput
	}
	if cached, err := s.contents.GetSkeletonResult(ctx, contentID); err == nil {
		return cached, nil
	}
	content, err := s.contents.GetContent(ctx, contentID)
	if err != nil {
		return domain.SkeletonResult{}, err
	}
	result, err := s.skeletonAnalyzer.AnalyzeSkeleton(ctx, content.RawText, content.Language)
	if err != nil {
		return domain.SkeletonResult{}, err
	}
	if err := s.contents.SaveSkeletonResult(ctx, contentID, result); err != nil {
		return domain.SkeletonResult{}, err
	}
	return result, nil
}

func (s ContentService) Create(ctx context.Context, input domain.ContentUpsertInput) (domain.Content, error) {
	normalized := input.Normalize()
	if err := normalized.Validate(); err != nil {
		return domain.Content{}, err
	}

	now := time.Now().UTC()
	contentID := fmt.Sprintf("cnt_%d", now.UnixNano())
	content := domain.NewContentFromInput(contentID, normalized, now)
	return s.contents.CreateContent(ctx, content)
}

func (s ContentService) Update(ctx context.Context, contentID string, input domain.ContentUpsertInput) (domain.Content, error) {
	if contentID == "" {
		return domain.Content{}, domain.ErrInvalidInput
	}

	normalized := input.Normalize()
	if err := normalized.Validate(); err != nil {
		return domain.Content{}, err
	}

	existing, err := s.contents.GetContent(ctx, contentID)
	if err != nil {
		return domain.Content{}, err
	}

	updated := domain.Content{
		ID:          existing.ID,
		Title:       normalized.Title,
		ContentType: normalized.ContentType,
		Level:       normalized.Level,
		Topic:       normalized.Topic,
		Language:    normalized.Language,
		RawText:     normalized.RawText,
		SummaryText: normalized.SummaryText,
		CreatedAt:   existing.CreatedAt,
		UpdatedAt:   time.Now().UTC(),
	}

	content, err := s.contents.UpdateContent(ctx, updated)
	if err != nil {
		return domain.Content{}, err
	}
	if err := s.contents.DeleteChunkingResult(ctx, contentID); err != nil {
		return domain.Content{}, err
	}
	if err := s.contents.DeleteSkeletonResult(ctx, contentID); err != nil {
		return domain.Content{}, err
	}
	return content, nil
}
