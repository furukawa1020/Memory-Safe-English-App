package handlers

import (
	"net/http"

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
