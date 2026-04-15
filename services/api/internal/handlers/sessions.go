package handlers

import (
	"net/http"
	"time"

	"memory-safe-english/services/api/internal/httpx"
	"memory-safe-english/services/api/internal/httpjson"
	"memory-safe-english/services/api/internal/service"
)

type SessionHandler struct {
	service service.SessionService
}

func NewSessionHandler(service service.SessionService) SessionHandler {
	return SessionHandler{service: service}
}

func (h SessionHandler) Start(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		httpjson.Error(w, http.StatusMethodNotAllowed, "method_not_allowed", "method not allowed")
		return
	}

	var req struct {
		Mode      string `json:"mode"`
		ContentID string `json:"content_id"`
	}
	if err := httpx.DecodeJSON(r, &req); err != nil {
		httpjson.Error(w, http.StatusBadRequest, "invalid_json", "request body must be valid JSON")
		return
	}

	session, err := h.service.Start(service.StartSessionInput{
		UserID:    httpx.UserIDFromHeader(r),
		Mode:      req.Mode,
		ContentID: req.ContentID,
	})
	if err != nil {
		httpx.WriteDomainError(w, err, "mode is required", "user not found")
		return
	}

	httpjson.Write(w, http.StatusCreated, session)
}

func (h SessionHandler) Complete(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		httpjson.Error(w, http.StatusMethodNotAllowed, "method_not_allowed", "method not allowed")
		return
	}

	sessionID, action, ok := httpx.SessionAction(r.URL.Path)
	if !ok || action != "complete" {
		httpjson.Error(w, http.StatusNotFound, "not_found", "session route not found")
		return
	}

	session, err := h.service.Complete(sessionID)
	if err != nil {
		httpx.WriteDomainError(w, err, "session_id is required", "session not found")
		return
	}

	httpjson.Write(w, http.StatusOK, session)
}

func (h SessionHandler) AddEvent(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		httpjson.Error(w, http.StatusMethodNotAllowed, "method_not_allowed", "method not allowed")
		return
	}

	sessionID, action, ok := httpx.SessionAction(r.URL.Path)
	if !ok || action != "event" {
		httpjson.Error(w, http.StatusNotFound, "not_found", "session route not found")
		return
	}

	var req struct {
		EventType  string         `json:"event_type"`
		OccurredAt string         `json:"occurred_at"`
		Payload    map[string]any `json:"payload"`
	}
	if err := httpx.DecodeJSON(r, &req); err != nil {
		httpjson.Error(w, http.StatusBadRequest, "invalid_json", "request body must be valid JSON")
		return
	}

	occurredAt := time.Now().UTC()
	if req.OccurredAt != "" {
		parsed, err := time.Parse(time.RFC3339, req.OccurredAt)
		if err != nil {
			httpjson.Error(w, http.StatusBadRequest, "invalid_occurred_at", "occurred_at must be RFC3339")
			return
		}
		occurredAt = parsed
	}

	event, err := h.service.AddEvent(service.AddEventInput{
		UserID:     httpx.UserIDFromHeader(r),
		SessionID:  sessionID,
		EventType:  req.EventType,
		Payload:    req.Payload,
		OccurredAt: occurredAt,
	})
	if err != nil {
		httpx.WriteDomainError(w, err, "event_type is required", "session not found for user")
		return
	}

	httpjson.Write(w, http.StatusCreated, event)
}
