package handlers

import (
	"net/http"

	"memory-safe-english/services/api/internal/httpx"
	"memory-safe-english/services/api/internal/httpjson"
	"memory-safe-english/services/api/internal/service"
)

type AuthHandler struct {
	service service.AuthService
}

type registerRequest struct {
	Email          string `json:"email"`
	Password       string `json:"password"`
	DisplayName    string `json:"display_name"`
	AgreedToTerms  bool   `json:"agreed_to_terms"`
	NativeLanguage string `json:"native_language"`
}

func NewAuthHandler(service service.AuthService) AuthHandler {
	return AuthHandler{service: service}
}

func (h AuthHandler) Register(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		httpjson.Error(w, http.StatusMethodNotAllowed, "method_not_allowed", "method not allowed")
		return
	}

	var req registerRequest
	if err := httpx.DecodeJSON(r, &req); err != nil {
		httpjson.Error(w, http.StatusBadRequest, "invalid_json", "request body must be valid JSON")
		return
	}

	result, err := h.service.Register(service.RegisterInput{
		Email:         req.Email,
		Password:      req.Password,
		DisplayName:   req.DisplayName,
		AgreedToTerms: req.AgreedToTerms,
	})
	if err != nil {
		httpx.WriteDomainError(w, err, "email, password, display_name, and terms agreement are required", "user not found")
		return
	}

	httpjson.Write(w, http.StatusCreated, result)
}

func (h AuthHandler) Login(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		httpjson.Error(w, http.StatusMethodNotAllowed, "method_not_allowed", "method not allowed")
		return
	}

	var req struct {
		UserID string `json:"user_id"`
	}
	if err := httpx.DecodeJSON(r, &req); err != nil {
		httpjson.Error(w, http.StatusBadRequest, "invalid_json", "request body must be valid JSON")
		return
	}

	result, err := h.service.Login(req.UserID)
	if err != nil {
		httpx.WriteDomainError(w, err, "user_id is required for the dev login flow", "user not found")
		return
	}

	httpjson.Write(w, http.StatusOK, result)
}
