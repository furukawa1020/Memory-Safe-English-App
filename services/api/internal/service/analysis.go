package service

import (
	"context"
	"strings"

	"memory-safe-english/services/api/internal/domain"
	"memory-safe-english/services/api/internal/workerclient"
)

type ChunkAnalyzer interface {
	AnalyzeChunks(ctx context.Context, text, language string) (workerclient.ChunkingResult, error)
}

type AnalysisService struct {
	analyzer ChunkAnalyzer
}

type AnalyzeChunksInput struct {
	Text     string
	Language string
}

func NewAnalysisService(analyzer ChunkAnalyzer) AnalysisService {
	return AnalysisService{analyzer: analyzer}
}

func (s AnalysisService) AnalyzeChunks(ctx context.Context, input AnalyzeChunksInput) (workerclient.ChunkingResult, error) {
	text := strings.TrimSpace(input.Text)
	language := strings.TrimSpace(input.Language)
	if text == "" {
		return workerclient.ChunkingResult{}, domain.ErrInvalidInput
	}
	if language == "" {
		language = "en"
	}
	return s.analyzer.AnalyzeChunks(ctx, text, language)
}
