package app

import (
	"memory-safe-english/services/api/internal/config"
	"memory-safe-english/services/api/internal/handlers"
	"memory-safe-english/services/api/internal/service"
	"memory-safe-english/services/api/internal/store/memory"
)

type Application struct {
	Config config.Config
	Store  *memory.Store
}

func NewApplication(cfg config.Config) *Application {
	return &Application{
		Config: cfg,
		Store:  memory.NewStore(),
	}
}

func (a *Application) Routes() handlers.RouteSet {
	authService := service.NewAuthService(a.Store)
	userService := service.NewUserService(a.Store)
	sessionService := service.NewSessionService(a.Store, a.Store)

	return handlers.RouteSet{
		Health:  handlers.NewHealthHandler(),
		Auth:    handlers.NewAuthHandler(authService),
		Me:      handlers.NewMeHandler(userService),
		Session: handlers.NewSessionHandler(sessionService),
	}
}
