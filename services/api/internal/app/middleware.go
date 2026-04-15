package app

import (
	"crypto/rand"
	"encoding/hex"
	"log"
	"net/http"
	"time"

	"memory-safe-english/services/api/internal/authctx"
	"memory-safe-english/services/api/internal/httpjson"
	"memory-safe-english/services/api/internal/httpx"
)

type statusRecorder struct {
	http.ResponseWriter
	statusCode int
}

func (r *statusRecorder) WriteHeader(statusCode int) {
	r.statusCode = statusCode
	r.ResponseWriter.WriteHeader(statusCode)
}

func withRequestID(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		requestID := r.Header.Get("X-Request-ID")
		if requestID == "" {
			requestID = newRequestID()
		}

		w.Header().Set("X-Request-ID", requestID)
		next.ServeHTTP(w, r.WithContext(httpx.WithRequestID(r.Context(), requestID)))
	})
}

func withLogging(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		start := time.Now()
		recorder := &statusRecorder{ResponseWriter: w, statusCode: http.StatusOK}
		next.ServeHTTP(recorder, r)

		log.Printf(
			"request_id=%s method=%s path=%s status=%d duration=%s",
			httpx.RequestID(r.Context()),
			r.Method,
			r.URL.Path,
			recorder.statusCode,
			time.Since(start).String(),
		)
	})
}

func withAuthContext(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		userID := r.Header.Get("X-User-ID")
		if userID == "" {
			httpjson.Error(w, http.StatusUnauthorized, "unauthorized", "X-User-ID header is required")
			return
		}
		next.ServeHTTP(w, r.WithContext(authctx.WithUserID(r.Context(), userID)))
	})
}

func recoverer(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		defer func() {
			if recovered := recover(); recovered != nil {
				log.Printf("request_id=%s panic recovered: %v", httpx.RequestID(r.Context()), recovered)
				httpjson.Error(w, http.StatusInternalServerError, "internal_error", "internal server error")
			}
		}()
		next.ServeHTTP(w, r)
	})
}

func chain(h http.Handler, middlewares ...func(http.Handler) http.Handler) http.Handler {
	for i := len(middlewares) - 1; i >= 0; i-- {
		h = middlewares[i](h)
	}
	return h
}

func newRequestID() string {
	buf := make([]byte, 8)
	if _, err := rand.Read(buf); err != nil {
		return time.Now().UTC().Format("20060102150405.000000000")
	}
	return hex.EncodeToString(buf)
}
