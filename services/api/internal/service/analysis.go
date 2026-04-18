package service

import (
	"context"

	"memory-safe-english/services/api/internal/domain"
)

type ChunkAnalyzer interface {
	AnalyzeChunks(ctx context.Context, text, language string) (domain.ChunkingResult, error)
}

type AnalysisService struct {
	analyzer ChunkAnalyzer
}

func NewAnalysisService(analyzer ChunkAnalyzer) AnalysisService {
	return AnalysisService{analyzer: analyzer}
}

func (s AnalysisService) AnalyzeChunks(ctx context.Context, input domain.AnalyzeChunksInput) (domain.ChunkingResult, error) {
	normalized := input.Normalize()
	if err := normalized.Validate(); err != nil {
		return domain.ChunkingResult{}, err
	}
	return s.analyzer.AnalyzeChunks(ctx, normalized.Text, normalized.Language)
}
