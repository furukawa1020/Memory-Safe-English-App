package domain

import (
	"strings"
	"time"
)

var allowedContentTypes = map[string]struct{}{
	"reading":           {},
	"listening":         {},
	"speaking_template": {},
	"rescue":            {},
}

type ContentUpsertInput struct {
	Title       string `json:"title"`
	ContentType string `json:"content_type"`
	Level       string `json:"level"`
	Topic       string `json:"topic"`
	Language    string `json:"language"`
	RawText     string `json:"raw_text"`
	SummaryText string `json:"summary_text"`
}

func (in ContentUpsertInput) Normalize() ContentUpsertInput {
	return ContentUpsertInput{
		Title:       strings.TrimSpace(in.Title),
		ContentType: strings.TrimSpace(strings.ToLower(in.ContentType)),
		Level:       strings.TrimSpace(strings.ToLower(in.Level)),
		Topic:       strings.TrimSpace(strings.ToLower(in.Topic)),
		Language:    strings.TrimSpace(strings.ToLower(in.Language)),
		RawText:     strings.TrimSpace(in.RawText),
		SummaryText: strings.TrimSpace(in.SummaryText),
	}
}

func (in ContentUpsertInput) Validate() error {
	if in.Title == "" || in.ContentType == "" || in.Level == "" || in.Topic == "" || in.RawText == "" {
		return ErrInvalidInput
	}
	if _, ok := allowedContentTypes[in.ContentType]; !ok {
		return ErrInvalidInput
	}
	if in.Language == "" {
		return ErrInvalidInput
	}
	return nil
}

func NewContentFromInput(contentID string, input ContentUpsertInput, now time.Time) Content {
	return Content{
		ID:          contentID,
		Title:       input.Title,
		ContentType: input.ContentType,
		Level:       input.Level,
		Topic:       input.Topic,
		Language:    input.Language,
		RawText:     input.RawText,
		SummaryText: input.SummaryText,
		CreatedAt:   now,
		UpdatedAt:   now,
	}
}
