package handlers

import (
	"net/http"

	"memory-safe-english/services/api/internal/httpx"
	"memory-safe-english/services/api/internal/httpjson"
	"memory-safe-english/services/api/internal/service"
)

type MeHandler struct {
	service service.UserService
}

func NewMeHandler(service service.UserService) MeHandler {
	return MeHandler{service: service}
}

func (h MeHandler) Get(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		httpjson.Error(w, http.StatusMethodNotAllowed, "method_not_allowed", "method not allowed")
		return
	}

	user, err := h.service.GetMe(httpx.UserIDFromHeader(r))
	if err != nil {
		httpx.WriteDomainError(w, err, "invalid user", "user not found")
		return
	}

	httpjson.Write(w, http.StatusOK, user)
}
