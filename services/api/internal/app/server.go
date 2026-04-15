package app

import (
	"net/http"
	"time"

	"memory-safe-english/services/api/internal/config"
	"memory-safe-english/services/api/internal/handlers"
	"memory-safe-english/services/api/internal/httpjson"
	"memory-safe-english/services/api/internal/service"
	"memory-safe-english/services/api/internal/store/memory"
)

func NewServer(cfg config.Config) (*http.Server, error) {
	store := memory.NewStore()
	authService := service.NewAuthService(store)
	userService := service.NewUserService(store)
	sessionService := service.NewSessionService(store, store)

	healthHandler := handlers.NewHealthHandler()
	authHandler := handlers.NewAuthHandler(authService)
	meHandler := handlers.NewMeHandler(userService)
	sessionHandler := handlers.NewSessionHandler(sessionService)

	mux := http.NewServeMux()
	mux.Handle("/health", withLogging(healthHandler))
	mux.Handle("/auth/register", withLogging(http.HandlerFunc(authHandler.Register)))
	mux.Handle("/auth/login", withLogging(http.HandlerFunc(authHandler.Login)))
	mux.Handle("/me", withLogging(http.HandlerFunc(meHandler.Get)))
	mux.Handle("/sessions/start", withLogging(http.HandlerFunc(sessionHandler.Start)))
	mux.Handle("/sessions/", withLogging(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		switch {
		case hasSuffix(r.URL.Path, "/event"):
			sessionHandler.AddEvent(w, r)
		case hasSuffix(r.URL.Path, "/complete"):
			sessionHandler.Complete(w, r)
		default:
			httpjson.Error(w, http.StatusNotFound, "not_found", "route not found")
		}
	})))

	return &http.Server{
		Addr:              cfg.HTTPAddr,
		Handler:           recoverer(mux),
		ReadHeaderTimeout: 5 * time.Second,
		ReadTimeout:       10 * time.Second,
		WriteTimeout:      15 * time.Second,
		IdleTimeout:       60 * time.Second,
	}, nil
}

func hasSuffix(path, suffix string) bool {
	if len(path) < len(suffix) {
		return false
	}
	return path[len(path)-len(suffix):] == suffix
}
