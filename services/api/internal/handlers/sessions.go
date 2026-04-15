package handlers

import (
	"encoding/json"
	"net/http"
	"strings"
	"time"

	"memory-safe-english/services/api/internal/httpjson"
	"memory-safe-english/services/api/internal/store/memory"
)

type SessionHandler struct {
	store *memory.Store
}

func NewSessionHandler(store *memory.Store) SessionHandler {
	return SessionHandler{store: store}
}

func (h SessionHandler) Start(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		httpjson.Error(w, http.StatusMethodNotAllowed, "method_not_allowed", "method not allowed")
		return
	}

	userID := r.Header.Get("X-User-ID")
	if userID == "" {
		httpjson.Error(w, http.StatusUnauthorized, "missing_user", "X-User-ID header is required")
		return
	}

	var req struct {
		Mode      string `json:"mode"`
		ContentID string `json:"content_id"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		httpjson.Error(w, http.StatusBadRequest, "invalid_json", "request body must be valid JSON")
		return
	}
	if req.Mode == "" {
		httpjson.Error(w, http.StatusBadRequest, "invalid_request", "mode is required")
		return
	}

	session, err := h.store.StartSession(userID, req.Mode, req.ContentID)
	if err != nil {
		httpjson.Error(w, http.StatusNotFound, "user_not_found", "user not found")
		return
	}

	httpjson.Write(w, http.StatusCreated, session)
}

func (h SessionHandler) Complete(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		httpjson.Error(w, http.StatusMethodNotAllowed, "method_not_allowed", "method not allowed")
		return
	}

	sessionID, ok := sessionIDFromPath(r.URL.Path)
	if !ok {
		httpjson.Error(w, http.StatusNotFound, "not_found", "session route not found")
		return
	}

	session, err := h.store.CompleteSession(sessionID)
	if err != nil {
		httpjson.Error(w, http.StatusNotFound, "session_not_found", "session not found")
		return
	}

	httpjson.Write(w, http.StatusOK, session)
}

func (h SessionHandler) AddEvent(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		httpjson.Error(w, http.StatusMethodNotAllowed, "method_not_allowed", "method not allowed")
		return
	}

	userID := r.Header.Get("X-User-ID")
	if userID == "" {
		httpjson.Error(w, http.StatusUnauthorized, "missing_user", "X-User-ID header is required")
		return
	}

	sessionID, ok := sessionIDFromPath(r.URL.Path)
	if !ok {
		httpjson.Error(w, http.StatusNotFound, "not_found", "session route not found")
		return
	}

	var req struct {
		EventType  string         `json:"event_type"`
		OccurredAt string         `json:"occurred_at"`
		Payload    map[string]any `json:"payload"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		httpjson.Error(w, http.StatusBadRequest, "invalid_json", "request body must be valid JSON")
		return
	}
	if req.EventType == "" {
		httpjson.Error(w, http.StatusBadRequest, "invalid_request", "event_type is required")
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

	event, err := h.store.AddEvent(userID, sessionID, req.EventType, req.Payload, occurredAt)
	if err != nil {
		httpjson.Error(w, http.StatusNotFound, "session_not_found", "session not found for user")
		return
	}

	httpjson.Write(w, http.StatusCreated, event)
}

func sessionIDFromPath(path string) (string, bool) {
	trimmed := strings.Trim(path, "/")
	parts := strings.Split(trimmed, "/")
	if len(parts) != 3 {
		return "", false
	}
	if parts[0] != "sessions" {
		return "", false
	}
	return parts[1], true
}
