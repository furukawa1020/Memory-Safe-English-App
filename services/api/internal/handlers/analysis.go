package handlers

import (
	"net/http"

	"memory-safe-english/services/api/internal/httpjson"
	"memory-safe-english/services/api/internal/httpx"
	"memory-safe-english/services/api/internal/service"
)

type AnalysisHandler struct {
	service service.AnalysisService
}

func NewAnalysisHandler(service service.AnalysisService) AnalysisHandler {
	return AnalysisHandler{service: service}
}

func (h AnalysisHandler) AnalyzeChunks(w http.ResponseWriter, r *http.Request) {
	var req struct {
		Text     string `json:"text"`
		Language string `json:"language"`
	}
	if err := httpx.DecodeJSON(r, &req); err != nil {
		httpjson.Error(w, http.StatusBadRequest, "invalid_json", "request body must be valid JSON")
		return
	}

	result, err := h.service.AnalyzeChunks(r.Context(), service.AnalyzeChunksInput{
		Text:     req.Text,
		Language: req.Language,
	})
	if err != nil {
		httpx.WriteDomainError(w, err, "text is required", "analysis resource not found")
		return
	}

	httpjson.Write(w, http.StatusOK, result)
}
