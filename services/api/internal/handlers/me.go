package handlers

import (
	"net/http"

	"memory-safe-english/services/api/internal/httpjson"
	"memory-safe-english/services/api/internal/store/memory"
)

type MeHandler struct {
	store *memory.Store
}

func NewMeHandler(store *memory.Store) MeHandler {
	return MeHandler{store: store}
}

func (h MeHandler) Get(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		httpjson.Error(w, http.StatusMethodNotAllowed, "method_not_allowed", "method not allowed")
		return
	}

	userID := r.Header.Get("X-User-ID")
	if userID == "" {
		httpjson.Error(w, http.StatusUnauthorized, "missing_user", "X-User-ID header is required")
		return
	}

	user, err := h.store.GetUser(userID)
	if err != nil {
		httpjson.Error(w, http.StatusNotFound, "user_not_found", "user not found")
		return
	}

	httpjson.Write(w, http.StatusOK, user)
}
