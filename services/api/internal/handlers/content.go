package handlers

import (
	"net/http"

	"memory-safe-english/services/api/internal/domain"
	"memory-safe-english/services/api/internal/httpjson"
	"memory-safe-english/services/api/internal/httpx"
	"memory-safe-english/services/api/internal/service"
)

type ContentHandler struct {
	service service.ContentService
}

func NewContentHandler(service service.ContentService) ContentHandler {
	return ContentHandler{service: service}
}

func (h ContentHandler) List(w http.ResponseWriter, r *http.Request) {
	result, err := h.service.List(r.Context(), service.ListContentsInput{
		ContentType: r.URL.Query().Get("type"),
		Level:       r.URL.Query().Get("level"),
		Topic:       r.URL.Query().Get("topic"),
		Language:    r.URL.Query().Get("language"),
	})
	if err != nil {
		httpx.WriteDomainError(w, err, "invalid content filters", "content not found")
		return
	}
	httpjson.Write(w, http.StatusOK, map[string]any{"items": result})
}

func (h ContentHandler) Get(w http.ResponseWriter, r *http.Request) {
	result, err := h.service.Get(r.Context(), r.PathValue("contentID"))
	if err != nil {
		httpx.WriteDomainError(w, err, "content_id is required", "content not found")
		return
	}
	httpjson.Write(w, http.StatusOK, result)
}

func (h ContentHandler) GetChunks(w http.ResponseWriter, r *http.Request) {
	result, err := h.service.GetChunks(r.Context(), r.PathValue("contentID"))
	if err != nil {
		httpx.WriteDomainError(w, err, "content_id is required", "content not found")
		return
	}
	httpjson.Write(w, http.StatusOK, result)
}

func (h ContentHandler) Create(w http.ResponseWriter, r *http.Request) {
	var req service.ListContentsInput
	_ = req
	var input struct {
		Title       string `json:"title"`
		ContentType string `json:"content_type"`
		Level       string `json:"level"`
		Topic       string `json:"topic"`
		Language    string `json:"language"`
		RawText     string `json:"raw_text"`
		SummaryText string `json:"summary_text"`
	}
	if err := httpx.DecodeJSON(r, &input); err != nil {
		httpjson.Error(w, http.StatusBadRequest, "invalid_json", "request body must be valid JSON")
		return
	}

	result, err := h.service.Create(r.Context(), domain.ContentUpsertInput(input))
	if err != nil {
		httpx.WriteDomainError(w, err, "valid content fields are required", "content not found")
		return
	}
	httpjson.Write(w, http.StatusCreated, result)
}

func (h ContentHandler) Update(w http.ResponseWriter, r *http.Request) {
	var input struct {
		Title       string `json:"title"`
		ContentType string `json:"content_type"`
		Level       string `json:"level"`
		Topic       string `json:"topic"`
		Language    string `json:"language"`
		RawText     string `json:"raw_text"`
		SummaryText string `json:"summary_text"`
	}
	if err := httpx.DecodeJSON(r, &input); err != nil {
		httpjson.Error(w, http.StatusBadRequest, "invalid_json", "request body must be valid JSON")
		return
	}

	result, err := h.service.Update(r.Context(), r.PathValue("contentID"), domain.ContentUpsertInput(input))
	if err != nil {
		httpx.WriteDomainError(w, err, "valid content fields are required", "content not found")
		return
	}
	httpjson.Write(w, http.StatusOK, result)
}
