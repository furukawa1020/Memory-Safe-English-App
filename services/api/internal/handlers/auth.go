package handlers

import (
	"math"
	"net/http"
	"strconv"

	"memory-safe-english/services/api/internal/httpjson"
	"memory-safe-english/services/api/internal/httpx"
	security "memory-safe-english/services/api/internal/security"
	"memory-safe-english/services/api/internal/security/audit"
	"memory-safe-english/services/api/internal/service"
)

type AuthRateLimiters struct {
	Login    *security.AttemptLimiter
	Register *security.AttemptLimiter
	Refresh  *security.AttemptLimiter
}

type AuthHandler struct {
	service  service.AuthService
	limiters AuthRateLimiters
}

type registerRequest struct {
	Email         string `json:"email"`
	Password      string `json:"password"`
	DisplayName   string `json:"display_name"`
	AgreedToTerms bool   `json:"agreed_to_terms"`
}

func NewAuthHandler(service service.AuthService, limiters AuthRateLimiters) AuthHandler {
	return AuthHandler{
		service:  service,
		limiters: limiters,
	}
}

func (h AuthHandler) Register(w http.ResponseWriter, r *http.Request) {
	var req registerRequest
	if err := httpx.DecodeJSON(r, &req); err != nil {
		audit.LogAuthEvent(httpx.RequestID(r.Context()), "register_invalid_json", security.ClientIPFromRequest(r), security.NormalizeRateLimitSubject(req.Email), false, "request body must be valid JSON")
		httpjson.Error(w, http.StatusBadRequest, "invalid_json", "request body must be valid JSON")
		return
	}

	if !h.allowAttempt(w, r, h.limiters.Register, req.Email) {
		return
	}

	result, err := h.service.Register(r.Context(), service.RegisterInput{
		Email:         req.Email,
		Password:      req.Password,
		DisplayName:   req.DisplayName,
		AgreedToTerms: req.AgreedToTerms,
	})
	if err != nil {
		audit.LogAuthEvent(httpx.RequestID(r.Context()), "register_failed", security.ClientIPFromRequest(r), security.NormalizeRateLimitSubject(req.Email), false, err.Error())
		httpx.WriteDomainError(w, err, "email, password, display_name, and terms agreement are required", "user not found")
		return
	}

	audit.LogAuthEvent(httpx.RequestID(r.Context()), "register_succeeded", security.ClientIPFromRequest(r), security.NormalizeRateLimitSubject(req.Email), true, "")
	httpjson.Write(w, http.StatusCreated, result)
}

func (h AuthHandler) Login(w http.ResponseWriter, r *http.Request) {
	var req struct {
		Email    string `json:"email"`
		Password string `json:"password"`
	}
	if err := httpx.DecodeJSON(r, &req); err != nil {
		audit.LogAuthEvent(httpx.RequestID(r.Context()), "login_invalid_json", security.ClientIPFromRequest(r), security.NormalizeRateLimitSubject(req.Email), false, "request body must be valid JSON")
		httpjson.Error(w, http.StatusBadRequest, "invalid_json", "request body must be valid JSON")
		return
	}

	if !h.allowAttempt(w, r, h.limiters.Login, req.Email) {
		return
	}

	result, err := h.service.Login(r.Context(), req.Email, req.Password)
	if err != nil {
		audit.LogAuthEvent(httpx.RequestID(r.Context()), "login_failed", security.ClientIPFromRequest(r), security.NormalizeRateLimitSubject(req.Email), false, err.Error())
		httpx.WriteDomainError(w, err, "email and password are required", "user not found")
		return
	}

	audit.LogAuthEvent(httpx.RequestID(r.Context()), "login_succeeded", security.ClientIPFromRequest(r), security.NormalizeRateLimitSubject(req.Email), true, "")
	httpjson.Write(w, http.StatusOK, result)
}

func (h AuthHandler) Refresh(w http.ResponseWriter, r *http.Request) {
	var req struct {
		RefreshToken string `json:"refresh_token"`
	}
	if err := httpx.DecodeJSON(r, &req); err != nil {
		audit.LogAuthEvent(httpx.RequestID(r.Context()), "refresh_invalid_json", security.ClientIPFromRequest(r), "-", false, "request body must be valid JSON")
		httpjson.Error(w, http.StatusBadRequest, "invalid_json", "request body must be valid JSON")
		return
	}

	if !h.allowAttempt(w, r, h.limiters.Refresh, "") {
		return
	}

	result, err := h.service.Refresh(r.Context(), req.RefreshToken)
	if err != nil {
		audit.LogAuthEvent(httpx.RequestID(r.Context()), "refresh_failed", security.ClientIPFromRequest(r), "-", false, err.Error())
		httpx.WriteDomainError(w, err, "refresh_token is required", "user not found")
		return
	}

	audit.LogAuthEvent(httpx.RequestID(r.Context()), "refresh_succeeded", security.ClientIPFromRequest(r), "-", true, "")
	httpjson.Write(w, http.StatusOK, result)
}

func (h AuthHandler) Guest(w http.ResponseWriter, r *http.Request) {
	if !h.allowAttempt(w, r, h.limiters.Register, "guest") {
		return
	}

	result, err := h.service.Guest(r.Context())
	if err != nil {
		audit.LogAuthEvent(httpx.RequestID(r.Context()), "guest_failed", security.ClientIPFromRequest(r), "guest", false, err.Error())
		httpx.WriteDomainError(w, err, "guest session could not be created", "user not found")
		return
	}

	audit.LogAuthEvent(httpx.RequestID(r.Context()), "guest_succeeded", security.ClientIPFromRequest(r), "guest", true, "")
	httpjson.Write(w, http.StatusCreated, result)
}

func (h AuthHandler) allowAttempt(w http.ResponseWriter, r *http.Request, limiter *security.AttemptLimiter, subject string) bool {
	decision := limiter.Allow(
		security.ClientIPFromRequest(r),
		security.NormalizeRateLimitSubject(subject),
	)
	if decision.Allowed {
		return true
	}

	audit.LogAuthEvent(httpx.RequestID(r.Context()), "auth_rate_limited", security.ClientIPFromRequest(r), security.NormalizeRateLimitSubject(subject), false, "too many authentication attempts")

	retryAfterSeconds := int(math.Ceil(decision.RetryAfter.Seconds()))
	if retryAfterSeconds < 1 {
		retryAfterSeconds = 1
	}
	w.Header().Set("Retry-After", strconv.Itoa(retryAfterSeconds))
	httpjson.Error(w, http.StatusTooManyRequests, "rate_limited", "too many authentication attempts")
	return false
}
