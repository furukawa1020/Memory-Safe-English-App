package app

import (
	"context"
	"net/http"
	"time"

	"memory-safe-english/services/api/internal/config"
	"memory-safe-english/services/api/internal/handlers"
)

func NewServer(cfg config.Config) (*http.Server, error) {
	if err := cfg.Validate(); err != nil {
		return nil, err
	}

	application, err := NewApplication(cfg)
	if err != nil {
		return nil, err
	}
	mux := http.NewServeMux()
	handlers.RegisterRoutes(mux, application.Routes(), authMiddleware(application.TokenManager))

	server := &http.Server{
		Addr:              cfg.HTTPAddr,
		Handler:           chain(mux, withRequestID, withSecurityHeaders, withLogging, recoverer),
		ReadHeaderTimeout: 5 * time.Second,
		ReadTimeout:       10 * time.Second,
		WriteTimeout:      15 * time.Second,
		IdleTimeout:       60 * time.Second,
	}
	server.RegisterOnShutdown(func() {
		_ = application.Close(context.Background())
	})
	return server, nil
}
