package app

import (
	"net/http"
	"time"

	"memory-safe-english/services/api/internal/config"
	"memory-safe-english/services/api/internal/handlers"
)

func NewServer(cfg config.Config) (*http.Server, error) {
	application := NewApplication(cfg)
	mux := http.NewServeMux()
	handlers.RegisterRoutes(mux, application.Routes(), withAuthContext)

	return &http.Server{
		Addr:              cfg.HTTPAddr,
		Handler:           chain(mux, withRequestID, withLogging, recoverer),
		ReadHeaderTimeout: 5 * time.Second,
		ReadTimeout:       10 * time.Second,
		WriteTimeout:      15 * time.Second,
		IdleTimeout:       60 * time.Second,
	}, nil
}
