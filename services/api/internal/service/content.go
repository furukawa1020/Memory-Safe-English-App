package service

import (
	"context"

	"memory-safe-english/services/api/internal/domain"
	"memory-safe-english/services/api/internal/repository"
)

type ContentService struct {
	contents repository.ContentRepository
	analyzer ChunkAnalyzer
}

type ListContentsInput struct {
	ContentType string
	Level       string
	Topic       string
	Language    string
}

func NewContentService(contents repository.ContentRepository, analyzer ChunkAnalyzer) ContentService {
	return ContentService{
		contents: contents,
		analyzer: analyzer,
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
	result, err := s.analyzer.AnalyzeChunks(ctx, content.RawText, content.Language)
	if err != nil {
		return domain.ChunkingResult{}, err
	}
	if err := s.contents.SaveChunkingResult(ctx, contentID, result); err != nil {
		return domain.ChunkingResult{}, err
	}
	return result, nil
}
