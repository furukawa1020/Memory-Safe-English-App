package handlers

import (
	"net/http"

	"memory-safe-english/services/api/internal/authctx"
	"memory-safe-english/services/api/internal/httpjson"
	"memory-safe-english/services/api/internal/httpx"
	"memory-safe-english/services/api/internal/service"
)

type MeHandler struct {
	service service.UserService
}

func NewMeHandler(service service.UserService) MeHandler {
	return MeHandler{service: service}
}

func (h MeHandler) Get(w http.ResponseWriter, r *http.Request) {
	userID, _ := authctx.UserID(r.Context())
	user, err := h.service.GetMe(r.Context(), userID)
	if err != nil {
		httpx.WriteDomainError(w, err, "invalid user", "user not found")
		return
	}

	httpjson.Write(w, http.StatusOK, user)
}
