package app

import (
	"memory-safe-english/services/api/internal/config"
	"memory-safe-english/services/api/internal/handlers"
	"memory-safe-english/services/api/internal/security/password"
	"memory-safe-english/services/api/internal/security/token"
	"memory-safe-english/services/api/internal/service"
	"memory-safe-english/services/api/internal/store/memory"
)

type Application struct {
	Config       config.Config
	Store        *memory.Store
	PasswordHash password.Hasher
	TokenManager token.Manager
}

func NewApplication(cfg config.Config) *Application {
	return &Application{
		Config:       cfg,
		Store:        memory.NewStore(),
		PasswordHash: password.NewHasher(cfg.PasswordHashIterations),
		TokenManager: token.NewManager(cfg.AuthTokenSecret, cfg.AccessTokenTTL, cfg.RefreshTokenTTL),
	}
}

func (a *Application) Routes() handlers.RouteSet {
	authService := service.NewAuthService(a.Store, a.Store, a.PasswordHash, a.TokenManager)
	userService := service.NewUserService(a.Store)
	sessionService := service.NewSessionService(a.Store, a.Store)

	return handlers.RouteSet{
		Health:  handlers.NewHealthHandler(),
		Auth:    handlers.NewAuthHandler(authService),
		Me:      handlers.NewMeHandler(userService),
		Session: handlers.NewSessionHandler(sessionService),
	}
}
