package httpx

import (
	"errors"
	"net/http"

	"memory-safe-english/services/api/internal/domain"
	"memory-safe-english/services/api/internal/httpjson"
)

func WriteDomainError(w http.ResponseWriter, err error, invalidMessage, notFoundMessage string) {
	switch {
	case errors.Is(err, domain.ErrUnauthorized):
		httpjson.Error(w, http.StatusUnauthorized, "unauthorized", "authentication failed")
	case errors.Is(err, domain.ErrInvalidInput):
		httpjson.Error(w, http.StatusBadRequest, "invalid_request", invalidMessage)
	case errors.Is(err, domain.ErrNotFound):
		httpjson.Error(w, http.StatusNotFound, "not_found", notFoundMessage)
	case errors.Is(err, domain.ErrConflict):
		httpjson.Error(w, http.StatusConflict, "conflict", "resource conflict")
	default:
		httpjson.Error(w, http.StatusInternalServerError, "internal_error", "internal server error")
	}
}
