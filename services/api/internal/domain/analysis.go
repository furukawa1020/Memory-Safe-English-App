package domain

import "strings"

const (
	DefaultAnalysisLanguage = "en"
	MaxAnalysisTextLength   = 4000
)

type Chunk struct {
	Order        int    `json:"order"`
	Text         string `json:"text"`
	Role         string `json:"role"`
	SkeletonRank int    `json:"skeleton_rank"`
}

type ChunkingResult struct {
	Version  string  `json:"version"`
	Language string  `json:"language"`
	Chunks   []Chunk `json:"chunks"`
	Summary  string  `json:"summary"`
}

type SkeletonPart struct {
	Order    int    `json:"order"`
	Text     string `json:"text"`
	Role     string `json:"role"`
	Emphasis int    `json:"emphasis"`
}

type SkeletonResult struct {
	Version  string         `json:"version"`
	Language string         `json:"language"`
	Parts    []SkeletonPart `json:"parts"`
	Summary  string         `json:"summary"`
}

type AnalyzeChunksInput struct {
	Text     string `json:"text"`
	Language string `json:"language"`
}

type AnalyzeSkeletonInput struct {
	Text     string `json:"text"`
	Language string `json:"language"`
}

func (in AnalyzeChunksInput) Normalize() AnalyzeChunksInput {
	text := strings.TrimSpace(in.Text)
	language := strings.TrimSpace(in.Language)
	if language == "" {
		language = DefaultAnalysisLanguage
	}
	return AnalyzeChunksInput{
		Text:     text,
		Language: strings.ToLower(language),
	}
}

func (in AnalyzeChunksInput) Validate() error {
	if strings.TrimSpace(in.Text) == "" {
		return ErrInvalidInput
	}
	if len([]rune(strings.TrimSpace(in.Text))) > MaxAnalysisTextLength {
		return ErrInvalidInput
	}
	return nil
}

func (in AnalyzeSkeletonInput) Normalize() AnalyzeSkeletonInput {
	text := strings.TrimSpace(in.Text)
	language := strings.TrimSpace(in.Language)
	if language == "" {
		language = DefaultAnalysisLanguage
	}
	return AnalyzeSkeletonInput{
		Text:     text,
		Language: strings.ToLower(language),
	}
}

func (in AnalyzeSkeletonInput) Validate() error {
	if strings.TrimSpace(in.Text) == "" {
		return ErrInvalidInput
	}
	if len([]rune(strings.TrimSpace(in.Text))) > MaxAnalysisTextLength {
		return ErrInvalidInput
	}
	return nil
}
