package handlers

import "net/http"

type ProtectedMiddleware func(http.Handler) http.Handler

type RouteSet struct {
	Health   HealthHandler
	Auth     AuthHandler
	Me       MeHandler
	Session  SessionHandler
	Analysis AnalysisHandler
	Content  ContentHandler
}

func RegisterRoutes(mux *http.ServeMux, routes RouteSet, protected ProtectedMiddleware) {
	mux.Handle("GET /health", routes.Health)
	mux.Handle("POST /auth/register", http.HandlerFunc(routes.Auth.Register))
	mux.Handle("POST /auth/login", http.HandlerFunc(routes.Auth.Login))
	mux.Handle("POST /auth/refresh", http.HandlerFunc(routes.Auth.Refresh))
	mux.Handle("GET /me", protected(http.HandlerFunc(routes.Me.Get)))
	mux.Handle("POST /analysis/chunks", protected(http.HandlerFunc(routes.Analysis.AnalyzeChunks)))
	mux.Handle("POST /analysis/skeleton", protected(http.HandlerFunc(routes.Analysis.AnalyzeSkeleton)))
	mux.Handle("GET /contents", protected(http.HandlerFunc(routes.Content.List)))
	mux.Handle("POST /contents", protected(http.HandlerFunc(routes.Content.Create)))
	mux.Handle("GET /contents/{contentID}", protected(http.HandlerFunc(routes.Content.Get)))
	mux.Handle("PATCH /contents/{contentID}", protected(http.HandlerFunc(routes.Content.Update)))
	mux.Handle("GET /contents/{contentID}/chunks", protected(http.HandlerFunc(routes.Content.GetChunks)))
	mux.Handle("GET /contents/{contentID}/skeleton", protected(http.HandlerFunc(routes.Content.GetSkeleton)))
	mux.Handle("POST /sessions/start", protected(http.HandlerFunc(routes.Session.Start)))
	mux.Handle("POST /sessions/{sessionID}/event", protected(http.HandlerFunc(routes.Session.AddEvent)))
	mux.Handle("POST /sessions/{sessionID}/complete", protected(http.HandlerFunc(routes.Session.Complete)))
}
