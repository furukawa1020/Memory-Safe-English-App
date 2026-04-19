package service

import (
	"context"

	"memory-safe-english/services/api/internal/domain"
)

type ChunkAnalyzer interface {
	AnalyzeChunks(ctx context.Context, text, language string) (domain.ChunkingResult, error)
}

type SkeletonAnalyzer interface {
	AnalyzeSkeleton(ctx context.Context, text, language string) (domain.SkeletonResult, error)
}

type AnalysisService struct {
	chunkAnalyzer    ChunkAnalyzer
	skeletonAnalyzer SkeletonAnalyzer
}

func NewAnalysisService(chunkAnalyzer ChunkAnalyzer, skeletonAnalyzer SkeletonAnalyzer) AnalysisService {
	return AnalysisService{
		chunkAnalyzer:    chunkAnalyzer,
		skeletonAnalyzer: skeletonAnalyzer,
	}
}

func (s AnalysisService) AnalyzeChunks(ctx context.Context, input domain.AnalyzeChunksInput) (domain.ChunkingResult, error) {
	normalized := input.Normalize()
	if err := normalized.Validate(); err != nil {
		return domain.ChunkingResult{}, err
	}
	return s.chunkAnalyzer.AnalyzeChunks(ctx, normalized.Text, normalized.Language)
}

func (s AnalysisService) AnalyzeSkeleton(ctx context.Context, input domain.AnalyzeSkeletonInput) (domain.SkeletonResult, error) {
	normalized := input.Normalize()
	if err := normalized.Validate(); err != nil {
		return domain.SkeletonResult{}, err
	}
	return s.skeletonAnalyzer.AnalyzeSkeleton(ctx, normalized.Text, normalized.Language)
}
