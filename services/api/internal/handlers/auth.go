package handlers

import (
	"encoding/json"
	"net/http"

	"memory-safe-english/services/api/internal/httpjson"
	"memory-safe-english/services/api/internal/store/memory"
)

type AuthHandler struct {
	store *memory.Store
}

type registerRequest struct {
	Email          string `json:"email"`
	Password       string `json:"password"`
	DisplayName    string `json:"display_name"`
	AgreedToTerms  bool   `json:"agreed_to_terms"`
	NativeLanguage string `json:"native_language"`
}

func NewAuthHandler(store *memory.Store) AuthHandler {
	return AuthHandler{store: store}
}

func (h AuthHandler) Register(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		httpjson.Error(w, http.StatusMethodNotAllowed, "method_not_allowed", "method not allowed")
		return
	}

	var req registerRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		httpjson.Error(w, http.StatusBadRequest, "invalid_json", "request body must be valid JSON")
		return
	}
	if req.Email == "" || req.DisplayName == "" || req.Password == "" {
		httpjson.Error(w, http.StatusBadRequest, "invalid_request", "email, password, and display_name are required")
		return
	}
	if !req.AgreedToTerms {
		httpjson.Error(w, http.StatusBadRequest, "terms_required", "terms agreement is required")
		return
	}

	user := h.store.CreateUser(req.Email, req.DisplayName, "email")
	httpjson.Write(w, http.StatusCreated, map[string]any{
		"user": user,
		"tokens": map[string]string{
			"access_token":  "dev-access-" + user.ID,
			"refresh_token": "dev-refresh-" + user.ID,
		},
	})
}

func (h AuthHandler) Login(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		httpjson.Error(w, http.StatusMethodNotAllowed, "method_not_allowed", "method not allowed")
		return
	}

	var req struct {
		UserID string `json:"user_id"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		httpjson.Error(w, http.StatusBadRequest, "invalid_json", "request body must be valid JSON")
		return
	}
	if req.UserID == "" {
		httpjson.Error(w, http.StatusBadRequest, "invalid_request", "user_id is required for the dev login flow")
		return
	}

	user, err := h.store.GetUser(req.UserID)
	if err != nil {
		httpjson.Error(w, http.StatusNotFound, "user_not_found", "user not found")
		return
	}

	httpjson.Write(w, http.StatusOK, map[string]any{
		"user": user,
		"tokens": map[string]string{
			"access_token":  "dev-access-" + user.ID,
			"refresh_token": "dev-refresh-" + user.ID,
		},
	})
}
