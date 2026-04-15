package handlers

import (
	"net/http"

	"memory-safe-english/services/api/internal/httpjson"
	"memory-safe-english/services/api/internal/httpx"
	"memory-safe-english/services/api/internal/service"
)

type AuthHandler struct {
	service service.AuthService
}

type registerRequest struct {
	Email         string `json:"email"`
	Password      string `json:"password"`
	DisplayName   string `json:"display_name"`
	AgreedToTerms bool   `json:"agreed_to_terms"`
}

func NewAuthHandler(service service.AuthService) AuthHandler {
	return AuthHandler{service: service}
}

func (h AuthHandler) Register(w http.ResponseWriter, r *http.Request) {
	var req registerRequest
	if err := httpx.DecodeJSON(r, &req); err != nil {
		httpjson.Error(w, http.StatusBadRequest, "invalid_json", "request body must be valid JSON")
		return
	}

	result, err := h.service.Register(r.Context(), service.RegisterInput{
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
	var req struct {
		Email    string `json:"email"`
		Password string `json:"password"`
	}
	if err := httpx.DecodeJSON(r, &req); err != nil {
		httpjson.Error(w, http.StatusBadRequest, "invalid_json", "request body must be valid JSON")
		return
	}

	result, err := h.service.Login(r.Context(), req.Email, req.Password)
	if err != nil {
		httpx.WriteDomainError(w, err, "email and password are required", "user not found")
		return
	}

	httpjson.Write(w, http.StatusOK, result)
}

func (h AuthHandler) Refresh(w http.ResponseWriter, r *http.Request) {
	var req struct {
		RefreshToken string `json:"refresh_token"`
	}
	if err := httpx.DecodeJSON(r, &req); err != nil {
		httpjson.Error(w, http.StatusBadRequest, "invalid_json", "request body must be valid JSON")
		return
	}

	result, err := h.service.Refresh(r.Context(), req.RefreshToken)
	if err != nil {
		httpx.WriteDomainError(w, err, "refresh_token is required", "user not found")
		return
	}

	httpjson.Write(w, http.StatusOK, result)
}
