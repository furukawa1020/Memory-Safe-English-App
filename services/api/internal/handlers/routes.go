package handlers

import "net/http"

type ProtectedMiddleware func(http.Handler) http.Handler

type RouteSet struct {
	Health  HealthHandler
	Auth    AuthHandler
	Me      MeHandler
	Session SessionHandler
}

func RegisterRoutes(mux *http.ServeMux, routes RouteSet, protected ProtectedMiddleware) {
	mux.Handle("GET /health", routes.Health)
	mux.Handle("POST /auth/register", http.HandlerFunc(routes.Auth.Register))
	mux.Handle("POST /auth/login", http.HandlerFunc(routes.Auth.Login))
	mux.Handle("GET /me", protected(http.HandlerFunc(routes.Me.Get)))
	mux.Handle("POST /sessions/start", protected(http.HandlerFunc(routes.Session.Start)))
	mux.Handle("POST /sessions/{sessionID}/event", protected(http.HandlerFunc(routes.Session.AddEvent)))
	mux.Handle("POST /sessions/{sessionID}/complete", protected(http.HandlerFunc(routes.Session.Complete)))
}
